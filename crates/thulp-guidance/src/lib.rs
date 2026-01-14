//! # thulp-guidance
//!
//! Prompt guidance and template system for thulp.
//!
//! This crate provides utilities for creating, managing, and rendering
//! prompt templates for AI agent interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for guidance operations
pub type Result<T> = std::result::Result<T, GuidanceError>;

/// Errors that can occur in the guidance system
#[derive(Debug, thiserror::Error)]
pub enum GuidanceError {
    #[error("Template error: {0}")]
    Template(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// A prompt template with variable substitution support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template name
    pub name: String,

    /// Template content with {{variable}} placeholders
    pub content: String,

    /// Default values for variables
    #[serde(default)]
    pub defaults: HashMap<String, String>,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            defaults: HashMap::new(),
        }
    }

    /// Set a default value for a variable
    pub fn with_default(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.defaults.insert(key.into(), value.into());
        self
    }

    /// Render the template with the given variables
    pub fn render(&self, variables: &HashMap<String, String>) -> Result<String> {
        let mut result = self.content.clone();

        // Merge defaults with provided variables
        let mut all_vars = self.defaults.clone();
        all_vars.extend(variables.clone());

        // Replace all {{variable}} placeholders
        for (key, value) in all_vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, &value);
        }

        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
            return Err(GuidanceError::Template(
                "Template contains unresolved placeholders".to_string(),
            ));
        }

        Ok(result)
    }
}

/// A collection of prompt templates
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    templates: HashMap<String, PromptTemplate>,
}

impl TemplateRegistry {
    /// Create a new template registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a template
    pub fn register(&mut self, template: PromptTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// Render a template by name with variables
    pub fn render(&self, name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let template = self
            .get(name)
            .ok_or_else(|| GuidanceError::VariableNotFound(name.to_string()))?;
        template.render(variables)
    }

    /// List all template names
    pub fn list(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = PromptTemplate::new("test", "Hello {{name}}!");
        assert_eq!(template.name, "test");
        assert_eq!(template.content, "Hello {{name}}!");
    }

    #[test]
    fn test_template_render() {
        let template = PromptTemplate::new("test", "Hello {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_template_defaults() {
        let template =
            PromptTemplate::new("test", "Hello {{name}}!").with_default("name", "Default");

        let result = template.render(&HashMap::new()).unwrap();
        assert_eq!(result, "Hello Default!");
    }

    #[test]
    fn test_template_override_default() {
        let template =
            PromptTemplate::new("test", "Hello {{name}}!").with_default("name", "Default");

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Custom".to_string());

        let result = template.render(&vars).unwrap();
        assert_eq!(result, "Hello Custom!");
    }

    #[test]
    fn test_registry() {
        let mut registry = TemplateRegistry::new();

        let template = PromptTemplate::new("greeting", "Hello {{name}}!");
        registry.register(template);

        assert!(registry.get("greeting").is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_registry_render() {
        let mut registry = TemplateRegistry::new();

        let template = PromptTemplate::new("greeting", "Hello {{name}}!");
        registry.register(template);

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());

        let result = registry.render("greeting", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }
}
