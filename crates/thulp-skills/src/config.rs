//! Configuration types for skill execution.
//!
//! This module provides configuration for timeouts and retries during skill execution.

use std::time::Duration;

/// Configuration for execution timeouts.
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Maximum time for entire skill execution.
    pub skill_timeout: Duration,

    /// Maximum time for a single step.
    pub step_timeout: Duration,

    /// Maximum time for a single tool call.
    pub tool_timeout: Duration,

    /// Action to take on timeout.
    pub timeout_action: TimeoutAction,
}

/// Action to take when a timeout occurs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TimeoutAction {
    /// Fail immediately with an error.
    #[default]
    Fail,

    /// Skip the timed-out step and continue execution.
    Skip,

    /// Return partial results collected so far.
    Partial,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            skill_timeout: Duration::from_secs(300),  // 5 minutes
            step_timeout: Duration::from_secs(60),    // 1 minute
            tool_timeout: Duration::from_secs(30),    // 30 seconds
            timeout_action: TimeoutAction::Fail,
        }
    }
}

impl TimeoutConfig {
    /// Create a new timeout configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the skill timeout.
    pub fn with_skill_timeout(mut self, timeout: Duration) -> Self {
        self.skill_timeout = timeout;
        self
    }

    /// Set the step timeout.
    pub fn with_step_timeout(mut self, timeout: Duration) -> Self {
        self.step_timeout = timeout;
        self
    }

    /// Set the tool timeout.
    pub fn with_tool_timeout(mut self, timeout: Duration) -> Self {
        self.tool_timeout = timeout;
        self
    }

    /// Set the timeout action.
    pub fn with_timeout_action(mut self, action: TimeoutAction) -> Self {
        self.timeout_action = action;
        self
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: usize,

    /// Initial delay between retries.
    pub initial_delay: Duration,

    /// Maximum delay between retries (caps exponential backoff).
    pub max_delay: Duration,

    /// Backoff strategy to use.
    pub backoff: BackoffStrategy,

    /// Which error types are retryable.
    pub retryable_errors: Vec<RetryableError>,
}

/// Strategy for calculating delay between retries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BackoffStrategy {
    /// Fixed delay between retries.
    Fixed,

    /// Exponential backoff (delay * 2^attempt).
    Exponential,

    /// Exponential backoff with random jitter.
    #[default]
    ExponentialJitter,
}

/// Types of errors that can be retried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryableError {
    /// Network or connection errors.
    Network,

    /// Rate limit errors (HTTP 429).
    RateLimit,

    /// Timeout errors.
    Timeout,

    /// Server errors (HTTP 5xx).
    ServerError,

    /// All errors are retryable.
    All,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff: BackoffStrategy::ExponentialJitter,
            retryable_errors: vec![
                RetryableError::Network,
                RetryableError::RateLimit,
                RetryableError::Timeout,
            ],
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Disable retries.
    pub fn no_retries() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Set the maximum number of retries.
    pub fn with_max_retries(mut self, max: usize) -> Self {
        self.max_retries = max;
        self
    }

    /// Set the initial delay.
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff strategy.
    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff = strategy;
        self
    }

    /// Set which errors are retryable.
    pub fn with_retryable_errors(mut self, errors: Vec<RetryableError>) -> Self {
        self.retryable_errors = errors;
        self
    }

    /// Make all errors retryable.
    pub fn retry_all_errors(mut self) -> Self {
        self.retryable_errors = vec![RetryableError::All];
        self
    }
}

/// Combined execution configuration.
#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    /// Timeout configuration.
    pub timeout: TimeoutConfig,

    /// Retry configuration.
    pub retry: RetryConfig,
}

impl ExecutionConfig {
    /// Create a new execution configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set timeout configuration.
    pub fn with_timeout(mut self, config: TimeoutConfig) -> Self {
        self.timeout = config;
        self
    }

    /// Set retry configuration.
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry = config;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_config_defaults() {
        let config = TimeoutConfig::default();
        assert_eq!(config.skill_timeout, Duration::from_secs(300));
        assert_eq!(config.step_timeout, Duration::from_secs(60));
        assert_eq!(config.tool_timeout, Duration::from_secs(30));
        assert_eq!(config.timeout_action, TimeoutAction::Fail);
    }

    #[test]
    fn test_timeout_config_builder() {
        let config = TimeoutConfig::new()
            .with_skill_timeout(Duration::from_secs(120))
            .with_step_timeout(Duration::from_secs(30))
            .with_timeout_action(TimeoutAction::Skip);

        assert_eq!(config.skill_timeout, Duration::from_secs(120));
        assert_eq!(config.step_timeout, Duration::from_secs(30));
        assert_eq!(config.timeout_action, TimeoutAction::Skip);
    }

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.backoff, BackoffStrategy::ExponentialJitter);
        assert!(config.retryable_errors.contains(&RetryableError::Network));
    }

    #[test]
    fn test_retry_config_no_retries() {
        let config = RetryConfig::no_retries();
        assert_eq!(config.max_retries, 0);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_retries(5)
            .with_initial_delay(Duration::from_millis(200))
            .with_backoff(BackoffStrategy::Fixed)
            .retry_all_errors();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(200));
        assert_eq!(config.backoff, BackoffStrategy::Fixed);
        assert!(config.retryable_errors.contains(&RetryableError::All));
    }

    #[test]
    fn test_execution_config() {
        let config = ExecutionConfig::new()
            .with_timeout(TimeoutConfig::new().with_skill_timeout(Duration::from_secs(60)))
            .with_retry(RetryConfig::no_retries());

        assert_eq!(config.timeout.skill_timeout, Duration::from_secs(60));
        assert_eq!(config.retry.max_retries, 0);
    }
}
