//! Real integration with dirmacs' `ares-server` library.
//!
//! Wraps `ares::ToolRegistry` and exposes its registered tools to
//! thulp via the [`thulp_core::ToolDefinition`] format. Also provides a
//! direct pass-through so callers can register or invoke ares tools.
//!
//! Unlike the earlier placeholder module, this one actually uses ares-server
//! types (`ares::ToolRegistry`, `ares::tools::calculator::Calculator`)
//! and calls `get_tool_definitions()` instead of hard-coding the list.
//!
//! # Feature gates
//!
//! - Default (`ares` feature): Calculator tool only.
//! - With `ares-search`: also registers [`WebSearch`] (requires
//!   `ares-server/search-tools`).

use crate::{McpClient, McpTransport, Result};
use ares::tools::calculator::Calculator;
#[cfg(feature = "ares-search")]
use ares::tools::search::WebSearch;
use ares::tools::registry::Tool as AresTool;
use ares::types::ToolDefinition as AresToolDefinition;
use ares::ToolRegistry as AresRegistryInner;
use std::sync::Arc;
use thulp_core::{Parameter, ParameterType, ToolDefinition};

/// Ares-server based MCP client wrapping the plain [`McpClient`] alongside
/// a real [`AresToolRegistry`].
///
/// Use this when you want the agent to see both the MCP transport's tools
/// and ares-server's built-in tool registry.
pub struct AresMcpClient {
    inner: McpClient,
    registry: Arc<AresToolRegistry>,
}

impl AresMcpClient {
    /// Create a new client with the default Calculator-only registry.
    pub fn new(transport: McpTransport) -> Self {
        Self {
            inner: McpClient::new(transport),
            registry: Arc::new(AresToolRegistry::with_default_tools()),
        }
    }

    /// Create a client with a caller-supplied registry.
    pub fn with_registry(transport: McpTransport, registry: Arc<AresToolRegistry>) -> Self {
        Self {
            inner: McpClient::new(transport),
            registry,
        }
    }

    /// Connect to the underlying MCP transport.
    pub async fn connect(&mut self) -> Result<()> {
        self.inner.connect().await
    }

    /// Disconnect from the underlying MCP transport.
    pub async fn disconnect(&mut self) -> Result<()> {
        self.inner.disconnect().await
    }

    /// Returns true once the MCP transport is connected.
    pub fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    /// List all tools — MCP transport tools merged with ares registry tools.
    ///
    /// Names from the MCP transport take precedence: if the remote server
    /// publishes `calculator` and the local registry also has `calculator`,
    /// the remote one wins (listed first).
    pub async fn list_tools(&mut self) -> Result<Vec<ToolDefinition>> {
        let mut tools = self.inner.list_tools().await?;
        let existing: std::collections::HashSet<String> =
            tools.iter().map(|t| t.name.clone()).collect();
        for ares_tool in self.registry.get_all_tools() {
            if !existing.contains(&ares_tool.name) {
                tools.push(ares_tool);
            }
        }
        Ok(tools)
    }

    /// Access the wrapped ares tool registry.
    pub fn registry(&self) -> Arc<AresToolRegistry> {
        Arc::clone(&self.registry)
    }
}

/// Wrapper around [`ares::ToolRegistry`] that converts tool
/// definitions to thulp's [`ToolDefinition`] format.
///
/// Hold this in an `Arc` if you need to share it across tasks.
pub struct AresToolRegistry {
    inner: AresRegistryInner,
}

impl AresToolRegistry {
    /// Create an empty registry with no tools registered.
    pub fn new() -> Self {
        Self {
            inner: AresRegistryInner::new(),
        }
    }

    /// Create a registry pre-populated with the default tools.
    ///
    /// Currently: `Calculator`. With the `ares-search` feature, also
    /// registers `WebSearch`.
    pub fn with_default_tools() -> Self {
        let mut reg = Self::new();
        reg.inner.register(Arc::new(Calculator));
        #[cfg(feature = "ares-search")]
        {
            reg.inner.register(Arc::new(WebSearch::new()));
        }
        reg
    }

    /// Register an additional ares-server-compatible tool.
    pub fn register(&mut self, tool: Arc<dyn AresTool>) {
        self.inner.register(tool);
    }

    /// Return all registered tools in thulp's [`ToolDefinition`] format.
    ///
    /// Lossy: ares-server stores parameters as raw JSON Schema; this
    /// conversion walks the schema's `properties` map and turns each entry
    /// into a thulp [`Parameter`]. Types not in [`ParameterType`] fall
    /// back to `String`.
    pub fn get_all_tools(&self) -> Vec<ToolDefinition> {
        self.inner
            .get_tool_definitions()
            .into_iter()
            .map(ares_to_thulp_definition)
            .collect()
    }

    /// Look up a specific tool by name.
    pub fn get_tool(&self, name: &str) -> Option<ToolDefinition> {
        self.inner
            .get_tool_definitions()
            .into_iter()
            .find(|t| t.name == name)
            .map(ares_to_thulp_definition)
    }

    /// Returns `true` if the registry has a tool with the given name.
    pub fn has_tool(&self, name: &str) -> bool {
        self.inner.has_tool(name)
    }

    /// Returns the number of registered tools.
    pub fn len(&self) -> usize {
        self.inner.get_tool_definitions().len()
    }

    /// Returns `true` if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for AresToolRegistry {
    fn default() -> Self {
        Self::with_default_tools()
    }
}

/// Convert an ares-server [`AresToolDefinition`] into thulp's
/// [`ToolDefinition`] by parsing the `parameters` JSON Schema.
fn ares_to_thulp_definition(def: AresToolDefinition) -> ToolDefinition {
    let mut builder = ToolDefinition::builder(def.name).description(def.description);

    // ares uses JSON Schema format: {"type": "object", "properties": {...}, "required": [...]}
    if let Some(properties) = def.parameters.get("properties").and_then(|v| v.as_object()) {
        let required: std::collections::HashSet<String> = def
            .parameters
            .get("required")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        for (name, schema) in properties {
            let description = schema
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let param_type = match schema.get("type").and_then(|v| v.as_str()) {
                Some("string") => ParameterType::String,
                Some("integer") => ParameterType::Integer,
                Some("number") => ParameterType::Number,
                Some("boolean") => ParameterType::Boolean,
                Some("array") => ParameterType::Array,
                Some("object") => ParameterType::Object,
                _ => ParameterType::String, // JSON Schema allows no type; fall back to String
            };

            let mut p = Parameter::builder(name.clone())
                .param_type(param_type)
                .required(required.contains(name))
                .description(description);

            if let Some(default) = schema.get("default") {
                p = p.default(default.clone());
            }

            builder = builder.parameter(p.build());
        }
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_calculator() {
        let reg = AresToolRegistry::with_default_tools();
        assert!(reg.has_tool("calculator"));
        assert!(reg.get_tool("calculator").is_some());
        assert!(!reg.is_empty());
    }

    #[test]
    fn empty_registry_is_empty() {
        let reg = AresToolRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(!reg.has_tool("calculator"));
    }

    #[test]
    fn calculator_has_expected_parameters() {
        let reg = AresToolRegistry::with_default_tools();
        let calc = reg.get_tool("calculator").expect("calculator present");
        assert_eq!(calc.name, "calculator");
        assert!(!calc.description.is_empty());
        // ares-server's Calculator uses a structured API: {operation, a, b}
        // (e.g. operation="add", a=2, b=3) — not an expression string.
        // This integration test validates that the real ares API matches
        // our JSON Schema → Parameter conversion.
        let names: Vec<&String> = calc.parameters.iter().map(|p| &p.name).collect();
        assert!(
            names.iter().any(|n| n.as_str() == "operation"),
            "calculator missing 'operation' parameter; got: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.as_str() == "a"),
            "calculator missing 'a' operand; got: {names:?}"
        );
        assert!(
            names.iter().any(|n| n.as_str() == "b"),
            "calculator missing 'b' operand; got: {names:?}"
        );
        // Structural check: at least the three documented parameters.
        assert!(calc.parameters.len() >= 3);
    }

    #[test]
    fn get_all_tools_matches_len() {
        let reg = AresToolRegistry::with_default_tools();
        assert_eq!(reg.get_all_tools().len(), reg.len());
    }

    #[test]
    fn missing_tool_returns_none() {
        let reg = AresToolRegistry::with_default_tools();
        assert!(reg.get_tool("nonexistent_tool_xyz").is_none());
    }

    #[test]
    fn default_impl_matches_with_default_tools() {
        let default = AresToolRegistry::default();
        let explicit = AresToolRegistry::with_default_tools();
        assert_eq!(default.len(), explicit.len());
        assert_eq!(default.has_tool("calculator"), explicit.has_tool("calculator"));
    }

    #[tokio::test]
    async fn ares_client_creates_without_connect() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        let client = AresMcpClient::new(transport);
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn ares_client_registry_accessible() {
        let transport =
            McpTransport::new_http("test".to_string(), "http://localhost:8080".to_string());
        let client = AresMcpClient::new(transport);
        let reg = client.registry();
        assert!(reg.has_tool("calculator"));
    }
}
