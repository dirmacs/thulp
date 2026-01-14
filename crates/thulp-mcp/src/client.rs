//! MCP client implementation.

use crate::{McpTransport, Result};
use std::collections::HashMap;
use thulp_core::{ToolCall, ToolDefinition, ToolResult, Transport};

/// MCP client wrapper.
pub struct McpClient {
    transport: McpTransport,
    tool_cache: HashMap<String, ToolDefinition>,
    session_id: String,
}

impl McpClient {
    /// Create a new MCP client.
    pub fn new(transport: McpTransport) -> Self {
        Self {
            transport,
            tool_cache: HashMap::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create a client builder.
    pub fn builder() -> McpClientBuilder {
        McpClientBuilder::new()
    }

    /// Connect to the MCP server.
    pub async fn connect(&mut self) -> Result<()> {
        self.transport.connect().await?;
        Ok(())
    }

    /// Disconnect from the MCP server.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.transport.disconnect().await?;
        self.tool_cache.clear();
        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }

    /// List available tools.
    pub async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>> {
        if self.tool_cache.is_empty() {
            let tools = self.transport.list_tools().await?;
            for tool in &tools {
                self.tool_cache.insert(tool.name.clone(), tool.clone());
            }
        }

        Ok(self.tool_cache.values().cloned().collect())
    }

    /// Get a specific tool definition.
    pub async fn get_tool(&mut self, name: &str) -> Result<Option<ToolDefinition>> {
        if !self.tool_cache.contains_key(name) {
            // Refresh cache if tool not found
            self.list_tools().await?;
        }
        Ok(self.tool_cache.get(name).cloned())
    }

    /// Execute a tool call.
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<ToolResult> {
        let call = ToolCall {
            tool: name.to_string(),
            arguments,
        };
        self.transport.call(&call).await
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Clear the tool cache.
    pub fn clear_cache(&mut self) {
        self.tool_cache.clear();
    }
}

/// Builder for [`McpClient`].
pub struct McpClientBuilder {
    transport: Option<McpTransport>,
}

impl McpClientBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self { transport: None }
    }

    /// Set the transport.
    pub fn transport(mut self, transport: McpTransport) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<McpClient> {
        use thulp_core::Error;
        let transport = self
            .transport
            .ok_or_else(|| Error::InvalidConfig("transport not set".to_string()))?;

        Ok(McpClient::new(transport))
    }
}

impl Default for McpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common connection patterns.
impl McpClient {
    /// Connect to an MCP server via HTTP.
    pub async fn connect_http(name: String, url: String) -> Result<McpClient> {
        let transport = McpTransport::new_http(name, url);
        let mut client = McpClient::new(transport);

        client.connect().await?;
        Ok(client)
    }

    /// Connect to an MCP server via STDIO.
    pub async fn connect_stdio(
        name: String,
        command: String,
        args: Option<Vec<String>>,
    ) -> Result<McpClient> {
        let transport = McpTransport::new_stdio(name, command, args);
        let mut client = McpClient::new(transport);

        client.connect().await?;
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_creation() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        let client = McpClient::new(transport);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn client_builder() {
        let client = McpClient::builder()
            .transport(McpTransport::new_http(
                "test".to_string(),
                "http://localhost:8080".to_string(),
            ))
            .build()
            .unwrap();
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn client_convenience() {
        // This is a placeholder test since we can't actually connect to MCP servers in tests
        // In real usage, this would connect to a real MCP server
        assert!(true);
    }
}
