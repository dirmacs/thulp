//! Skill execution abstraction.
//!
//! This module provides the [`SkillExecutor`] trait for pluggable skill execution,
//! along with [`ExecutionContext`] for managing state between steps.
//!
//! # Example
//!
//! ```ignore
//! use thulp_skills::{SkillExecutor, ExecutionContext, DefaultSkillExecutor};
//!
//! let executor = DefaultSkillExecutor::new(transport);
//! let mut context = ExecutionContext::new()
//!     .with_input("query", json!("search term"));
//!
//! let result = executor.execute(&skill, &mut context).await?;
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::{ExecutionConfig, Skill, SkillError, SkillResult, SkillStep};

/// Result of executing a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Name of the step that was executed
    pub step_name: String,

    /// Whether the step completed successfully
    pub success: bool,

    /// Output data from the step
    pub output: Option<Value>,

    /// Error message if the step failed
    pub error: Option<String>,

    /// Duration of the step execution in milliseconds
    pub duration_ms: u64,

    /// Number of retry attempts made
    pub retry_attempts: usize,
}

impl StepResult {
    /// Create a successful step result.
    pub fn success(step_name: impl Into<String>, output: Option<Value>, duration_ms: u64) -> Self {
        Self {
            step_name: step_name.into(),
            success: true,
            output,
            error: None,
            duration_ms,
            retry_attempts: 0,
        }
    }

    /// Create a failed step result.
    pub fn failure(
        step_name: impl Into<String>,
        error: impl Into<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            step_name: step_name.into(),
            success: false,
            output: None,
            error: Some(error.into()),
            duration_ms,
            retry_attempts: 0,
        }
    }

    /// Set the number of retry attempts.
    pub fn with_retry_attempts(mut self, attempts: usize) -> Self {
        self.retry_attempts = attempts;
        self
    }
}

/// Context passed through skill execution, carrying inputs, outputs, and configuration.
///
/// The execution context maintains state between steps, allowing later steps to
/// reference outputs from earlier steps.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Input arguments provided to the skill
    inputs: HashMap<String, Value>,

    /// Outputs from completed steps, keyed by step name
    outputs: HashMap<String, Value>,

    /// Execution configuration (timeouts, retries, etc.)
    config: ExecutionConfig,

    /// Optional metadata for tracking/debugging
    metadata: HashMap<String, Value>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionContext {
    /// Create a new empty execution context with default configuration.
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            config: ExecutionConfig::default(),
            metadata: HashMap::new(),
        }
    }

    /// Create a context from input arguments.
    pub fn from_inputs(inputs: HashMap<String, Value>) -> Self {
        Self {
            inputs,
            outputs: HashMap::new(),
            config: ExecutionConfig::default(),
            metadata: HashMap::new(),
        }
    }

    /// Add an input value.
    pub fn with_input(mut self, key: impl Into<String>, value: Value) -> Self {
        self.inputs.insert(key.into(), value);
        self
    }

    /// Set the execution configuration.
    pub fn with_config(mut self, config: ExecutionConfig) -> Self {
        self.config = config;
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get an input value by key.
    pub fn get_input(&self, key: &str) -> Option<&Value> {
        self.inputs.get(key)
    }

    /// Get all inputs.
    pub fn inputs(&self) -> &HashMap<String, Value> {
        &self.inputs
    }

    /// Get an output value by step name.
    pub fn get_output(&self, step_name: &str) -> Option<&Value> {
        self.outputs.get(step_name)
    }

    /// Get all outputs.
    pub fn outputs(&self) -> &HashMap<String, Value> {
        &self.outputs
    }

    /// Set an output value for a step.
    pub fn set_output(&mut self, step_name: impl Into<String>, value: Value) {
        self.outputs.insert(step_name.into(), value);
    }

    /// Get the execution configuration.
    pub fn config(&self) -> &ExecutionConfig {
        &self.config
    }

    /// Get mutable reference to the execution configuration.
    pub fn config_mut(&mut self) -> &mut ExecutionConfig {
        &mut self.config
    }

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    /// Get all metadata.
    pub fn metadata(&self) -> &HashMap<String, Value> {
        &self.metadata
    }

    /// Set metadata value.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get a combined view of inputs and outputs for variable substitution.
    ///
    /// Outputs take precedence over inputs if there are key conflicts.
    pub fn variables(&self) -> HashMap<String, Value> {
        let mut vars = self.inputs.clone();
        vars.extend(self.outputs.clone());
        vars
    }

    /// Clear all outputs (useful for re-execution).
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }
}

/// Trait for executing skills.
///
/// This trait abstracts the execution of skills, allowing different implementations
/// to handle execution in different ways (e.g., local execution, remote execution,
/// cached execution, etc.).
///
/// # Example
///
/// ```ignore
/// use thulp_skills::{SkillExecutor, ExecutionContext};
///
/// struct MyExecutor;
///
/// #[async_trait]
/// impl SkillExecutor for MyExecutor {
///     async fn execute(
///         &self,
///         skill: &Skill,
///         context: &mut ExecutionContext,
///     ) -> Result<SkillResult, SkillError> {
///         // Custom execution logic
///     }
///
///     async fn execute_step(
///         &self,
///         step: &SkillStep,
///         context: &mut ExecutionContext,
///     ) -> Result<StepResult, SkillError> {
///         // Custom step execution logic
///     }
/// }
/// ```
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    /// Execute a complete skill.
    ///
    /// This method executes all steps in the skill sequentially, passing
    /// outputs from earlier steps to later steps via the context.
    ///
    /// # Arguments
    ///
    /// * `skill` - The skill to execute
    /// * `context` - Mutable execution context for inputs/outputs
    ///
    /// # Returns
    ///
    /// A `SkillResult` containing the outcome of all steps.
    async fn execute(
        &self,
        skill: &Skill,
        context: &mut ExecutionContext,
    ) -> Result<SkillResult, SkillError>;

    /// Execute a single step.
    ///
    /// This method executes a single step from a skill workflow.
    ///
    /// # Arguments
    ///
    /// * `step` - The step to execute
    /// * `context` - Mutable execution context for inputs/outputs
    ///
    /// # Returns
    ///
    /// A `StepResult` containing the outcome of the step.
    async fn execute_step(
        &self,
        step: &SkillStep,
        context: &mut ExecutionContext,
    ) -> Result<StepResult, SkillError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_result_success() {
        let result = StepResult::success("step1", Some(serde_json::json!({"data": "test"})), 100);

        assert!(result.success);
        assert_eq!(result.step_name, "step1");
        assert!(result.output.is_some());
        assert!(result.error.is_none());
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_step_result_failure() {
        let result = StepResult::failure("step1", "something went wrong", 50);

        assert!(!result.success);
        assert_eq!(result.step_name, "step1");
        assert!(result.output.is_none());
        assert_eq!(result.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_step_result_with_retry_attempts() {
        let result = StepResult::success("step1", None, 100).with_retry_attempts(3);

        assert_eq!(result.retry_attempts, 3);
    }

    #[test]
    fn test_execution_context_new() {
        let context = ExecutionContext::new();

        assert!(context.inputs().is_empty());
        assert!(context.outputs().is_empty());
        assert!(context.metadata().is_empty());
    }

    #[test]
    fn test_execution_context_with_inputs() {
        let context = ExecutionContext::new()
            .with_input("query", serde_json::json!("test"))
            .with_input("limit", serde_json::json!(10));

        assert_eq!(context.inputs().len(), 2);
        assert_eq!(context.get_input("query"), Some(&serde_json::json!("test")));
        assert_eq!(context.get_input("limit"), Some(&serde_json::json!(10)));
    }

    #[test]
    fn test_execution_context_from_inputs() {
        let mut inputs = HashMap::new();
        inputs.insert("key".to_string(), serde_json::json!("value"));

        let context = ExecutionContext::from_inputs(inputs);

        assert_eq!(context.inputs().len(), 1);
        assert_eq!(context.get_input("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_execution_context_outputs() {
        let mut context = ExecutionContext::new();

        context.set_output("step1", serde_json::json!({"result": 42}));

        assert_eq!(
            context.get_output("step1"),
            Some(&serde_json::json!({"result": 42}))
        );
    }

    #[test]
    fn test_execution_context_variables() {
        let mut context = ExecutionContext::new()
            .with_input("input1", serde_json::json!("in"))
            .with_input("shared", serde_json::json!("from_input"));

        context.set_output("step1", serde_json::json!("out"));
        context.set_output("shared", serde_json::json!("from_output"));

        let vars = context.variables();

        // Should have all keys
        assert_eq!(vars.len(), 3);
        // Outputs override inputs for shared keys
        assert_eq!(vars.get("shared"), Some(&serde_json::json!("from_output")));
    }

    #[test]
    fn test_execution_context_metadata() {
        let context = ExecutionContext::new()
            .with_metadata("trace_id", serde_json::json!("abc123"))
            .with_metadata("user_id", serde_json::json!(42));

        assert_eq!(context.metadata().len(), 2);
        assert_eq!(
            context.get_metadata("trace_id"),
            Some(&serde_json::json!("abc123"))
        );
    }

    #[test]
    fn test_execution_context_clear_outputs() {
        let mut context = ExecutionContext::new();
        context.set_output("step1", serde_json::json!("result"));

        assert_eq!(context.outputs().len(), 1);

        context.clear_outputs();

        assert!(context.outputs().is_empty());
    }

    #[test]
    fn test_execution_context_with_config() {
        use crate::TimeoutConfig;
        use std::time::Duration;

        let config = ExecutionConfig::new()
            .with_timeout(TimeoutConfig::new().with_step_timeout(Duration::from_secs(60)));

        let context = ExecutionContext::new().with_config(config);

        assert_eq!(
            context.config().timeout.step_timeout,
            Duration::from_secs(60)
        );
    }
}
