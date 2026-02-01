//! Retry utilities for skill execution.
//!
//! This module provides retry logic with configurable backoff strategies.

use crate::config::{BackoffStrategy, RetryConfig, RetryableError};
use std::future::Future;
use std::time::Duration;

/// Errors that can occur during retried execution.
#[derive(Debug, thiserror::Error)]
pub enum RetryError<E> {
    /// All retry attempts were exhausted.
    #[error("exhausted {attempts} retry attempts: {last_error}")]
    Exhausted {
        /// Number of attempts made.
        attempts: usize,
        /// The last error that occurred.
        #[source]
        last_error: E,
    },

    /// The error was not retryable.
    #[error("non-retryable error: {0}")]
    NotRetryable(#[source] E),
}

impl<E> RetryError<E> {
    /// Check if retries were exhausted.
    pub fn is_exhausted(&self) -> bool {
        matches!(self, RetryError::Exhausted { .. })
    }

    /// Get the number of attempts made if exhausted.
    pub fn attempts(&self) -> Option<usize> {
        match self {
            RetryError::Exhausted { attempts, .. } => Some(*attempts),
            _ => None,
        }
    }

    /// Get the underlying error.
    pub fn into_inner(self) -> E {
        match self {
            RetryError::Exhausted { last_error, .. } => last_error,
            RetryError::NotRetryable(e) => e,
        }
    }
}

/// Execute an operation with retries.
///
/// # Arguments
///
/// * `config` - Retry configuration.
/// * `operation_name` - Name of the operation for logging.
/// * `is_retryable` - Function to determine if an error is retryable.
/// * `operation` - The async operation to execute.
///
/// # Returns
///
/// Returns `Ok(T)` if the operation succeeds within the retry limit,
/// or `Err(RetryError)` if all retries are exhausted or the error is not retryable.
///
/// # Examples
///
/// ```ignore
/// use thulp_skills::retry::with_retry;
/// use thulp_skills::config::RetryConfig;
///
/// async fn example() {
///     let result = with_retry(
///         &RetryConfig::default(),
///         "fetch data",
///         |e: &std::io::Error| e.kind() == std::io::ErrorKind::TimedOut,
///         || async { Ok::<_, std::io::Error>("data") }
///     ).await;
/// }
/// ```
pub async fn with_retry<F, Fut, T, E, R>(
    config: &RetryConfig,
    operation_name: &str,
    is_retryable: R,
    mut operation: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    R: Fn(&E) -> bool,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;

                // Check if we should retry
                if attempt > config.max_retries {
                    return Err(RetryError::Exhausted {
                        attempts: attempt,
                        last_error: e,
                    });
                }

                if !is_retryable(&e) {
                    return Err(RetryError::NotRetryable(e));
                }

                let delay = calculate_delay(config, attempt);
                tracing::warn!(
                    attempt = attempt,
                    max_retries = config.max_retries,
                    operation = operation_name,
                    delay_ms = delay.as_millis() as u64,
                    error = %e,
                    "Retrying operation after error"
                );

                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Execute an operation with retries using standard retryable error detection.
///
/// This is a convenience wrapper around `with_retry` that uses the
/// `RetryConfig::retryable_errors` to determine if an error is retryable.
pub async fn with_retry_default<F, Fut, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    operation: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display + AsRef<dyn std::error::Error + 'static>,
{
    with_retry(config, operation_name, |e| is_error_retryable(e, config), operation).await
}

/// Calculate the delay before the next retry attempt.
pub fn calculate_delay(config: &RetryConfig, attempt: usize) -> Duration {
    let base_delay = match config.backoff {
        BackoffStrategy::Fixed => config.initial_delay,
        BackoffStrategy::Exponential => {
            let multiplier = 2u32.saturating_pow(attempt.saturating_sub(1) as u32);
            config.initial_delay.saturating_mul(multiplier)
        }
        BackoffStrategy::ExponentialJitter => {
            let multiplier = 2u32.saturating_pow(attempt.saturating_sub(1) as u32);
            let base = config.initial_delay.saturating_mul(multiplier);
            let jitter_range = base.as_millis() as u64 / 2;
            let jitter = if jitter_range > 0 {
                fastrand::u64(0..jitter_range)
            } else {
                0
            };
            base + Duration::from_millis(jitter)
        }
    };

    std::cmp::min(base_delay, config.max_delay)
}

/// Check if an error is retryable based on the configuration.
pub fn is_error_retryable<E>(error: &E, config: &RetryConfig) -> bool
where
    E: std::fmt::Display,
{
    if config.retryable_errors.contains(&RetryableError::All) {
        return true;
    }

    let msg = error.to_string().to_lowercase();

    // Check for rate limit errors
    if config.retryable_errors.contains(&RetryableError::RateLimit)
        && (msg.contains("rate limit") || msg.contains("429") || msg.contains("too many requests"))
    {
        return true;
    }

    // Check for timeout errors
    if config.retryable_errors.contains(&RetryableError::Timeout)
        && (msg.contains("timeout") || msg.contains("timed out"))
    {
        return true;
    }

    // Check for server errors
    if config.retryable_errors.contains(&RetryableError::ServerError)
        && (msg.contains("500")
            || msg.contains("502")
            || msg.contains("503")
            || msg.contains("504")
            || msg.contains("internal server error")
            || msg.contains("bad gateway")
            || msg.contains("service unavailable"))
    {
        return true;
    }

    // Check for network errors
    if config.retryable_errors.contains(&RetryableError::Network)
        && (msg.contains("connection")
            || msg.contains("network")
            || msg.contains("dns")
            || msg.contains("resolve")
            || msg.contains("unreachable"))
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_calculate_delay_fixed() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff: BackoffStrategy::Fixed,
            ..Default::default()
        };

        assert_eq!(calculate_delay(&config, 1), Duration::from_millis(100));
        assert_eq!(calculate_delay(&config, 2), Duration::from_millis(100));
        assert_eq!(calculate_delay(&config, 5), Duration::from_millis(100));
    }

    #[test]
    fn test_calculate_delay_exponential() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff: BackoffStrategy::Exponential,
            ..Default::default()
        };

        assert_eq!(calculate_delay(&config, 1), Duration::from_millis(100));
        assert_eq!(calculate_delay(&config, 2), Duration::from_millis(200));
        assert_eq!(calculate_delay(&config, 3), Duration::from_millis(400));
        assert_eq!(calculate_delay(&config, 4), Duration::from_millis(800));
    }

    #[test]
    fn test_calculate_delay_max_cap() {
        let config = RetryConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff: BackoffStrategy::Exponential,
            ..Default::default()
        };

        // Would be 8 seconds without cap
        assert_eq!(calculate_delay(&config, 4), Duration::from_secs(5));
    }

    #[test]
    fn test_is_error_retryable_rate_limit() {
        let config = RetryConfig::default();

        assert!(is_error_retryable(&"Rate limit exceeded", &config));
        assert!(is_error_retryable(&"HTTP 429 Too Many Requests", &config));
    }

    #[test]
    fn test_is_error_retryable_timeout() {
        let config = RetryConfig::default();

        assert!(is_error_retryable(&"Connection timed out", &config));
        assert!(is_error_retryable(&"Request timeout", &config));
    }

    #[test]
    fn test_is_error_retryable_network() {
        let config = RetryConfig::default();

        assert!(is_error_retryable(&"Connection refused", &config));
        assert!(is_error_retryable(&"Network unreachable", &config));
        assert!(is_error_retryable(&"DNS resolution failed", &config));
    }

    #[test]
    fn test_is_error_retryable_not_configured() {
        let config = RetryConfig {
            retryable_errors: vec![RetryableError::Timeout],
            ..Default::default()
        };

        assert!(!is_error_retryable(&"Rate limit exceeded", &config));
        assert!(is_error_retryable(&"Request timed out", &config));
    }

    #[test]
    fn test_is_error_retryable_all() {
        let config = RetryConfig::new().retry_all_errors();

        assert!(is_error_retryable(&"Any error at all", &config));
        assert!(is_error_retryable(&"Random failure", &config));
    }

    #[tokio::test]
    async fn test_retry_success_first_try() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = with_retry(
            &RetryConfig::default(),
            "test",
            |_: &String| true,
            || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>("success")
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let result = with_retry(&config, "test", |_: &String| true, || {
            let attempts = Arc::clone(&attempts_clone);
            async move {
                let n = attempts.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err("transient error".to_string())
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let result: Result<(), RetryError<String>> =
            with_retry(&config, "test", |_: &String| true, || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err("persistent error".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_exhausted());
        assert_eq!(err.attempts(), Some(3)); // 1 initial + 2 retries = 3 attempts
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_not_retryable() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let config = RetryConfig::default();

        let result: Result<(), RetryError<String>> =
            with_retry(&config, "test", |_: &String| false, || {
                let attempts = Arc::clone(&attempts_clone);
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err("non-retryable error".to_string())
                }
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.is_exhausted());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
