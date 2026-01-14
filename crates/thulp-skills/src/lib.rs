//! # thulp-skills
//!
//! Skill system for composing and executing complex tool workflows.
//!
//! Skills are predefined sequences of tool calls that accomplish higher-level tasks.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thulp_core::ToolResult;

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
}
