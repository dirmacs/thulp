//! YAML frontmatter types for SKILL.md files.
//!
//! This module defines the structure of the YAML frontmatter that appears
//! at the top of SKILL.md files, between `---` delimiters.

use serde::{Deserialize, Serialize};

/// YAML frontmatter configuration for a skill file.
///
/// All fields are optional - skills can be defined with just content
/// and no frontmatter at all.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SkillFrontmatter {
    /// Display name for the skill (defaults to directory name).
    #[serde(default)]
    pub name: Option<String>,

    /// What the skill does and when to use it.
    #[serde(default)]
    pub description: Option<String>,

    /// Hint shown during autocomplete for expected arguments.
    #[serde(default)]
    pub argument_hint: Option<String>,

    /// Prevent agent from automatically invoking this skill.
    #[serde(default)]
    pub disable_model_invocation: bool,

    /// Whether users can invoke this skill directly (default: true).
    #[serde(default = "default_true")]
    pub user_invocable: bool,

    /// Tools the skill is allowed to use.
    /// If None, all tools are allowed.
    #[serde(default)]
    pub allowed_tools: Option<Vec<String>>,

    /// Model to use when this skill is active.
    #[serde(default)]
    pub model: Option<String>,

    /// Execution context: "fork" for subagent, "inline" for current context.
    #[serde(default)]
    pub context: Option<SkillContext>,

    /// Which subagent type to use when context is "fork".
    #[serde(default)]
    pub agent: Option<String>,

    /// Hooks scoped to this skill's lifecycle.
    #[serde(default)]
    pub hooks: Option<SkillHooks>,

    // === DOT DOT Marketplace Extensions ===
    /// Skill version (semver).
    #[serde(default)]
    pub version: Option<String>,

    /// Author identifier.
    #[serde(default)]
    pub author: Option<String>,

    /// Pricing model for marketplace.
    #[serde(default)]
    pub price: Option<PriceModel>,

    /// Whether this skill requires user approval before execution.
    #[serde(default)]
    pub requires_approval: bool,

    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Minimum Thulp version required.
    #[serde(default)]
    pub min_thulp_version: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SkillFrontmatter {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            argument_hint: None,
            disable_model_invocation: false,
            user_invocable: true, // Default to true
            allowed_tools: None,
            model: None,
            context: None,
            agent: None,
            hooks: None,
            version: None,
            author: None,
            price: None,
            requires_approval: false,
            tags: Vec::new(),
            min_thulp_version: None,
        }
    }
}

/// Execution context for skills.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SkillContext {
    /// Run in current conversation context.
    #[default]
    Inline,
    /// Run in isolated subagent context.
    Fork,
}

/// Pricing model for marketplace skills.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PriceModel {
    /// Free skill (represented as "free" string).
    Free,
    /// Fixed price per call.
    PerCall(f64),
    /// Subscription-based pricing.
    Subscription {
        /// Monthly subscription cost.
        monthly: f64,
    },
    /// Custom pricing string.
    Custom(String),
}

// Note: Can't use #[derive(Default)] with #[serde(untagged)] - serde parsing breaks
#[allow(clippy::derivable_impls)]
impl Default for PriceModel {
    fn default() -> Self {
        Self::Free
    }
}

/// Lifecycle hooks for skills.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SkillHooks {
    /// Run before skill starts.
    #[serde(default)]
    pub pre_execute: Option<String>,

    /// Run after skill completes.
    #[serde(default)]
    pub post_execute: Option<String>,

    /// Run on error.
    #[serde(default)]
    pub on_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_frontmatter() {
        let fm = SkillFrontmatter::default();
        assert!(fm.name.is_none());
        assert!(fm.user_invocable); // Should default to true
        assert!(!fm.disable_model_invocation);
        assert!(!fm.requires_approval);
    }

    #[test]
    fn test_parse_minimal_frontmatter() {
        let yaml = r#"
name: test-skill
description: A test skill
"#;
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.name, Some("test-skill".to_string()));
        assert_eq!(fm.description, Some("A test skill".to_string()));
    }

    #[test]
    fn test_parse_full_frontmatter() {
        let yaml = r#"
name: advanced-skill
description: An advanced skill with all options
argument-hint: <file-path>
disable-model-invocation: true
user-invocable: false
allowed-tools:
  - Read
  - Write
  - Bash
model: claude-sonnet-4-20250514
context: fork
agent: code-reviewer
requires-approval: true
tags:
  - code
  - review
version: 1.2.3
author: dirmacs
"#;
        let fm: SkillFrontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.name, Some("advanced-skill".to_string()));
        assert!(fm.disable_model_invocation);
        assert!(!fm.user_invocable);
        assert_eq!(
            fm.allowed_tools,
            Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Bash".to_string()
            ])
        );
        assert_eq!(fm.context, Some(SkillContext::Fork));
        assert_eq!(fm.agent, Some("code-reviewer".to_string()));
        assert!(fm.requires_approval);
        assert_eq!(fm.tags, vec!["code".to_string(), "review".to_string()]);
    }

    #[test]
    fn test_skill_context_serialization() {
        assert_eq!(
            serde_yaml::to_string(&SkillContext::Fork).unwrap().trim(),
            "fork"
        );
        assert_eq!(
            serde_yaml::to_string(&SkillContext::Inline).unwrap().trim(),
            "inline"
        );
    }
}
