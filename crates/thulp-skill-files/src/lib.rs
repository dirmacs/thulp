//! # thulp-skill-files
//!
//! SKILL.md file parsing and loading for the Thulp execution context platform.
//!
//! This crate provides functionality for:
//! - Parsing SKILL.md files with YAML frontmatter
//! - Preprocessing skill content (arguments, commands, variables)
//! - Loading skills from directories with scope-based priority
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use thulp_skill_files::{SkillFile, SkillLoader, SkillLoaderConfig, SkillPreprocessor};
//! use std::collections::HashMap;
//!
//! // Parse a single skill file
//! let skill = SkillFile::parse("path/to/SKILL.md")?;
//! println!("Skill: {}", skill.effective_name());
//!
//! // Load all skills from configured directories
//! let config = SkillLoaderConfig::default();
//! let loader = SkillLoader::new(config);
//! let skills = loader.load_all()?;
//!
//! // Preprocess skill content
//! let pp = SkillPreprocessor::new();
//! let context = HashMap::new();
//! let processed = pp.preprocess(&skill.content, "my args", &context)?;
//! ```
//!
//! ## SKILL.md Format
//!
//! Skills are defined in SKILL.md files with optional YAML frontmatter:
//!
//! ```markdown
//! ---
//! name: my-skill
//! description: Does something useful
//! allowed-tools:
//!   - Read
//!   - Write
//! ---
//! # Instructions
//!
//! When invoked with $ARGUMENTS, do the following...
//! ```
//!
//! ## Frontmatter Options
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `name` | string | Display name (defaults to directory name) |
//! | `description` | string | What the skill does |
//! | `argument-hint` | string | Hint for expected arguments |
//! | `disable-model-invocation` | bool | Prevent automatic invocation |
//! | `user-invocable` | bool | Allow user to invoke (default: true) |
//! | `allowed-tools` | list | Restrict tools the skill can use |
//! | `context` | inline/fork | Execution context |
//! | `requires-approval` | bool | Require user approval |
//!
//! ## Scope Priority
//!
//! Skills are loaded with priority: Enterprise > Personal > Project.
//! Plugin skills are namespaced and don't conflict.

pub mod error;
pub mod frontmatter;
pub mod loader;
pub mod parser;
pub mod preprocessor;

// Re-export main types
pub use error::{Result, SkillFileError};
pub use frontmatter::{PriceModel, SkillContext, SkillFrontmatter, SkillHooks};
pub use loader::{LoadedSkill, SkillLoader, SkillLoaderConfig, SkillScope};
pub use parser::{SkillFile, SupportingFile, SupportingFileType};
pub use preprocessor::SkillPreprocessor;
