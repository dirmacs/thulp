//! # thulp-skills
//!
//! Skill system for composing and executing complex tool workflows.
//!
//! Skills are predefined sequences of tool calls that accomplish higher-level tasks.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thulp_core::ToolResult;

use thulp_core::{ToolCall, Transport};

#[cfg(test)]
use async_trait::async_trait;

/// Result type for skill operations
pub type Result<T> = std::result::Result<T, SkillError>;

/// Errors that can occur in skill execution
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Skill not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// A step in a skill workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    /// Step name/identifier
    pub name: String,

    /// Tool to execute
    pub tool: String,

    /// Arguments for the tool (can reference previous step outputs)
    pub arguments: Value,

    /// Whether to continue on error
    #[serde(default)]
    pub continue_on_error: bool,
}

/// A skill definition - a sequence of tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Skill name
    pub name: String,

    /// Description of what the skill does
    pub description: String,

    /// Input parameters for the skill
    #[serde(default)]
    pub inputs: Vec<String>,

    /// Steps to execute
    pub steps: Vec<SkillStep>,
}

impl Skill {
    /// Create a new skill
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            inputs: Vec::new(),
            steps: Vec::new(),
        }
    }

    /// Add an input parameter
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.inputs.push(input.into());
        self
    }

    /// Add a step
    pub fn with_step(mut self, step: SkillStep) -> Self {
        self.steps.push(step);
        self
    }
}

impl Skill {
    /// Execute the skill with the given transport and input arguments
    pub async fn execute<T: Transport>(
        &self,
        transport: &T,
        input_args: &HashMap<String, serde_json::Value>,
    ) -> Result<SkillResult> {
        let mut step_results = Vec::new();
        let mut context = input_args.clone();

        for step in &self.steps {
            // Prepare arguments by substituting context variables
            let prepared_args = self.prepare_arguments(&step.arguments, &context)?;

            let tool_call = ToolCall {
                tool: step.tool.clone(),
                arguments: prepared_args,
            };

            match transport.call(&tool_call).await {
                Ok(result) => {
                    step_results.push((step.name.clone(), result.clone()));

                    // Add result to context for use in subsequent steps
                    context.insert(
                        step.name.clone(),
                        result.data.clone().unwrap_or(Value::Null),
                    );

                    // If this is the last step, use its result as output
                    if step_results.len() == self.steps.len() {
                        return Ok(SkillResult {
                            success: true,
                            step_results,
                            output: result.data,
                            error: None,
                        });
                    }
                }
                Err(e) => {
                    if !step.continue_on_error {
                        return Ok(SkillResult {
                            success: false,
                            step_results,
                            output: None,
                            error: Some(e.to_string()),
                        });
                    }
                    // Continue on error
                    step_results.push((step.name.clone(), ToolResult::failure(e.to_string())));
                }
            }
        }

        Ok(SkillResult {
            success: true,
            step_results,
            output: None,
            error: None,
        })
    }

    /// Prepare arguments by substituting context variables
    fn prepare_arguments(
        &self,
        args: &serde_json::Value,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // Simple string substitution for now
        // In a real implementation, this would use a proper templating engine
        let args_str = serde_json::to_string(args)
            .map_err(|e| SkillError::InvalidConfig(format!("Failed to serialize args: {}", e)))?;

        let mut processed_str = args_str.clone();

        // Replace {{variable}} placeholders with context values
        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = serde_json::to_string(value).map_err(|e| {
                SkillError::InvalidConfig(format!("Failed to serialize value: {}", e))
            })?;
            processed_str = processed_str.replace(&placeholder, &value_str);
        }

        serde_json::from_str(&processed_str).map_err(|e| {
            SkillError::InvalidConfig(format!("Failed to parse processed args: {}", e))
        })
    }
}

/// Result of executing a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    /// Whether the skill executed successfully
    pub success: bool,

    /// Results from each step
    pub step_results: Vec<(String, ToolResult)>,

    /// Final output
    pub output: Option<Value>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Registry for managing skills
#[derive(Debug, Default)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a skill
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Get a skill by name
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all skill names
    pub fn list(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }

    /// Remove a skill
    pub fn unregister(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }
}

/// Mock transport for testing
#[cfg(test)]
struct MockTransport {
    responses: HashMap<String, ToolResult>,
}

#[cfg(test)]
#[async_trait]
impl Transport for MockTransport {
    async fn connect(&mut self) -> thulp_core::Result<()> {
        Ok(())
    }

    async fn disconnect(&mut self) -> thulp_core::Result<()> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        true
    }

    async fn list_tools(&self) -> thulp_core::Result<Vec<thulp_core::ToolDefinition>> {
        Ok(vec![])
    }

    async fn call(&self, call: &ToolCall) -> thulp_core::Result<ToolResult> {
        if let Some(result) = self.responses.get(&call.tool) {
            Ok(result.clone())
        } else {
            Err(thulp_core::Error::ToolNotFound(call.tool.clone()))
        }
    }
}

#[cfg(test)]
impl MockTransport {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    fn with_response(mut self, tool_name: &str, result: ToolResult) -> Self {
        self.responses.insert(tool_name.to_string(), result);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new("test_skill", "A test skill");
        assert_eq!(skill.name, "test_skill");
        assert_eq!(skill.description, "A test skill");
    }

    #[test]
    fn test_skill_builder() {
        let skill = Skill::new("search_and_summarize", "Search and summarize results")
            .with_input("query")
            .with_step(SkillStep {
                name: "search".to_string(),
                tool: "web_search".to_string(),
                arguments: serde_json::json!({"query": "{{query}}"}),
                continue_on_error: false,
            })
            .with_step(SkillStep {
                name: "summarize".to_string(),
                tool: "summarize".to_string(),
                arguments: serde_json::json!({"text": "{{search.results}}"}),
                continue_on_error: false,
            });

        assert_eq!(skill.inputs.len(), 1);
        assert_eq!(skill.steps.len(), 2);
    }

    #[test]
    fn test_registry() {
        let mut registry = SkillRegistry::new();
        let skill = Skill::new("test", "Test skill");
        registry.register(skill);
        assert!(registry.get("test").is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_registry_unregister() {
        let mut registry = SkillRegistry::new();
        let skill = Skill::new("test", "Test skill");
        registry.register(skill);
        assert!(registry.unregister("test").is_some());
        assert_eq!(registry.list().len(), 0);
    }

    #[tokio::test]
    async fn test_skill_execution() {
        let transport = MockTransport::new()
            .with_response(
                "search",
                ToolResult::success(serde_json::json!({"results": ["result1", "result2"]})),
            )
            .with_response(
                "summarize",
                ToolResult::success(serde_json::json!("Summary of results")),
            );

        let skill = Skill::new("search_and_summarize", "Search and summarize results")
            .with_input("query")
            .with_step(SkillStep {
                name: "search".to_string(),
                tool: "search".to_string(),
                arguments: serde_json::json!({"query": "test query"}),
                continue_on_error: false,
            })
            .with_step(SkillStep {
                name: "summarize".to_string(),
                tool: "summarize".to_string(),
                arguments: serde_json::json!({"text": "summary text"}),
                continue_on_error: false,
            });

        let input_args = HashMap::new();

        let result = skill.execute(&transport, &input_args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.step_results.len(), 2);
        assert!(result.output.is_some());
    }
}
