//! # thulp-registry
//!
//! Tool registry implementation for thulp.
//!
//! This crate provides a registry for managing tool definitions and configurations,
//! including loading from configuration files, caching, and discovery.

use std::collections::HashMap;
use std::sync::Arc;
use thulp_core::{Error, Result, ToolDefinition};
use tokio::sync::RwLock;

/// Tool registry for managing and discovering tools.
///
/// The registry supports:
/// - Dynamic tool registration and unregistration
/// - Tool discovery by name or tag
/// - Caching for performance
/// - Thread-safe concurrent access
pub struct ToolRegistry {
    /// Map of tool name to tool definition
    tools: Arc<RwLock<HashMap<String, ToolDefinition>>>,

    /// Map of tags to tool names for discovery
    tags: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            tags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a tool in the registry.
    pub async fn register(&self, tool: ToolDefinition) -> Result<()> {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name.clone(), tool);
        Ok(())
    }

    /// Register multiple tools at once.
    pub async fn register_many(&self, tools: Vec<ToolDefinition>) -> Result<()> {
        let mut registry = self.tools.write().await;
        for tool in tools {
            registry.insert(tool.name.clone(), tool);
        }
        Ok(())
    }

    /// Unregister a tool from the registry.
    pub async fn unregister(&self, name: &str) -> Result<Option<ToolDefinition>> {
        let mut tools = self.tools.write().await;
        Ok(tools.remove(name))
    }

    /// Get a tool definition by name.
    pub async fn get(&self, name: &str) -> Result<Option<ToolDefinition>> {
        let tools = self.tools.read().await;
        Ok(tools.get(name).cloned())
    }

    /// List all registered tools.
    pub async fn list(&self) -> Result<Vec<ToolDefinition>> {
        let tools = self.tools.read().await;
        Ok(tools.values().cloned().collect())
    }

    /// Get the number of registered tools.
    pub async fn count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Clear all tools from the registry.
    pub async fn clear(&self) {
        let mut tools = self.tools.write().await;
        let mut tags = self.tags.write().await;
        tools.clear();
        tags.clear();
    }

    /// Check if a tool is registered.
    pub async fn contains(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// Tag a tool for discovery.
    pub async fn tag(&self, tool_name: &str, tag: &str) -> Result<()> {
        let tools = self.tools.read().await;
        if !tools.contains_key(tool_name) {
            return Err(Error::InvalidConfig(format!(
                "Tool '{}' not found in registry",
                tool_name
            )));
        }
        drop(tools);

        let mut tags = self.tags.write().await;
        tags.entry(tag.to_string())
            .or_insert_with(Vec::new)
            .push(tool_name.to_string());
        Ok(())
    }

    /// Find tools by tag.
    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<ToolDefinition>> {
        let tags = self.tags.read().await;
        let tool_names = match tags.get(tag) {
            Some(names) => names.clone(),
            None => return Ok(Vec::new()),
        };
        drop(tags);

        let tools = self.tools.read().await;
        let mut results = Vec::new();
        for name in tool_names {
            if let Some(tool) = tools.get(&name) {
                results.push(tool.clone());
            }
        }
        Ok(results)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thulp_core::Parameter;

    fn create_test_tool(name: &str) -> ToolDefinition {
        ToolDefinition::builder(name)
            .description(format!("Test tool: {}", name))
            .parameter(Parameter::required_string("test_param"))
            .build()
    }

    #[tokio::test]
    async fn registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn register_and_get_tool() {
        let registry = ToolRegistry::new();
        let tool = create_test_tool("test_tool");

        registry.register(tool.clone()).await.unwrap();

        let retrieved = registry.get("test_tool").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test_tool");
    }

    #[tokio::test]
    async fn register_many_tools() {
        let registry = ToolRegistry::new();
        let tools = vec![
            create_test_tool("tool1"),
            create_test_tool("tool2"),
            create_test_tool("tool3"),
        ];

        registry.register_many(tools).await.unwrap();

        assert_eq!(registry.count().await, 3);
        assert!(registry.contains("tool1").await);
        assert!(registry.contains("tool2").await);
        assert!(registry.contains("tool3").await);
    }

    #[tokio::test]
    async fn unregister_tool() {
        let registry = ToolRegistry::new();
        let tool = create_test_tool("test_tool");

        registry.register(tool).await.unwrap();
        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister("test_tool").await.unwrap();
        assert!(removed.is_some());
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn list_tools() {
        let registry = ToolRegistry::new();
        let tools = vec![create_test_tool("tool1"), create_test_tool("tool2")];

        registry.register_many(tools).await.unwrap();

        let listed = registry.list().await.unwrap();
        assert_eq!(listed.len(), 2);
    }

    #[tokio::test]
    async fn clear_registry() {
        let registry = ToolRegistry::new();
        let tools = vec![create_test_tool("tool1"), create_test_tool("tool2")];

        registry.register_many(tools).await.unwrap();
        assert_eq!(registry.count().await, 2);

        registry.clear().await;
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn tag_and_find_tools() {
        let registry = ToolRegistry::new();
        let tool1 = create_test_tool("tool1");
        let tool2 = create_test_tool("tool2");
        let tool3 = create_test_tool("tool3");

        registry.register(tool1).await.unwrap();
        registry.register(tool2).await.unwrap();
        registry.register(tool3).await.unwrap();

        registry.tag("tool1", "filesystem").await.unwrap();
        registry.tag("tool2", "filesystem").await.unwrap();
        registry.tag("tool3", "network").await.unwrap();

        let filesystem_tools = registry.find_by_tag("filesystem").await.unwrap();
        assert_eq!(filesystem_tools.len(), 2);

        let network_tools = registry.find_by_tag("network").await.unwrap();
        assert_eq!(network_tools.len(), 1);
    }

    #[tokio::test]
    async fn tag_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let result = registry.tag("nonexistent", "test").await;
        assert!(result.is_err());
    }
}
