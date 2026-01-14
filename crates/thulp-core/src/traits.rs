//! Core traits for thulp.

use crate::{Result, ToolCall, ToolDefinition, ToolResult};
use async_trait::async_trait;
use serde_json::Value;

/// Trait for executable tools.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition.
    fn definition(&self) -> &ToolDefinition;

    /// Execute the tool with the given arguments.
    async fn execute(&self, args: Value) -> Result<ToolResult>;

    /// Get the tool name.
    fn name(&self) -> &str {
        &self.definition().name
    }

    /// Validate arguments before execution.
    fn validate(&self, args: &Value) -> Result<()> {
        self.definition().validate_args(args)
    }
}

/// Trait for communication transports (MCP, HTTP, etc.).
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect to the transport.
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the transport.
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected.
    fn is_connected(&self) -> bool;

    /// List available tools.
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>>;

    /// Execute a tool call.
    async fn call(&self, call: &ToolCall) -> Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parameter;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Mock tool for testing
    struct MockTool {
        definition: ToolDefinition,
        execute_result: ToolResult,
    }

    impl MockTool {
        fn new(name: &str, result: ToolResult) -> Self {
            Self {
                definition: ToolDefinition::new(name),
                execute_result: result,
            }
        }
    }

    #[async_trait]
    impl Tool for MockTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        async fn execute(&self, _args: Value) -> Result<ToolResult> {
            Ok(self.execute_result.clone())
        }
    }

    // Mock transport for testing
    struct MockTransport {
        connected: Arc<AtomicBool>,
        tools: Vec<ToolDefinition>,
    }

    impl MockTransport {
        fn new(tools: Vec<ToolDefinition>) -> Self {
            Self {
                connected: Arc::new(AtomicBool::new(false)),
                tools,
            }
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn connect(&mut self) -> Result<()> {
            self.connected.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.connected.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
            Ok(self.tools.clone())
        }

        async fn call(&self, call: &ToolCall) -> Result<ToolResult> {
            Ok(ToolResult::success(json!({
                "tool": call.tool,
                "called": true
            })))
        }
    }

    #[tokio::test]
    async fn tool_trait_execute() {
        let tool = MockTool::new("test", ToolResult::success(json!({"result": "ok"})));

        let result = tool.execute(json!({})).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.data.unwrap()["result"], "ok");
    }

    #[tokio::test]
    async fn tool_trait_name() {
        let tool = MockTool::new("my_tool", ToolResult::success(json!(null)));
        assert_eq!(tool.name(), "my_tool");
    }

    #[tokio::test]
    async fn tool_trait_validate() {
        let mut tool = MockTool::new("test", ToolResult::success(json!(null)));
        tool.definition = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("name"))
            .build();

        // Valid args
        assert!(tool.validate(&json!({"name": "value"})).is_ok());

        // Missing required
        assert!(tool.validate(&json!({})).is_err());
    }

    #[tokio::test]
    async fn transport_trait_connect_disconnect() {
        let mut transport = MockTransport::new(vec![]);

        assert!(!transport.is_connected());

        transport.connect().await.unwrap();
        assert!(transport.is_connected());

        transport.disconnect().await.unwrap();
        assert!(!transport.is_connected());
    }

    #[tokio::test]
    async fn transport_trait_list_tools() {
        let tools = vec![ToolDefinition::new("tool1"), ToolDefinition::new("tool2")];
        let transport = MockTransport::new(tools.clone());

        let listed = transport.list_tools().await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "tool1");
        assert_eq!(listed[1].name, "tool2");
    }

    #[tokio::test]
    async fn transport_trait_call() {
        let transport = MockTransport::new(vec![]);

        let call = ToolCall::new("test_tool");
        let result = transport.call(&call).await.unwrap();

        assert!(result.is_success());
        assert_eq!(result.data.unwrap()["tool"], "test_tool");
    }
}
