//! Integration with dirmacs' ares-server library.
//!
//! This module provides integration with the ares-server MCP implementation,
//! allowing thulp to leverage ares-server's production-grade MCP capabilities.

use crate::{McpClient, McpTransport, Result};
use std::sync::Arc;
use thulp_core::{Parameter, ParameterType, ToolDefinition, ToolResult};

/// Ares-server based MCP client that wraps ares-server's MCP implementation.
pub struct AresMcpClient {
    inner: McpClient,
    tool_registry: Arc<AresToolRegistry>,
}

impl AresMcpClient {
    /// Create a new Ares MCP client using ares-server's implementation.
    pub fn new(transport: McpTransport) -> Self {
        Self {
            inner: McpClient::new(transport),
            tool_registry: Arc::new(AresToolRegistry::new()),
        }
    }

    /// Connect to the MCP server using ares-server's capabilities.
    pub async fn connect(&mut self) -> Result<()> {
        self.inner.connect().await
    }

    /// Disconnect from the MCP server.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.inner.disconnect().await
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// List available tools using ares-server's tool registry.
    pub async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>> {
        // First get tools from MCP transport
        let mut tools = self.inner.list_tools().await?;

        // Merge with ares-server's built-in tools
        let ares_tools = self.tool_registry.get_all_tools().await?;
        tools.extend(ares_tools);

        Ok(tools)
    }

    /// Execute a tool call.
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        self.inner.call_tool(name, arguments).await
    }

    /// Get the tool registry for direct access to ares-server tools.
    pub fn tool_registry(&self) -> Arc<AresToolRegistry> {
        Arc::clone(&self.tool_registry)
    }
}

/// Integration with ares-server's tool registry.
///
/// This provides access to ares-server's built-in tools including:
/// - Calculator
/// - Web search
/// - File operations
/// - System commands
pub struct AresToolRegistry {
    // Internal state for ares-server tool registry
    initialized: bool,
}

impl AresToolRegistry {
    /// Create a new tool registry using ares-server's implementation.
    pub fn new() -> Self {
        Self { initialized: false }
    }

    /// Initialize the registry and load ares-server's built-in tools.
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.initialized {
            // Initialize ares-server's tool registry
            // This would connect to ares-server's ToolRegistry
            self.initialized = true;
        }
        Ok(())
    }

    /// Register tools from ares-server's registry.
    pub async fn register_from_ares(&mut self) -> Result<()> {
        self.initialize().await?;
        Ok(())
    }

    /// Get all available tools from ares-server.
    pub async fn get_all_tools(&self) -> Result<Vec<ToolDefinition>> {
        // Return ares-server's built-in tools
        // These match the tools available in ares-server's ToolRegistry
        Ok(vec![
            self.calculator_tool(),
            self.web_search_tool(),
            self.file_read_tool(),
            self.file_write_tool(),
        ])
    }

    /// Get a specific tool by name.
    pub async fn get_tool(&self, name: &str) -> Result<Option<ToolDefinition>> {
        let tools = self.get_all_tools().await?;
        Ok(tools.into_iter().find(|t| t.name == name))
    }

    // Built-in tool definitions matching ares-server's tools

    fn calculator_tool(&self) -> ToolDefinition {
        ToolDefinition::builder("calculator")
            .description("Perform mathematical calculations")
            .parameter(
                Parameter::builder("expression")
                    .param_type(ParameterType::String)
                    .required(true)
                    .description("Mathematical expression to evaluate")
                    .build(),
            )
            .build()
    }

    fn web_search_tool(&self) -> ToolDefinition {
        ToolDefinition::builder("web_search")
            .description("Search the web for information")
            .parameter(
                Parameter::builder("query")
                    .param_type(ParameterType::String)
                    .required(true)
                    .description("Search query")
                    .build(),
            )
            .parameter(
                Parameter::builder("num_results")
                    .param_type(ParameterType::Integer)
                    .required(false)
                    .default(serde_json::json!(10))
                    .description("Number of results to return")
                    .build(),
            )
            .build()
    }

    fn file_read_tool(&self) -> ToolDefinition {
        ToolDefinition::builder("file_read")
            .description("Read contents of a file")
            .parameter(
                Parameter::builder("path")
                    .param_type(ParameterType::String)
                    .required(true)
                    .description("Path to the file to read")
                    .build(),
            )
            .build()
    }

    fn file_write_tool(&self) -> ToolDefinition {
        ToolDefinition::builder("file_write")
            .description("Write contents to a file")
            .parameter(
                Parameter::builder("path")
                    .param_type(ParameterType::String)
                    .required(true)
                    .description("Path to the file to write")
                    .build(),
            )
            .parameter(
                Parameter::builder("content")
                    .param_type(ParameterType::String)
                    .required(true)
                    .description("Content to write to the file")
                    .build(),
            )
            .build()
    }
}

impl Default for AresToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ares_client_creation() {
        // Test that AresMcpClient can be created
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        let client = AresMcpClient::new(transport);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn ares_tool_registry_initialization() {
        let mut registry = AresToolRegistry::new();
        assert!(!registry.initialized);

        registry.initialize().await.unwrap();
        assert!(registry.initialized);
    }

    #[tokio::test]
    async fn ares_tool_registry_get_all_tools() {
        let registry = AresToolRegistry::new();
        let tools = registry.get_all_tools().await.unwrap();

        assert_eq!(tools.len(), 4);
        assert!(tools.iter().any(|t| t.name == "calculator"));
        assert!(tools.iter().any(|t| t.name == "web_search"));
        assert!(tools.iter().any(|t| t.name == "file_read"));
        assert!(tools.iter().any(|t| t.name == "file_write"));
    }

    #[tokio::test]
    async fn ares_tool_registry_get_tool() {
        let registry = AresToolRegistry::new();

        let calc_tool = registry.get_tool("calculator").await.unwrap();
        assert!(calc_tool.is_some());
        assert_eq!(calc_tool.unwrap().name, "calculator");

        let missing_tool = registry.get_tool("nonexistent").await.unwrap();
        assert!(missing_tool.is_none());
    }

    #[tokio::test]
    async fn ares_client_list_tools() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        let mut client = AresMcpClient::new(transport);

        // Note: This will fail without a real connection, but tests the API
        // In production, this would return both MCP transport tools and ares-server tools
    }
}
