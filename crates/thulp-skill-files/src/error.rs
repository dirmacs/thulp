//! Error types for skill file operations.

use thiserror::Error;

/// Result type alias for skill file operations.
pub type Result<T> = std::result::Result<T, SkillFileError>;

/// Errors that can occur when working with skill files.
#[derive(Debug, Error)]
pub enum SkillFileError {
    /// IO error when reading/writing files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error in frontmatter.
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// General parsing error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Invalid path provided.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Shell command execution error.
    #[error("Command execution error: {0}")]
    CommandExecution(String),

    /// Variable not found during substitution.
    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    /// Skill not found in registry.
    #[error("Skill not found: {0}")]
    SkillNotFound(String),

    /// Tool not allowed for skill execution.
    #[error("Tool not allowed: {0}")]
    ToolNotAllowed(String),

    /// Skill requires user approval before execution.
    #[error("Approval required for skill: {0}")]
    ApprovalRequired(String),

    /// Regex compilation error.
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}
