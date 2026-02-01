//! Execution lifecycle hooks.
//!
//! This module provides the [`ExecutionHooks`] trait for observing and reacting
//! to skill execution lifecycle events.
//!
//! # Example
//!
//! ```ignore
//! use thulp_skills::{ExecutionHooks, Skill, SkillStep, SkillResult, StepResult, ExecutionContext};
//!
//! struct LoggingHooks;
//!
//! impl ExecutionHooks for LoggingHooks {
//!     fn before_skill(&self, skill: &Skill, context: &ExecutionContext) {
//!         println!("Starting skill: {}", skill.name);
//!     }
//!
//!     fn after_skill(&self, skill: &Skill, result: &SkillResult, context: &ExecutionContext) {
//!         println!("Completed skill: {} (success={})", skill.name, result.success);
//!     }
//! }
//! ```

use crate::{ExecutionContext, Skill, SkillError, SkillResult, SkillStep, StepResult};

/// Lifecycle hooks for skill execution.
///
/// Implement this trait to observe and react to execution lifecycle events.
/// All methods have default no-op implementations, so you only need to
/// implement the ones you care about.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow use with async executors.
///
/// # Example
///
/// ```ignore
/// use thulp_skills::{ExecutionHooks, Skill, ExecutionContext, SkillResult};
///
/// struct MetricsHooks {
///     // metrics collector
/// }
///
/// impl ExecutionHooks for MetricsHooks {
///     fn before_skill(&self, skill: &Skill, _context: &ExecutionContext) {
///         // Record skill execution start
///     }
///
///     fn after_skill(&self, skill: &Skill, result: &SkillResult, _context: &ExecutionContext) {
///         // Record skill execution end, success/failure
///     }
/// }
/// ```
pub trait ExecutionHooks: Send + Sync {
    /// Called before a skill begins execution.
    ///
    /// # Arguments
    ///
    /// * `skill` - The skill about to be executed
    /// * `context` - The execution context with inputs
    fn before_skill(&self, _skill: &Skill, _context: &ExecutionContext) {}

    /// Called after a skill completes execution (success or failure).
    ///
    /// # Arguments
    ///
    /// * `skill` - The skill that was executed
    /// * `result` - The result of the skill execution
    /// * `context` - The execution context with outputs
    fn after_skill(&self, _skill: &Skill, _result: &SkillResult, _context: &ExecutionContext) {}

    /// Called before a step begins execution.
    ///
    /// # Arguments
    ///
    /// * `step` - The step about to be executed
    /// * `step_index` - Zero-based index of the step in the skill
    /// * `context` - The current execution context
    fn before_step(&self, _step: &SkillStep, _step_index: usize, _context: &ExecutionContext) {}

    /// Called after a step completes execution (success or failure).
    ///
    /// # Arguments
    ///
    /// * `step` - The step that was executed
    /// * `step_index` - Zero-based index of the step in the skill
    /// * `result` - The result of the step execution
    /// * `context` - The current execution context
    fn after_step(
        &self,
        _step: &SkillStep,
        _step_index: usize,
        _result: &StepResult,
        _context: &ExecutionContext,
    ) {
    }

    /// Called when a step is about to be retried.
    ///
    /// # Arguments
    ///
    /// * `step` - The step being retried
    /// * `attempt` - The attempt number (1-based, so first retry is attempt 2)
    /// * `error` - The error that caused the retry
    /// * `context` - The current execution context
    fn on_retry(
        &self,
        _step: &SkillStep,
        _attempt: usize,
        _error: &str,
        _context: &ExecutionContext,
    ) {
    }

    /// Called when an error occurs during execution.
    ///
    /// This is called for errors that are not retried, or after all retries
    /// are exhausted.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that occurred
    /// * `context` - The current execution context
    fn on_error(&self, _error: &SkillError, _context: &ExecutionContext) {}

    /// Called when a step times out.
    ///
    /// # Arguments
    ///
    /// * `step` - The step that timed out
    /// * `duration_ms` - How long the step ran before timing out
    /// * `context` - The current execution context
    fn on_timeout(&self, _step: &SkillStep, _duration_ms: u64, _context: &ExecutionContext) {}
}

/// A no-op implementation of [`ExecutionHooks`].
///
/// This is the default hooks implementation that does nothing for all
/// lifecycle events. Use this when you don't need any hooks.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpHooks;

impl NoOpHooks {
    /// Create a new no-op hooks instance.
    pub fn new() -> Self {
        Self
    }
}

impl ExecutionHooks for NoOpHooks {}

/// A hooks implementation that logs execution events using tracing.
///
/// This provides observability into skill execution without requiring
/// custom hook implementations.
#[derive(Debug, Clone, Copy, Default)]
pub struct TracingHooks {
    /// Log level for debug messages
    include_debug: bool,
}

impl TracingHooks {
    /// Create a new tracing hooks instance.
    pub fn new() -> Self {
        Self {
            include_debug: false,
        }
    }

    /// Include debug-level messages.
    pub fn with_debug(mut self) -> Self {
        self.include_debug = true;
        self
    }
}

impl ExecutionHooks for TracingHooks {
    fn before_skill(&self, skill: &Skill, context: &ExecutionContext) {
        tracing::info!(
            skill_name = %skill.name,
            input_count = context.inputs().len(),
            step_count = skill.steps.len(),
            "Starting skill execution"
        );

        if self.include_debug {
            tracing::debug!(
                skill_name = %skill.name,
                inputs = ?context.inputs().keys().collect::<Vec<_>>(),
                "Skill inputs"
            );
        }
    }

    fn after_skill(&self, skill: &Skill, result: &SkillResult, context: &ExecutionContext) {
        if result.success {
            tracing::info!(
                skill_name = %skill.name,
                steps_completed = result.step_results.len(),
                output_count = context.outputs().len(),
                "Skill execution completed successfully"
            );
        } else {
            tracing::warn!(
                skill_name = %skill.name,
                steps_completed = result.step_results.len(),
                error = ?result.error,
                "Skill execution failed"
            );
        }
    }

    fn before_step(&self, step: &SkillStep, step_index: usize, _context: &ExecutionContext) {
        tracing::info!(
            step_name = %step.name,
            step_index = step_index,
            tool = %step.tool,
            "Starting step execution"
        );
    }

    fn after_step(
        &self,
        step: &SkillStep,
        step_index: usize,
        result: &StepResult,
        _context: &ExecutionContext,
    ) {
        if result.success {
            tracing::info!(
                step_name = %step.name,
                step_index = step_index,
                duration_ms = result.duration_ms,
                retry_attempts = result.retry_attempts,
                "Step completed successfully"
            );
        } else {
            tracing::warn!(
                step_name = %step.name,
                step_index = step_index,
                duration_ms = result.duration_ms,
                error = ?result.error,
                "Step failed"
            );
        }
    }

    fn on_retry(&self, step: &SkillStep, attempt: usize, error: &str, _context: &ExecutionContext) {
        tracing::warn!(
            step_name = %step.name,
            attempt = attempt,
            error = %error,
            "Retrying step after failure"
        );
    }

    fn on_error(&self, error: &SkillError, _context: &ExecutionContext) {
        tracing::error!(
            error = %error,
            "Skill execution error"
        );
    }

    fn on_timeout(&self, step: &SkillStep, duration_ms: u64, _context: &ExecutionContext) {
        tracing::warn!(
            step_name = %step.name,
            duration_ms = duration_ms,
            "Step timed out"
        );
    }
}

/// Compose multiple hooks implementations.
///
/// This allows combining multiple hooks (e.g., logging + metrics) into a single
/// hooks instance that calls all of them.
pub struct CompositeHooks {
    hooks: Vec<Box<dyn ExecutionHooks>>,
}

impl CompositeHooks {
    /// Create a new composite hooks instance.
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Add a hooks implementation.
    pub fn with<H: ExecutionHooks + 'static>(mut self, hooks: H) -> Self {
        self.hooks.push(Box::new(hooks));
        self
    }
}

impl Default for CompositeHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionHooks for CompositeHooks {
    fn before_skill(&self, skill: &Skill, context: &ExecutionContext) {
        for h in &self.hooks {
            h.before_skill(skill, context);
        }
    }

    fn after_skill(&self, skill: &Skill, result: &SkillResult, context: &ExecutionContext) {
        for h in &self.hooks {
            h.after_skill(skill, result, context);
        }
    }

    fn before_step(&self, step: &SkillStep, step_index: usize, context: &ExecutionContext) {
        for h in &self.hooks {
            h.before_step(step, step_index, context);
        }
    }

    fn after_step(
        &self,
        step: &SkillStep,
        step_index: usize,
        result: &StepResult,
        context: &ExecutionContext,
    ) {
        for h in &self.hooks {
            h.after_step(step, step_index, result, context);
        }
    }

    fn on_retry(&self, step: &SkillStep, attempt: usize, error: &str, context: &ExecutionContext) {
        for h in &self.hooks {
            h.on_retry(step, attempt, error, context);
        }
    }

    fn on_error(&self, error: &SkillError, context: &ExecutionContext) {
        for h in &self.hooks {
            h.on_error(error, context);
        }
    }

    fn on_timeout(&self, step: &SkillStep, duration_ms: u64, context: &ExecutionContext) {
        for h in &self.hooks {
            h.on_timeout(step, duration_ms, context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_no_op_hooks() {
        let hooks = NoOpHooks::new();
        let skill = Skill::new("test", "Test skill");
        let context = ExecutionContext::new();

        // All methods should be callable without panic
        hooks.before_skill(&skill, &context);
        hooks.after_skill(
            &skill,
            &SkillResult {
                success: true,
                step_results: vec![],
                output: None,
                error: None,
            },
            &context,
        );
    }

    #[test]
    fn test_tracing_hooks_creation() {
        let hooks = TracingHooks::new();
        assert!(!hooks.include_debug);

        let hooks_with_debug = TracingHooks::new().with_debug();
        assert!(hooks_with_debug.include_debug);
    }

    #[test]
    fn test_composite_hooks() {
        struct CountingHooks {
            before_count: Arc<AtomicUsize>,
            after_count: Arc<AtomicUsize>,
        }

        impl ExecutionHooks for CountingHooks {
            fn before_skill(&self, _skill: &Skill, _context: &ExecutionContext) {
                self.before_count.fetch_add(1, Ordering::SeqCst);
            }

            fn after_skill(
                &self,
                _skill: &Skill,
                _result: &SkillResult,
                _context: &ExecutionContext,
            ) {
                self.after_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        let before_count1 = Arc::new(AtomicUsize::new(0));
        let after_count1 = Arc::new(AtomicUsize::new(0));
        let before_count2 = Arc::new(AtomicUsize::new(0));
        let after_count2 = Arc::new(AtomicUsize::new(0));

        let hooks = CompositeHooks::new()
            .with(CountingHooks {
                before_count: before_count1.clone(),
                after_count: after_count1.clone(),
            })
            .with(CountingHooks {
                before_count: before_count2.clone(),
                after_count: after_count2.clone(),
            });

        let skill = Skill::new("test", "Test skill");
        let context = ExecutionContext::new();
        let result = SkillResult {
            success: true,
            step_results: vec![],
            output: None,
            error: None,
        };

        hooks.before_skill(&skill, &context);
        hooks.after_skill(&skill, &result, &context);

        assert_eq!(before_count1.load(Ordering::SeqCst), 1);
        assert_eq!(after_count1.load(Ordering::SeqCst), 1);
        assert_eq!(before_count2.load(Ordering::SeqCst), 1);
        assert_eq!(after_count2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_composite_hooks_default() {
        let hooks = CompositeHooks::default();
        let skill = Skill::new("test", "Test skill");
        let context = ExecutionContext::new();

        // Should not panic with empty hooks list
        hooks.before_skill(&skill, &context);
    }
}
