//! # thulp-skills
//!
//! Skill system for composing and executing complex tool workflows.
//!
//! Skills are predefined sequences of tool calls that accomplish higher-level tasks.
//!
//! ## Features
//!
//! - **Skill Composition**: Define multi-step workflows as skills
//! - **Timeout Support**: Prevent hanging executions with configurable timeouts
//! - **Retry Logic**: Handle transient failures with exponential backoff
//! - **Context Propagation**: Pass results between steps using template variables
//! - **Pluggable Execution**: Use [`SkillExecutor`] trait for custom execution strategies
//! - **Lifecycle Hooks**: Observe execution with [`ExecutionHooks`]
//!
//! ## Example
//!
//! ```ignore
//! use thulp_skills::{Skill, SkillStep, DefaultSkillExecutor, ExecutionContext, SkillExecutor};
//!
//! // Define a skill
//! let skill = Skill::new("search_and_summarize", "Search and summarize results")
//!     .with_input("query")
//!     .with_step(SkillStep {
//!         name: "search".to_string(),
//!         tool: "web_search".to_string(),
//!         arguments: json!({"query": "{{query}}"}),
//!         ..Default::default()
//!     })
//!     .with_step(SkillStep {
//!         name: "summarize".to_string(),
//!         tool: "summarize".to_string(),
//!         arguments: json!({"text": "{{search}}"}),
//!         ..Default::default()
//!     });
//!
//! // Execute with the default executor
//! let executor = DefaultSkillExecutor::new(transport);
//! let mut context = ExecutionContext::new()
//!     .with_input("query", json!("rust async programming"));
//!
//! let result = executor.execute(&skill, &mut context).await?;
//! ```

pub mod config;
pub mod default_executor;
pub mod executor;
pub mod hooks;
pub mod retry;
pub mod timeout;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thulp_core::ToolResult;

use thulp_core::{ToolCall, Transport};

pub use config::{
    BackoffStrategy, ExecutionConfig, RetryConfig, RetryableError, TimeoutAction, TimeoutConfig,
};
pub use default_executor::DefaultSkillExecutor;
pub use executor::{ExecutionContext, SkillExecutor, StepResult};
pub use hooks::{CompositeHooks, ExecutionHooks, NoOpHooks, TracingHooks};
pub use retry::{calculate_delay, is_error_retryable, with_retry, RetryError};
pub use timeout::{with_timeout, with_timeout_infallible, TimeoutError};

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

    #[error("Step '{step}' timed out after {duration:?}")]
    StepTimeout {
        step: String,
        duration: std::time::Duration,
    },

    #[error("Skill timed out after {duration:?}")]
    SkillTimeout { duration: std::time::Duration },

    #[error("Step '{step}' failed after {attempts} attempts: {message}")]
    RetryExhausted {
        step: String,
        attempts: usize,
        message: String,
    },
}

/// A step in a skill workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillStep {
    /// Step name/identifier
    pub name: String,

    /// Tool to execute
    pub tool: String,

    /// Arguments for the tool (can reference previous step outputs)
    #[serde(default)]
    pub arguments: Value,

    /// Whether to continue on error
    #[serde(default)]
    pub continue_on_error: bool,

    /// Optional per-step timeout override (in seconds)
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Optional per-step max retries override
    #[serde(default)]
    pub max_retries: Option<usize>,
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
        self.execute_with_config(transport, input_args, &ExecutionConfig::default())
            .await
    }

    /// Execute the skill with custom timeout and retry configuration.
    ///
    /// This method wraps each step execution with timeout and retry logic
    /// based on the provided configuration. Per-step overrides in `SkillStep`
    /// take precedence over the global configuration.
    ///
    /// # Arguments
    ///
    /// * `transport` - The transport to use for tool calls
    /// * `input_args` - Input arguments for the skill
    /// * `config` - Execution configuration for timeouts and retries
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use thulp_skills::{Skill, ExecutionConfig, TimeoutConfig, RetryConfig};
    /// use std::time::Duration;
    ///
    /// let config = ExecutionConfig::new()
    ///     .with_timeout(TimeoutConfig::new().with_step_timeout(Duration::from_secs(30)))
    ///     .with_retry(RetryConfig::new().with_max_retries(2));
    ///
    /// let result = skill.execute_with_config(&transport, &args, &config).await?;
    /// ```
    pub async fn execute_with_config<T: Transport>(
        &self,
        transport: &T,
        input_args: &HashMap<String, serde_json::Value>,
        config: &ExecutionConfig,
    ) -> Result<SkillResult> {
        let skill_timeout = config.timeout.skill_timeout;

        // Wrap entire execution in skill-level timeout
        let result = tokio::time::timeout(skill_timeout, async {
            self.execute_steps_with_config(transport, input_args, config)
                .await
        })
        .await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_elapsed) => {
                // Handle based on timeout action
                match config.timeout.timeout_action {
                    TimeoutAction::Fail => Err(SkillError::SkillTimeout {
                        duration: skill_timeout,
                    }),
                    TimeoutAction::Skip | TimeoutAction::Partial => {
                        // Return partial result - but we don't have one at skill level
                        Ok(SkillResult {
                            success: false,
                            step_results: vec![],
                            output: None,
                            error: Some(format!("Skill timed out after {:?}", skill_timeout)),
                        })
                    }
                }
            }
        }
    }

    /// Internal method to execute steps with config (used within skill timeout)
    async fn execute_steps_with_config<T: Transport>(
        &self,
        transport: &T,
        input_args: &HashMap<String, serde_json::Value>,
        config: &ExecutionConfig,
    ) -> Result<SkillResult> {
        use std::time::Duration;

        let mut step_results = Vec::new();
        let mut context = input_args.clone();

        for step in &self.steps {
            // Determine timeout for this step (per-step override or global)
            let step_timeout = step
                .timeout_secs
                .map(Duration::from_secs)
                .unwrap_or(config.timeout.step_timeout);

            // Determine max retries for this step
            let max_retries = step.max_retries.unwrap_or(config.retry.max_retries);
            let step_retry_config = RetryConfig {
                max_retries,
                ..config.retry.clone()
            };

            // Prepare arguments
            let prepared_args = self.prepare_arguments(&step.arguments, &context)?;

            let tool_call = ToolCall {
                tool: step.tool.clone(),
                arguments: prepared_args,
            };

            // Execute with retry and timeout
            let step_result = self
                .execute_step_with_retry_timeout(
                    transport,
                    &tool_call,
                    &step.name,
                    step_timeout,
                    &step_retry_config,
                )
                .await;

            match step_result {
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
                    if step.continue_on_error {
                        // Continue on error
                        step_results.push((step.name.clone(), ToolResult::failure(e.to_string())));
                    } else {
                        // Check timeout action for Skip/Partial behavior
                        match &config.timeout.timeout_action {
                            TimeoutAction::Skip => {
                                step_results
                                    .push((step.name.clone(), ToolResult::failure(e.to_string())));
                                // Continue to next step
                            }
                            TimeoutAction::Partial => {
                                return Ok(SkillResult {
                                    success: false,
                                    step_results,
                                    output: None,
                                    error: Some(e.to_string()),
                                });
                            }
                            TimeoutAction::Fail => {
                                return Err(e);
                            }
                        }
                    }
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

    /// Execute a single step with timeout and retry
    async fn execute_step_with_retry_timeout<T: Transport>(
        &self,
        transport: &T,
        tool_call: &ToolCall,
        step_name: &str,
        timeout: std::time::Duration,
        retry_config: &RetryConfig,
    ) -> Result<ToolResult> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            // Execute with timeout
            let result = tokio::time::timeout(timeout, transport.call(tool_call)).await;

            match result {
                Ok(Ok(tool_result)) => {
                    // Success!
                    return Ok(tool_result);
                }
                Ok(Err(e)) => {
                    // Transport error - check if retryable
                    let error_msg = e.to_string();

                    if attempts > retry_config.max_retries
                        || !is_error_retryable(&error_msg, retry_config)
                    {
                        return Err(SkillError::RetryExhausted {
                            step: step_name.to_string(),
                            attempts,
                            message: error_msg,
                        });
                    }

                    let delay = calculate_delay(retry_config, attempts);
                    tracing::warn!(
                        step = step_name,
                        attempt = attempts,
                        max_retries = retry_config.max_retries,
                        delay_ms = delay.as_millis() as u64,
                        error = %e,
                        "Retrying step after error"
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(_elapsed) => {
                    // Timeout - check if retryable
                    if attempts > retry_config.max_retries
                        || !retry_config
                            .retryable_errors
                            .contains(&RetryableError::Timeout)
                    {
                        return Err(SkillError::StepTimeout {
                            step: step_name.to_string(),
                            duration: timeout,
                        });
                    }

                    let delay = calculate_delay(retry_config, attempts);
                    tracing::warn!(
                        step = step_name,
                        attempt = attempts,
                        max_retries = retry_config.max_retries,
                        delay_ms = delay.as_millis() as u64,
                        "Retrying step after timeout"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
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
    use std::time::Duration;

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
                timeout_secs: None,
                max_retries: None,
            })
            .with_step(SkillStep {
                name: "summarize".to_string(),
                tool: "summarize".to_string(),
                arguments: serde_json::json!({"text": "{{search.results}}"}),
                continue_on_error: false,
                timeout_secs: Some(30),
                max_retries: Some(2),
            });

        assert_eq!(skill.inputs.len(), 1);
        assert_eq!(skill.steps.len(), 2);
        assert_eq!(skill.steps[1].timeout_secs, Some(30));
        assert_eq!(skill.steps[1].max_retries, Some(2));
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
                timeout_secs: None,
                max_retries: None,
            })
            .with_step(SkillStep {
                name: "summarize".to_string(),
                tool: "summarize".to_string(),
                arguments: serde_json::json!({"text": "summary text"}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            });

        let input_args = HashMap::new();

        let result = skill.execute(&transport, &input_args).await.unwrap();
        assert!(result.success);
        assert_eq!(result.step_results.len(), 2);
        assert!(result.output.is_some());
    }

    #[tokio::test]
    async fn test_skill_execution_with_config() {
        let transport = MockTransport::new().with_response(
            "test_tool",
            ToolResult::success(serde_json::json!({"ok": true})),
        );

        let skill = Skill::new("test_skill", "Test skill").with_step(SkillStep {
            name: "step1".to_string(),
            tool: "test_tool".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

        let config = ExecutionConfig::new()
            .with_timeout(TimeoutConfig::new().with_step_timeout(Duration::from_secs(10)))
            .with_retry(RetryConfig::no_retries());

        let result = skill
            .execute_with_config(&transport, &HashMap::new(), &config)
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.step_results.len(), 1);
    }

    #[tokio::test]
    async fn test_skill_step_timeout() {
        // Create a transport that delays response
        struct SlowTransport;

        #[async_trait]
        impl Transport for SlowTransport {
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
            async fn call(&self, _call: &ToolCall) -> thulp_core::Result<ToolResult> {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(ToolResult::success(serde_json::json!({})))
            }
        }

        let skill = Skill::new("slow_skill", "Slow skill").with_step(SkillStep {
            name: "slow_step".to_string(),
            tool: "slow_tool".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

        let config = ExecutionConfig::new()
            .with_timeout(TimeoutConfig::new().with_step_timeout(Duration::from_millis(50)))
            .with_retry(RetryConfig::no_retries());

        let result = skill
            .execute_with_config(&SlowTransport, &HashMap::new(), &config)
            .await;

        assert!(result.is_err());
        match result {
            Err(SkillError::StepTimeout { step, .. }) => {
                assert_eq!(step, "slow_step");
            }
            _ => panic!("Expected StepTimeout error"),
        }
    }

    #[tokio::test]
    async fn test_skill_step_per_step_timeout_override() {
        struct SlowTransport;

        #[async_trait]
        impl Transport for SlowTransport {
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
            async fn call(&self, _call: &ToolCall) -> thulp_core::Result<ToolResult> {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(ToolResult::success(serde_json::json!({})))
            }
        }

        let skill = Skill::new("skill", "Skill with per-step timeout").with_step(SkillStep {
            name: "step".to_string(),
            tool: "tool".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: Some(1), // Override: 1 second should be enough
            max_retries: Some(0),
        });

        // Global config has very short timeout, but step overrides it
        let config = ExecutionConfig::new()
            .with_timeout(TimeoutConfig::new().with_step_timeout(Duration::from_millis(10)));

        let result = skill
            .execute_with_config(&SlowTransport, &HashMap::new(), &config)
            .await;

        // Should succeed because per-step timeout (1s) overrides global (10ms)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_skill_continue_on_error() {
        let transport = MockTransport::new().with_response(
            "step2",
            ToolResult::success(serde_json::json!({"ok": true})),
        );

        let skill = Skill::new("skill", "Skill with continue_on_error")
            .with_step(SkillStep {
                name: "step1".to_string(),
                tool: "missing_tool".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: true, // Continue even if this fails
                timeout_secs: None,
                max_retries: Some(0),
            })
            .with_step(SkillStep {
                name: "step2".to_string(),
                tool: "step2".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            });

        let config = ExecutionConfig::new().with_retry(RetryConfig::no_retries());

        let result = skill
            .execute_with_config(&transport, &HashMap::new(), &config)
            .await
            .unwrap();

        // Should complete because continue_on_error=true for step1
        assert!(result.success);
        assert_eq!(result.step_results.len(), 2);

        // First step should have failed
        let (_, step1_result) = &result.step_results[0];
        assert!(!step1_result.is_success());

        // Second step should have succeeded
        let (_, step2_result) = &result.step_results[1];
        assert!(step2_result.is_success());
    }

    #[test]
    fn test_skill_step_serialization() {
        let step = SkillStep {
            name: "test".to_string(),
            tool: "tool".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: Some(30),
            max_retries: Some(2),
        };

        let json = serde_json::to_string(&step).unwrap();
        let deserialized: SkillStep = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.timeout_secs, Some(30));
        assert_eq!(deserialized.max_retries, Some(2));
    }

    #[test]
    fn test_skill_step_default_optional_fields() {
        let json = r#"{"name": "test", "tool": "tool", "arguments": {}}"#;
        let step: SkillStep = serde_json::from_str(json).unwrap();

        assert!(!step.continue_on_error);
        assert_eq!(step.timeout_secs, None);
        assert_eq!(step.max_retries, None);
    }
}
