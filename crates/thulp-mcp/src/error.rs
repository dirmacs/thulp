//! Error types for thulp-mcp.

use thulp_core::Error;

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
