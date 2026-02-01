//! Default skill executor implementation.
//!
//! This module provides [`DefaultSkillExecutor`], the standard implementation
//! of [`SkillExecutor`] that executes skills using a [`Transport`].
//!
//! # Example
//!
//! ```ignore
//! use thulp_skills::{DefaultSkillExecutor, ExecutionContext, NoOpHooks};
//! use thulp_core::Transport;
//!
//! let executor = DefaultSkillExecutor::new(transport);
//! let mut context = ExecutionContext::new()
//!     .with_input("query", json!("search term"));
//!
//! let result = executor.execute(&skill, &mut context).await?;
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::Value;
use thulp_core::{ToolCall, ToolResult, Transport};

use crate::{
    calculate_delay, is_error_retryable, ExecutionConfig, ExecutionContext, ExecutionHooks,
    NoOpHooks, RetryConfig, RetryableError, Skill, SkillError, SkillExecutor, SkillResult,
    SkillStep, StepResult, TimeoutAction,
};

/// Default skill executor that uses a [`Transport`] to execute tool calls.
///
/// This executor implements the standard skill execution flow:
/// 1. Execute steps sequentially
/// 2. Apply timeout and retry logic per step
/// 3. Propagate outputs from earlier steps to later steps
/// 4. Invoke lifecycle hooks at appropriate points
///
/// # Type Parameters
///
/// * `T` - The transport type for executing tool calls
/// * `H` - The hooks type for lifecycle callbacks (defaults to [`NoOpHooks`])
///
/// # Example
///
/// ```ignore
/// use thulp_skills::{DefaultSkillExecutor, ExecutionContext, TracingHooks};
///
/// // With default no-op hooks
/// let executor = DefaultSkillExecutor::new(transport);
///
/// // With tracing hooks
/// let executor = DefaultSkillExecutor::with_hooks(transport, TracingHooks::new());
///
/// let mut context = ExecutionContext::new()
///     .with_input("query", json!("test"));
///
/// let result = executor.execute(&skill, &mut context).await?;
/// ```
pub struct DefaultSkillExecutor<T, H = NoOpHooks> {
    transport: Arc<T>,
    hooks: Arc<H>,
}

impl<T: Transport> DefaultSkillExecutor<T, NoOpHooks> {
    /// Create a new executor with the given transport and no-op hooks.
    pub fn new(transport: T) -> Self {
        Self {
            transport: Arc::new(transport),
            hooks: Arc::new(NoOpHooks),
        }
    }
}

impl<T: Transport, H: ExecutionHooks> DefaultSkillExecutor<T, H> {
    /// Create a new executor with the given transport and hooks.
    pub fn with_hooks(transport: T, hooks: H) -> Self {
        Self {
            transport: Arc::new(transport),
            hooks: Arc::new(hooks),
        }
    }

    /// Create a new executor from Arc-wrapped transport and hooks.
    ///
    /// This is useful when you want to share the transport or hooks
    /// across multiple executors.
    pub fn from_arcs(transport: Arc<T>, hooks: Arc<H>) -> Self {
        Self { transport, hooks }
    }

    /// Get a reference to the transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Get a reference to the hooks.
    pub fn hooks(&self) -> &H {
        &self.hooks
    }

    /// Prepare arguments by substituting context variables.
    ///
    /// This handles two cases:
    /// 1. Entire string values like `"{{var}}"` → replaced with actual JSON value
    /// 2. Embedded placeholders like `"prefix {{var}} suffix"` → string interpolation
    fn prepare_arguments(
        &self,
        args: &Value,
        context: &ExecutionContext,
    ) -> Result<Value, SkillError> {
        self.substitute_value(args, &context.variables())
    }

    /// Recursively substitute variables in a JSON value.
    fn substitute_value(
        &self,
        value: &Value,
        variables: &std::collections::HashMap<String, Value>,
    ) -> Result<Value, SkillError> {
        match value {
            Value::String(s) => {
                // Check if the entire string is a single placeholder like "{{var}}"
                let trimmed = s.trim();
                if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
                    let inner = &trimmed[2..trimmed.len() - 2];
                    // Check if it's a simple variable reference (no other text)
                    if !inner.contains("{{") && !inner.contains("}}") {
                        let var_name = inner.trim();
                        if let Some(var_value) = variables.get(var_name) {
                            return Ok(var_value.clone());
                        }
                    }
                }

                // Otherwise, do string interpolation
                let mut result = s.clone();
                for (key, var_value) in variables {
                    let placeholder = format!("{{{{{}}}}}", key);
                    if result.contains(&placeholder) {
                        // For string interpolation, convert value to string representation
                        let replacement = match var_value {
                            Value::String(s) => s.clone(),
                            Value::Null => "null".to_string(),
                            Value::Bool(b) => b.to_string(),
                            Value::Number(n) => n.to_string(),
                            _ => serde_json::to_string(var_value).map_err(|e| {
                                SkillError::InvalidConfig(format!(
                                    "Failed to serialize value: {}",
                                    e
                                ))
                            })?,
                        };
                        result = result.replace(&placeholder, &replacement);
                    }
                }
                Ok(Value::String(result))
            }
            Value::Array(arr) => {
                let substituted: Result<Vec<Value>, SkillError> = arr
                    .iter()
                    .map(|v| self.substitute_value(v, variables))
                    .collect();
                Ok(Value::Array(substituted?))
            }
            Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.substitute_value(v, variables)?);
                }
                Ok(Value::Object(new_obj))
            }
            // Numbers, booleans, nulls pass through unchanged
            _ => Ok(value.clone()),
        }
    }

    /// Execute a single step with timeout and retry logic.
    async fn execute_step_with_retry_timeout(
        &self,
        tool_call: &ToolCall,
        step: &SkillStep,
        timeout: Duration,
        retry_config: &RetryConfig,
        context: &ExecutionContext,
    ) -> Result<(ToolResult, usize), SkillError> {
        let mut attempts = 0;

        loop {
            attempts += 1;

            // Execute with timeout
            let result = tokio::time::timeout(timeout, self.transport.call(tool_call)).await;

            match result {
                Ok(Ok(tool_result)) => {
                    // Success!
                    return Ok((tool_result, attempts - 1)); // attempts-1 = retry count
                }
                Ok(Err(e)) => {
                    // Transport error - check if retryable
                    let error_msg = e.to_string();

                    if attempts > retry_config.max_retries
                        || !is_error_retryable(&error_msg, retry_config)
                    {
                        return Err(SkillError::RetryExhausted {
                            step: step.name.clone(),
                            attempts,
                            message: error_msg,
                        });
                    }

                    // Notify hooks about retry
                    self.hooks.on_retry(step, attempts, &error_msg, context);

                    let delay = calculate_delay(retry_config, attempts);
                    tracing::warn!(
                        step = %step.name,
                        attempt = attempts,
                        max_retries = retry_config.max_retries,
                        delay_ms = delay.as_millis() as u64,
                        error = %e,
                        "Retrying step after error"
                    );
                    tokio::time::sleep(delay).await;
                }
                Err(_elapsed) => {
                    // Timeout - notify hooks
                    self.hooks
                        .on_timeout(step, timeout.as_millis() as u64, context);

                    // Check if retryable
                    if attempts > retry_config.max_retries
                        || !retry_config
                            .retryable_errors
                            .contains(&RetryableError::Timeout)
                    {
                        return Err(SkillError::StepTimeout {
                            step: step.name.clone(),
                            duration: timeout,
                        });
                    }

                    // Notify hooks about retry
                    self.hooks.on_retry(step, attempts, "timeout", context);

                    let delay = calculate_delay(retry_config, attempts);
                    tracing::warn!(
                        step = %step.name,
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
}

#[async_trait]
impl<T: Transport, H: ExecutionHooks> SkillExecutor for DefaultSkillExecutor<T, H> {
    async fn execute(
        &self,
        skill: &Skill,
        context: &mut ExecutionContext,
    ) -> Result<SkillResult, SkillError> {
        // Notify hooks
        self.hooks.before_skill(skill, context);

        let config = context.config().clone();
        let skill_timeout = config.timeout.skill_timeout;

        // Wrap entire execution in skill-level timeout
        let result = tokio::time::timeout(skill_timeout, async {
            self.execute_steps(skill, context, &config).await
        })
        .await;

        let skill_result = match result {
            Ok(inner_result) => inner_result,
            Err(_elapsed) => {
                // Handle based on timeout action
                match config.timeout.timeout_action {
                    TimeoutAction::Fail => {
                        let error = SkillError::SkillTimeout {
                            duration: skill_timeout,
                        };
                        self.hooks.on_error(&error, context);
                        Err(error)
                    }
                    TimeoutAction::Skip | TimeoutAction::Partial => {
                        // Return partial result
                        Ok(SkillResult {
                            success: false,
                            step_results: vec![],
                            output: None,
                            error: Some(format!("Skill timed out after {:?}", skill_timeout)),
                        })
                    }
                }
            }
        };

        // Notify hooks with result
        match &skill_result {
            Ok(result) => {
                self.hooks.after_skill(skill, result, context);
            }
            Err(e) => {
                self.hooks.on_error(e, context);
                // Still call after_skill with a failure result
                let failure_result = SkillResult {
                    success: false,
                    step_results: vec![],
                    output: None,
                    error: Some(e.to_string()),
                };
                self.hooks.after_skill(skill, &failure_result, context);
            }
        }

        skill_result
    }

    async fn execute_step(
        &self,
        step: &SkillStep,
        context: &mut ExecutionContext,
    ) -> Result<StepResult, SkillError> {
        let config = context.config().clone();

        // Determine timeout for this step
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
        let prepared_args = self.prepare_arguments(&step.arguments, context)?;

        let tool_call = ToolCall {
            tool: step.tool.clone(),
            arguments: prepared_args,
        };

        // Notify hooks
        self.hooks.before_step(step, 0, context);

        let start = Instant::now();

        // Execute with retry and timeout
        let result = self
            .execute_step_with_retry_timeout(
                &tool_call,
                step,
                step_timeout,
                &step_retry_config,
                context,
            )
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        let step_result = match result {
            Ok((tool_result, retry_attempts)) => {
                // Store output in context
                if let Some(data) = &tool_result.data {
                    context.set_output(step.name.clone(), data.clone());
                }

                let is_success = tool_result.is_success();
                StepResult {
                    step_name: step.name.clone(),
                    success: is_success,
                    output: tool_result.data,
                    error: if is_success { None } else { tool_result.error },
                    duration_ms,
                    retry_attempts,
                }
            }
            Err(e) => {
                self.hooks.on_error(&e, context);

                StepResult {
                    step_name: step.name.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms,
                    retry_attempts: 0,
                }
            }
        };

        // Notify hooks
        self.hooks.after_step(step, 0, &step_result, context);

        if step_result.success {
            Ok(step_result)
        } else {
            // Return the step result even on failure for continue_on_error support
            Err(SkillError::Execution(
                step_result.error.clone().unwrap_or_default(),
            ))
        }
    }
}

impl<T: Transport, H: ExecutionHooks> DefaultSkillExecutor<T, H> {
    /// Internal method to execute all steps (used within skill timeout).
    async fn execute_steps(
        &self,
        skill: &Skill,
        context: &mut ExecutionContext,
        config: &ExecutionConfig,
    ) -> Result<SkillResult, SkillError> {
        let mut step_results: Vec<(String, ToolResult)> = Vec::new();

        for (index, step) in skill.steps.iter().enumerate() {
            // Determine timeout for this step
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
            let prepared_args = self.prepare_arguments(&step.arguments, context)?;

            let tool_call = ToolCall {
                tool: step.tool.clone(),
                arguments: prepared_args,
            };

            // Notify hooks
            self.hooks.before_step(step, index, context);

            let start = Instant::now();

            // Execute with retry and timeout
            let step_result = self
                .execute_step_with_retry_timeout(
                    &tool_call,
                    step,
                    step_timeout,
                    &step_retry_config,
                    context,
                )
                .await;

            let duration_ms = start.elapsed().as_millis() as u64;

            match step_result {
                Ok((tool_result, retry_attempts)) => {
                    // Create StepResult for hooks
                    let sr = StepResult {
                        step_name: step.name.clone(),
                        success: true,
                        output: tool_result.data.clone(),
                        error: None,
                        duration_ms,
                        retry_attempts,
                    };
                    self.hooks.after_step(step, index, &sr, context);

                    step_results.push((step.name.clone(), tool_result.clone()));

                    // Add result to context for use in subsequent steps
                    context.set_output(
                        step.name.clone(),
                        tool_result.data.clone().unwrap_or(Value::Null),
                    );

                    // If this is the last step, use its result as output
                    if step_results.len() == skill.steps.len() {
                        return Ok(SkillResult {
                            success: true,
                            step_results,
                            output: tool_result.data,
                            error: None,
                        });
                    }
                }
                Err(e) => {
                    // Create StepResult for hooks
                    let sr = StepResult::failure(&step.name, e.to_string(), duration_ms);
                    self.hooks.after_step(step, index, &sr, context);
                    self.hooks.on_error(&e, context);

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock transport for testing
    struct MockTransport {
        responses: HashMap<String, ToolResult>,
    }

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

    #[tokio::test]
    async fn test_default_executor_basic() {
        let transport = MockTransport::new().with_response(
            "tool1",
            ToolResult::success(serde_json::json!({"result": 1})),
        );

        let executor = DefaultSkillExecutor::new(transport);

        let skill = Skill::new("test", "Test skill").with_step(SkillStep {
            name: "step1".to_string(),
            tool: "tool1".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

        let mut context = ExecutionContext::new();
        let result = executor.execute(&skill, &mut context).await.unwrap();

        assert!(result.success);
        assert_eq!(result.step_results.len(), 1);
    }

    #[tokio::test]
    async fn test_default_executor_with_hooks() {
        struct CountingHooks {
            before_skill_count: Arc<AtomicUsize>,
            after_skill_count: Arc<AtomicUsize>,
            before_step_count: Arc<AtomicUsize>,
            after_step_count: Arc<AtomicUsize>,
        }

        impl ExecutionHooks for CountingHooks {
            fn before_skill(&self, _skill: &Skill, _context: &ExecutionContext) {
                self.before_skill_count.fetch_add(1, Ordering::SeqCst);
            }

            fn after_skill(
                &self,
                _skill: &Skill,
                _result: &SkillResult,
                _context: &ExecutionContext,
            ) {
                self.after_skill_count.fetch_add(1, Ordering::SeqCst);
            }

            fn before_step(
                &self,
                _step: &SkillStep,
                _step_index: usize,
                _context: &ExecutionContext,
            ) {
                self.before_step_count.fetch_add(1, Ordering::SeqCst);
            }

            fn after_step(
                &self,
                _step: &SkillStep,
                _step_index: usize,
                _result: &StepResult,
                _context: &ExecutionContext,
            ) {
                self.after_step_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        let before_skill = Arc::new(AtomicUsize::new(0));
        let after_skill = Arc::new(AtomicUsize::new(0));
        let before_step = Arc::new(AtomicUsize::new(0));
        let after_step = Arc::new(AtomicUsize::new(0));

        let hooks = CountingHooks {
            before_skill_count: before_skill.clone(),
            after_skill_count: after_skill.clone(),
            before_step_count: before_step.clone(),
            after_step_count: after_step.clone(),
        };

        let transport = MockTransport::new()
            .with_response("tool1", ToolResult::success(serde_json::json!({})))
            .with_response("tool2", ToolResult::success(serde_json::json!({})));

        let executor = DefaultSkillExecutor::with_hooks(transport, hooks);

        let skill = Skill::new("test", "Test skill")
            .with_step(SkillStep {
                name: "step1".to_string(),
                tool: "tool1".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            })
            .with_step(SkillStep {
                name: "step2".to_string(),
                tool: "tool2".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            });

        let mut context = ExecutionContext::new();
        let result = executor.execute(&skill, &mut context).await.unwrap();

        assert!(result.success);
        assert_eq!(before_skill.load(Ordering::SeqCst), 1);
        assert_eq!(after_skill.load(Ordering::SeqCst), 1);
        assert_eq!(before_step.load(Ordering::SeqCst), 2);
        assert_eq!(after_step.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_default_executor_context_propagation() {
        let transport = MockTransport::new()
            .with_response(
                "step1_tool",
                ToolResult::success(serde_json::json!({"value": 42})),
            )
            .with_response(
                "step2_tool",
                ToolResult::success(serde_json::json!({"doubled": 84})),
            );

        let executor = DefaultSkillExecutor::new(transport);

        let skill = Skill::new("test", "Test skill")
            .with_step(SkillStep {
                name: "step1".to_string(),
                tool: "step1_tool".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            })
            .with_step(SkillStep {
                name: "step2".to_string(),
                tool: "step2_tool".to_string(),
                arguments: serde_json::json!({"input": "{{step1}}"}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            });

        let mut context = ExecutionContext::new();
        let result = executor.execute(&skill, &mut context).await.unwrap();

        assert!(result.success);

        // Check that step1 output was stored in context
        assert!(context.get_output("step1").is_some());
        assert!(context.get_output("step2").is_some());
    }

    #[tokio::test]
    async fn test_default_executor_continue_on_error() {
        let transport = MockTransport::new()
            // step1 will fail (no response configured)
            .with_response(
                "step2_tool",
                ToolResult::success(serde_json::json!({"ok": true})),
            );

        let executor = DefaultSkillExecutor::new(transport);

        let skill = Skill::new("test", "Test skill")
            .with_step(SkillStep {
                name: "step1".to_string(),
                tool: "step1_tool".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: true, // Should continue even if this fails
                timeout_secs: None,
                max_retries: Some(0),
            })
            .with_step(SkillStep {
                name: "step2".to_string(),
                tool: "step2_tool".to_string(),
                arguments: serde_json::json!({}),
                continue_on_error: false,
                timeout_secs: None,
                max_retries: None,
            });

        let config = ExecutionConfig::new().with_retry(crate::RetryConfig::no_retries());
        let mut context = ExecutionContext::new().with_config(config);

        let result = executor.execute(&skill, &mut context).await.unwrap();

        assert!(result.success);
        assert_eq!(result.step_results.len(), 2);

        // First step should have failed
        let (_, step1_result) = &result.step_results[0];
        assert!(!step1_result.is_success());

        // Second step should have succeeded
        let (_, step2_result) = &result.step_results[1];
        assert!(step2_result.is_success());
    }

    #[tokio::test]
    async fn test_default_executor_from_arcs() {
        let transport = Arc::new(
            MockTransport::new().with_response("tool", ToolResult::success(serde_json::json!({}))),
        );
        let hooks = Arc::new(NoOpHooks);

        let executor = DefaultSkillExecutor::from_arcs(transport.clone(), hooks.clone());

        let skill = Skill::new("test", "Test").with_step(SkillStep {
            name: "s".to_string(),
            tool: "tool".to_string(),
            arguments: serde_json::json!({}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

        let mut context = ExecutionContext::new();
        let result = executor.execute(&skill, &mut context).await.unwrap();

        assert!(result.success);
    }
}
