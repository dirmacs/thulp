//! Example demonstrating MCP integration with Thulp
//!
//! This example shows:
//! - Connecting to an MCP server
//! - Listing available tools
//! - Calling tools on the MCP server
//! - Working with MCP transport

#[cfg(feature = "mcp")]
use serde_json::json;
#[cfg(feature = "mcp")]
use thulp_mcp::{McpClient, McpTransport};

#[cfg(feature = "mcp")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔌 Thulp MCP Integration Example");
    println!("===============================\n");

    // Example 1: Creating an HTTP transport
    println!("1. Creating HTTP transport");
    let transport = McpTransport::new_http(
        "github-api".to_string(),
        "https://api.github.com".to_string(),
    );
    println!("   Created HTTP transport for github-api");

    // Example 2: Creating an STDIO transport
    println!("\n2. Creating STDIO transport");
    let _stdio_transport = McpTransport::new_stdio(
        "local-echo".to_string(),
        "npx".to_string(),
        Some(vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-echo".to_string(),
        ]),
    );
    println!("   Created STDIO transport for local echo server");

    // Example 3: Creating an MCP client
    println!("\n3. Creating MCP client");
    let mut client = McpClient::new(transport);
    println!(
        "   Created MCP client with session ID: {}",
        client.session_id()
    );

    // Example 4: Connecting to the server
    println!("\n4. Connecting to MCP server");
    match client.connect().await {
        Ok(()) => println!("   ✅ Connected successfully"),
        Err(e) => println!("   ⚠️  Connection failed (expected in example): {}", e),
    }

    // Example 5: Using the convenience methods
    println!("\n5. Using convenience connection methods");
    println!("   These would connect to real MCP servers:");
    println!(
        "   - McpClient::connect_http(\"server\", \"http://localhost:8080\".to_string()).await?"
    );
    println!("   - McpClient::connect_stdio(\"server\", \"command\".to_string(), None).await?");

    // Example 6: Demonstrating tool definition parsing from MCP schema
    println!("\n6. Parsing MCP input schema");
    let mcp_schema = json!({
        "type": "object",
        "properties": {
            "username": {
                "type": "string",
                "description": "GitHub username"
            },
            "repository": {
                "type": "string",
                "description": "Repository name"
            },
            "per_page": {
                "type": "integer",
                "description": "Number of items per page",
                "default": 30
            }
        },
        "required": ["username", "repository"]
    });

    let parameters = thulp_core::ToolDefinition::parse_mcp_input_schema(&mcp_schema)?;
    println!("   Parsed {} parameters from MCP schema:", parameters.len());
    for param in &parameters {
        println!(
            "     - {}: {} ({})",
            param.name,
            param.param_type.as_str(),
            if param.required {
                "required"
            } else {
                "optional"
            }
        );
    }

    println!("\n🎉 MCP example completed!");
    println!("💡 Note: This example shows the API usage patterns.");
    println!("   Actual connections would require running MCP servers.");

    Ok(())
}

#[cfg(not(feature = "mcp"))]
fn main() {
    println!("This example requires the 'mcp' feature to be enabled.");
    println!("Run with: cargo run --example mcp_example --features mcp");
}
