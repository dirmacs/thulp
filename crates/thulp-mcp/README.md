# thulp-mcp

**Model Context Protocol (MCP) Integration for Thulp**

This crate provides transport implementations for connecting to MCP servers using the Model Context Protocol. It wraps `rs-utcp` to provide a Thulp-native interface for MCP tool discovery and execution.

## Features

- **STDIO Transport**: Spawn and communicate with local MCP servers via standard input/output
- **HTTPS Transport**: Connect to remote MCP servers over HTTPS
- **Tool Discovery**: Automatic conversion from MCP JSON Schema to Thulp `ToolDefinition`
- **Tool Execution**: Call MCP tools with parameter validation
- **Connection Management**: Handle server lifecycle (connect, disconnect, reconnect)
- **Error Handling**: Rich error types for transport and protocol errors

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-mcp = "0.2"
```

For Ares server support:

```toml
[dependencies]
thulp-mcp = { version = "0.2", features = ["ares"] }
```

## Usage

### Connecting via STDIO

Connect to a local MCP server using standard input/output:

```rust
use thulp_mcp::McpTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn and connect to a local MCP server
    let transport = McpTransport::stdio(
        "/path/to/mcp-server",  // Server executable path
        &["--verbose"],          // Arguments
        None,                    // Optional environment variables
    ).await?;

    println!("Connected to MCP server!");
    Ok(())
}
```

### Connecting via HTTPS

Connect to a remote MCP server over HTTPS:

```rust
use thulp_mcp::McpTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = McpTransport::https("https://mcp.example.com").await?;
    
    println!("Connected to remote MCP server!");
    Ok(())
}
```

### Listing Available Tools

Discover tools provided by the MCP server:

```rust
use thulp_mcp::McpTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = McpTransport::stdio(
        "/path/to/mcp-server",
        &[],
        None,
    ).await?;

    // List all available tools
    let tools = transport.list_tools().await?;
    
    for tool in &tools {
        println!("Tool: {}", tool.name);
        println!("  Description: {}", tool.description);
        println!("  Parameters:");
        for param in &tool.parameters {
            println!("    - {}: {:?} (required: {})", 
                param.name, 
                param.parameter_type, 
                param.required
            );
        }
    }
    
    Ok(())
}
```

### Calling a Tool

Execute an MCP tool with parameters:

```rust
use thulp_mcp::McpTransport;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = McpTransport::stdio(
        "/path/to/mcp-server",
        &[],
        None,
    ).await?;

    // Call a tool with parameters
    let result = transport.call_tool(
        "search",
        json!({
            "query": "rust programming",
            "max_results": 10
        })
    ).await?;

    println!("Tool result: {}", result);
    
    Ok(())
}
```

### Complete Example

```rust
use thulp_mcp::McpTransport;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to MCP server
    let transport = McpTransport::stdio(
        "/usr/local/bin/weather-mcp-server",
        &["--api-key", "your-api-key"],
        None,
    ).await?;

    // Discover available tools
    let tools = transport.list_tools().await?;
    println!("Available tools: {:?}", tools.iter().map(|t| &t.name).collect::<Vec<_>>());

    // Call a tool
    let weather = transport.call_tool(
        "get_weather",
        json!({
            "location": "San Francisco, CA",
            "units": "metric"
        })
    ).await?;

    println!("Weather: {}", weather);

    // Disconnect (optional - happens automatically on drop)
    transport.disconnect().await?;

    Ok(())
}
```

## MCP JSON Schema to Thulp Parameters

The `thulp-mcp` crate automatically converts MCP JSON Schema tool definitions to Thulp's type system:

| MCP JSON Schema Type | Thulp ParameterType |
|---------------------|---------------------|
| `"string"`          | `ParameterType::String` |
| `"integer"`         | `ParameterType::Integer` |
| `"number"`          | `ParameterType::Number` |
| `"boolean"`         | `ParameterType::Boolean` |
| `"array"`           | `ParameterType::Array` |
| `"object"`          | `ParameterType::Object` |

### Schema Parsing Example

Given this MCP tool schema:

```json
{
  "name": "create_file",
  "description": "Create a new file",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "File path"
      },
      "content": {
        "type": "string",
        "description": "File content"
      },
      "overwrite": {
        "type": "boolean",
        "description": "Overwrite if exists"
      }
    },
    "required": ["path", "content"]
  }
}
```

Thulp will parse it as:

```rust
ToolDefinition {
    name: "create_file",
    description: "Create a new file",
    parameters: vec![
        Parameter {
            name: "path",
            description: "File path",
            parameter_type: ParameterType::String,
            required: true,
            ..
        },
        Parameter {
            name: "content",
            description: "File content",
            parameter_type: ParameterType::String,
            required: true,
            ..
        },
        Parameter {
            name: "overwrite",
            description: "Overwrite if exists",
            parameter_type: ParameterType::Boolean,
            required: false,
            ..
        },
    ],
}
```

## Error Handling

The crate provides detailed error types for different failure scenarios:

```rust
use thulp_mcp::{McpTransport, McpError};

#[tokio::main]
async fn main() {
    let result = McpTransport::stdio("/invalid/path", &[], None).await;
    
    match result {
        Ok(transport) => println!("Connected!"),
        Err(e) => {
            eprintln!("Connection failed: {}", e);
            // Handle specific error types
            if let Some(io_err) = e.source() {
                eprintln!("IO error: {}", io_err);
            }
        }
    }
}
```

## Connection Lifecycle

```rust
use thulp_mcp::McpTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Connect
    let mut transport = McpTransport::stdio("/path/to/server", &[], None).await?;
    
    // 2. Use the transport
    let tools = transport.list_tools().await?;
    let result = transport.call_tool("some_tool", json!({})).await?;
    
    // 3. Disconnect (optional - happens automatically on drop)
    transport.disconnect().await?;
    
    // 4. Reconnect if needed
    transport.connect().await?;
    
    Ok(())
}
```

## Testing

The crate includes comprehensive tests including edge cases:

```bash
# Run all tests
cargo test -p thulp-mcp

# Run with output
cargo test -p thulp-mcp -- --nocapture

# Run specific test
cargo test -p thulp-mcp test_list_tools
```

## Implementation Details

### Transport Architecture

`McpTransport` wraps `rs-utcp`'s transport types:

- **STDIO**: Uses `StdioClientTransport` to communicate with child processes
- **HTTPS**: Uses `SseClientTransport` for Server-Sent Events over HTTPS

### Tool Provider

The crate implements `rs-utcp`'s `ToolProvider` trait to enable tool discovery:

```rust
impl ToolProvider for McpToolProvider {
    async fn call(&self, name: &str, args: Value) -> Result<Value> {
        // Forward to actual tool implementation
    }
}
```

### Schema Parsing

`ToolDefinition::parse_mcp_input_schema` handles the conversion from MCP JSON Schema to Thulp parameters. It supports:

- All standard JSON Schema primitive types
- Required vs optional parameters (from `required` array)
- Nested object schemas (flattened to parameters)
- Missing or malformed schemas (returns empty parameter list)

## Compatibility

- **MCP Protocol Version**: Compatible with MCP protocol as implemented by `rs-utcp` v0.3.0
- **rs-utcp**: Version 0.3.0
- **Rust**: 1.75+

## Feature Flags

### `ares`

Enables integration with the Ares MCP server implementation:

```toml
[dependencies]
thulp-mcp = { version = "0.2", features = ["ares"] }
```

This feature:
- Includes `ares-server` as a dependency
- Enables Ares-specific utilities and helpers
- Required for testing with Ares-based servers

## Examples

See the `examples/` directory for more usage examples:

- `stdio_example.rs`: Basic STDIO connection
- `https_example.rs`: Remote HTTPS connection
- `tool_discovery.rs`: Tool listing and introspection
- `tool_execution.rs`: Calling tools with validation

You can also run the Thulp examples that demonstrate MCP integration:

```bash
# Run the MCP example (requires MCP feature)
cargo run --example mcp --features mcp
```

See the root examples directory for more comprehensive examples.

## Troubleshooting

### Server Won't Start

```rust
// Check server path
let result = McpTransport::stdio("/path/to/server", &[], None).await;
match result {
    Err(e) => eprintln!("Failed to start server: {}", e),
    Ok(_) => println!("Success!"),
}
```

### Tool Not Found

```rust
// List available tools first
let tools = transport.list_tools().await?;
for tool in tools {
    println!("Available: {}", tool.name);
}
```

### Invalid Parameters

```rust
// Check tool definition for required parameters
let tools = transport.list_tools().await?;
let tool = tools.iter().find(|t| t.name == "my_tool").unwrap();

for param in &tool.parameters {
    if param.required {
        println!("Required: {} ({:?})", param.name, param.parameter_type);
    }
}
```

## Contributing

Contributions are welcome! Please ensure:

1. Tests pass: `cargo test -p thulp-mcp`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy -p thulp-mcp`
4. Add tests for new features
5. Update documentation

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license

at your option.

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [rs-utcp Documentation](https://docs.rs/rs-utcp/)
- [Thulp Repository](https://github.com/dirmacs/thulp)
