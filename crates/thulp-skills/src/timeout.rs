//! Timeout utilities for skill execution.
//!
//! This module provides timeout wrappers for async operations.

use std::future::Future;
use std::time::Duration;

/// Errors that can occur during timeout-wrapped execution.
#[derive(Debug, thiserror::Error)]
pub enum TimeoutError<E> {
    /// The operation timed out.
    #[error("operation timed out after {duration:?}: {context}")]
    Timeout {
        /// Duration that elapsed before timeout.
        duration: Duration,
        /// Context describing what timed out.
        context: String,
    },

    /// The operation completed but returned an error.
    #[error("execution error: {0}")]
    ExecutionError(#[source] E),
}

impl<E> TimeoutError<E> {
    /// Check if this is a timeout error.
    pub fn is_timeout(&self) -> bool {
        matches!(self, TimeoutError::Timeout { .. })
    }

    /// Check if this is an execution error.
    pub fn is_execution_error(&self) -> bool {
        matches!(self, TimeoutError::ExecutionError(_))
    }

    /// Get the duration if this is a timeout error.
    pub fn timeout_duration(&self) -> Option<Duration> {
        match self {
            TimeoutError::Timeout { duration, .. } => Some(*duration),
            _ => None,
        }
    }
}

/// Execute a future with a timeout.
///
/// # Arguments
///
/// * `duration` - Maximum time to wait for the future to complete.
/// * `context` - Description of the operation for error messages.
/// * `future` - The async operation to execute.
///
/// # Returns
///
/// Returns `Ok(T)` if the future completes within the timeout,
/// `Err(TimeoutError::Timeout)` if the timeout expires, or
/// `Err(TimeoutError::ExecutionError)` if the future returns an error.
///
/// # Examples
///
/// ```ignore
/// use std::time::Duration;
/// use thulp_skills::timeout::with_timeout;
///
/// async fn example() {
///     let result = with_timeout(
///         Duration::from_secs(5),
///         "fetch data",
///         async { Ok::<_, std::io::Error>("data") }
///     ).await;
///     
///     assert!(result.is_ok());
/// }
/// ```
pub async fn with_timeout<F, T, E>(
    duration: Duration,
    context: impl Into<String>,
    future: F,
) -> Result<T, TimeoutError<E>>
where
    F: Future<Output = Result<T, E>>,
{
    let context = context.into();

    match tokio::time::timeout(duration, future).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(TimeoutError::ExecutionError(e)),
        Err(_elapsed) => Err(TimeoutError::Timeout { duration, context }),
    }
}

/// Execute an infallible future with a timeout.
///
/// Similar to `with_timeout` but for futures that don't return a Result.
///
/// # Returns
///
/// Returns `Some(T)` if the future completes within the timeout,
/// or `None` if the timeout expires.
pub async fn with_timeout_infallible<F, T>(duration: Duration, future: F) -> Option<T>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[tokio::test]
    async fn test_timeout_success() {
        let result: Result<&str, TimeoutError<io::Error>> =
            with_timeout(Duration::from_secs(1), "test operation", async {
                Ok("success")
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_timeout_fires() {
        let result: Result<(), TimeoutError<io::Error>> =
            with_timeout(Duration::from_millis(50), "slow operation", async {
                tokio::time::sleep(Duration::from_secs(10)).await;
                Ok(())
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_timeout());
        assert!(err.timeout_duration().is_some());
        assert!(err.to_string().contains("slow operation"));
    }

    #[tokio::test]
    async fn test_timeout_execution_error() {
        let result: Result<(), TimeoutError<io::Error>> =
            with_timeout(Duration::from_secs(1), "failing operation", async {
                Err(io::Error::new(io::ErrorKind::NotFound, "not found"))
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.is_execution_error());
        assert!(err.timeout_duration().is_none());
    }

    #[tokio::test]
    async fn test_timeout_infallible_success() {
        let result = with_timeout_infallible(Duration::from_secs(1), async { 42 }).await;

        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_timeout_infallible_timeout() {
        let result = with_timeout_infallible(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            42
        })
        .await;

        assert_eq!(result, None);
    }
}
