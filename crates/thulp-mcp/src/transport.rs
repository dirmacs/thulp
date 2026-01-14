//! MCP transport implementations using rs-utcp.

use crate::Result;
use async_trait::async_trait;
use rs_utcp::providers::base::Provider;
use rs_utcp::providers::mcp::McpProvider;
use rs_utcp::transports::mcp::McpTransport as RsUtcpMcpTransport;
use rs_utcp::transports::ClientTransport;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use thulp_core::{Error, ToolCall, ToolDefinition, ToolResult, Transport as CoreTransport};

/// Wrapper around rs-utcp's MCP transport
pub struct McpTransport {
    /// The underlying rs-utcp transport
    inner: RsUtcpMcpTransport,
    /// The MCP provider
    provider: Arc<dyn Provider>,
    /// Connection status
    connected: bool,
}

impl McpTransport {
    /// Create a new MCP transport for HTTP connection
    pub fn new_http(name: String, url: String) -> Self {
        let provider = Arc::new(McpProvider::new(name, url, None));
        let inner = RsUtcpMcpTransport::new();

        Self {
            inner,
            provider,
            connected: false,
        }
    }

    /// Create a new MCP transport for STDIO connection
    pub fn new_stdio(name: String, command: String, args: Option<Vec<String>>) -> Self {
        let provider = Arc::new(McpProvider::new_stdio(name, command, args, None));
        let inner = RsUtcpMcpTransport::new();

        Self {
            inner,
            provider,
            connected: false,
        }
    }

    /// Create a new MCP transport with default configuration
    pub fn new() -> Self {
        Self::new_http("default".to_string(), "http://localhost:8080".to_string())
    }
}

impl Default for McpTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CoreTransport for McpTransport {
    async fn connect(&mut self) -> Result<()> {
        // Register the provider with the transport
        let _tools = self
            .inner
            .register_tool_provider(&*self.provider)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("Failed to register provider: {}", e)))?;

        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.inner
            .deregister_tool_provider(&*self.provider)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("Failed to deregister provider: {}", e)))?;
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        if !self.connected {
            return Err(Error::ExecutionFailed("not connected".to_string()));
        }

        // Use rs-utcp's register_tool_provider which calls tools/list internally
        let tools = self
            .inner
            .register_tool_provider(&*self.provider)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("Failed to list tools: {}", e)))?;

        // Convert rs-utcp Tool to ToolDefinition
        let mut definitions = Vec::new();
        for tool in tools {
            // Convert the ToolInputOutputSchema to JSON for parsing
            let inputs_json = serde_json::to_value(&tool.inputs).map_err(|e| {
                Error::ExecutionFailed(format!("Failed to serialize inputs: {}", e))
            })?;

            // Extract parameters from tool.inputs
            let parameters =
                ToolDefinition::parse_mcp_input_schema(&inputs_json).unwrap_or_default();

            definitions.push(ToolDefinition {
                name: tool.name,
                description: tool.description,
                parameters,
            });
        }

        Ok(definitions)
    }

    async fn call(&self, call: &ToolCall) -> Result<ToolResult> {
        if !self.connected {
            return Err(Error::ExecutionFailed("not connected".to_string()));
        }

        // Convert arguments to the format expected by rs-utcp
        let args: HashMap<String, Value> = match &call.arguments {
            Value::Object(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            _ => HashMap::new(),
        };

        // Call the tool through the transport
        let result = self
            .inner
            .call_tool(&call.tool, args, &*self.provider)
            .await
            .map_err(|e| Error::ExecutionFailed(format!("Tool call failed: {}", e)))?;

        Ok(ToolResult::success(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn transport_new_http() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        assert!(!transport.is_connected());
    }

    #[test]
    fn transport_new_stdio() {
        let transport = McpTransport::new_stdio(
            "test".to_string(),
            "test-cmd".to_string(),
            Some(vec!["--arg1".to_string()]),
        );
        assert!(!transport.is_connected());
    }

    #[test]
    fn transport_new_default() {
        let transport = McpTransport::new();
        assert!(!transport.is_connected());
    }

    #[test]
    fn transport_default_impl() {
        let transport1 = McpTransport::default();
        let transport2 = McpTransport::new();
        assert!(!transport1.is_connected());
        assert!(!transport2.is_connected());
    }

    #[tokio::test]
    async fn transport_connect_disconnect() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:9999".to_string());

        // Start with disconnected
        assert!(!transport.is_connected());

        // Note: We can't actually connect in tests without a real MCP server
        // This test verifies the structure and state management
        assert_eq!(transport.is_connected(), false);
    }

    #[test]
    fn test_argument_conversion() {
        // Test argument conversion for tool calls
        let call = ToolCall::builder("test_tool")
            .arg_str("string_param", "value")
            .arg_int("int_param", 42)
            .arg_bool("bool_param", true)
            .build();

        // Verify arguments are in expected format
        assert!(call.arguments.is_object());
        assert_eq!(call.arguments["string_param"], "value");
        assert_eq!(call.arguments["int_param"], 42);
        assert_eq!(call.arguments["bool_param"], true);
    }

    #[test]
    fn test_argument_conversion_nested() {
        // Test with nested object arguments
        let call = ToolCall::builder("test_tool")
            .arg("nested", json!({"key": "value", "number": 123}))
            .build();

        assert!(call.arguments.is_object());
        assert_eq!(call.arguments["nested"]["key"], "value");
        assert_eq!(call.arguments["nested"]["number"], 123);
    }

    #[test]
    fn test_argument_conversion_array() {
        // Test with array arguments
        let call = ToolCall::builder("test_tool")
            .arg("items", json!([1, 2, 3, 4, 5]))
            .build();

        assert!(call.arguments.is_object());
        assert_eq!(call.arguments["items"], json!([1, 2, 3, 4, 5]));
    }

    // Edge case tests
    #[tokio::test]
    async fn test_list_tools_when_disconnected() {
        let transport = McpTransport::new();

        // Should fail when not connected
        let result = transport.list_tools().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not connected"));
    }

    #[tokio::test]
    async fn test_call_when_disconnected() {
        let transport = McpTransport::new();
        let call = ToolCall::new("test_tool");

        // Should fail when not connected
        let result = transport.call(&call).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not connected"));
    }

    #[test]
    fn test_empty_arguments() {
        let call = ToolCall::new("test_tool");
        assert!(call.arguments.is_object());
        assert_eq!(call.arguments.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_special_characters_in_tool_name() {
        let call = ToolCall::new("test-tool_v2.0");
        assert_eq!(call.tool, "test-tool_v2.0");
    }

    #[test]
    fn test_unicode_in_arguments() {
        let call = ToolCall::builder("test_tool")
            .arg_str("message", "Hello 世界 🌍")
            .build();

        assert_eq!(call.arguments["message"], "Hello 世界 🌍");
    }

    #[test]
    fn test_large_argument_values() {
        // Test with large string
        let large_string = "x".repeat(10000);
        let call = ToolCall::builder("test_tool")
            .arg_str("data", &large_string)
            .build();

        assert_eq!(call.arguments["data"].as_str().unwrap().len(), 10000);
    }

    #[test]
    fn test_null_arguments() {
        let call = ToolCall::builder("test_tool")
            .arg("null_param", json!(null))
            .build();

        assert!(call.arguments["null_param"].is_null());
    }

    #[test]
    fn test_mixed_type_arguments() {
        let call = ToolCall::builder("test_tool")
            .arg_str("string", "value")
            .arg_int("int", 42)
            .arg_bool("bool", true)
            .arg("null", json!(null))
            .arg("array", json!([1, 2, 3]))
            .arg("object", json!({"key": "value"}))
            .build();

        assert_eq!(call.arguments["string"], "value");
        assert_eq!(call.arguments["int"], 42);
        assert_eq!(call.arguments["bool"], true);
        assert!(call.arguments["null"].is_null());
        assert!(call.arguments["array"].is_array());
        assert!(call.arguments["object"].is_object());
    }

    #[test]
    fn test_stdio_transport_creation() {
        let transport = McpTransport::new_stdio(
            "echo-server".to_string(),
            "npx".to_string(),
            Some(vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-echo".to_string(),
            ]),
        );

        assert!(!transport.is_connected());
    }

    #[test]
    fn test_http_transport_with_https() {
        let transport = McpTransport::new_http(
            "secure".to_string(),
            "https://api.example.com/mcp".to_string(),
        );

        assert!(!transport.is_connected());
    }

    #[test]
    fn test_transport_creation_with_empty_name() {
        let transport = McpTransport::new_http("".to_string(), "http://localhost:8080".to_string());
        assert!(!transport.is_connected());
    }

    #[test]
    fn test_deeply_nested_arguments() {
        let nested = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": "deep"
                        }
                    }
                }
            }
        });

        let call = ToolCall::builder("test_tool")
            .arg("nested", nested.clone())
            .build();

        assert_eq!(call.arguments["nested"], nested);
        assert_eq!(
            call.arguments["nested"]["level1"]["level2"]["level3"]["level4"]["level5"],
            "deep"
        );
    }

    #[test]
    fn test_argument_with_numbers() {
        let call = ToolCall::builder("test_tool")
            .arg_int("positive", 42)
            .arg_int("negative", -42)
            .arg_int("zero", 0)
            .arg("float", json!(3.14159))
            .arg("scientific", json!(1.5e10))
            .build();

        assert_eq!(call.arguments["positive"], 42);
        assert_eq!(call.arguments["negative"], -42);
        assert_eq!(call.arguments["zero"], 0);
        assert_eq!(call.arguments["float"], 3.14159);
        assert_eq!(call.arguments["scientific"], 1.5e10);
    }

    #[test]
    fn test_argument_with_special_json_values() {
        let call = ToolCall::builder("test_tool")
            .arg("empty_string", json!(""))
            .arg("empty_array", json!([]))
            .arg("empty_object", json!({}))
            .arg("boolean_true", json!(true))
            .arg("boolean_false", json!(false))
            .build();

        assert_eq!(call.arguments["empty_string"], "");
        assert_eq!(call.arguments["empty_array"], json!([]));
        assert_eq!(call.arguments["empty_object"], json!({}));
        assert_eq!(call.arguments["boolean_true"], true);
        assert_eq!(call.arguments["boolean_false"], false);
    }
}
