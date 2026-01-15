//! Error types for thulp-core.

use thiserror::Error;

/// Result type alias using the [`Error`](enum@Error) enum.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in thulp-core.
#[derive(Debug, Error)]
pub enum Error {
    /// A required parameter was not provided.
    #[error("missing required parameter: {0}")]
    MissingParameter(String),

    /// A parameter has an invalid type.
    #[error("invalid parameter type for '{name}': expected {expected}, got {actual}")]
    InvalidParameterType {
        /// Parameter name.
        name: String,
        /// Expected type.
        expected: String,
        /// Actual type received.
        actual: String,
    },

    /// Tool not found.
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// Tool execution failed.
    #[error("tool execution failed: {0}")]
    ExecutionFailed(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_missing_parameter() {
        let err = Error::MissingParameter("username".to_string());
        assert_eq!(err.to_string(), "missing required parameter: username");
    }

    #[test]
    fn error_display_invalid_type() {
        let err = Error::InvalidParameterType {
            name: "count".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid parameter type for 'count': expected integer, got string"
        );
    }

    #[test]
    fn error_display_tool_not_found() {
        let err = Error::ToolNotFound("unknown_tool".to_string());
        assert_eq!(err.to_string(), "tool not found: unknown_tool");
    }

    #[test]
    fn error_from_serde_json() {
        let json_err: serde_json::Error = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }
}
