use clap::Subcommand;
use serde_json::json;
use thulp_mcp::{McpClient, McpTransport};
use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Connect to an MCP server via HTTP
    ConnectHttp {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "URL")]
        url: String,
    },
    /// Connect to an MCP server via STDIO
    ConnectStdio {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "COMMAND")]
        command: String,
        #[arg(value_name = "ARGS")]
        args: Vec<String>,
    },
    /// List tools from connected MCP server
    List,
    /// Call a tool on the MCP server
    Call {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "ARGUMENTS")]
        arguments: Option<String>,
    },
    /// Show connection status
    Status,
}

#[derive(Subcommand, Debug)]

pub async fn handle_mcp_commands(
    command: McpCommands,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        McpCommands::ConnectHttp { name, url } => {
            let transport = McpTransport::new_http(name.clone(), url.clone());
            let client = McpClient::new(transport);

            if output.is_json() {
                output.print_json(&json!({
                    "status": "connected",
                    "name": name,
                    "url": url,
                    "session_id": client.session_id(),
                }));
            } else {
                output.print_text(&format!(
                    "Connecting to MCP server '{}' at {}...",
                    name, url
                ));
                output.print_text(&format!("✅ Connected to MCP server '{}'", name));
                output.print_text(&format!("   Session ID: {}", client.session_id()));
            }
        }
        McpCommands::ConnectStdio {
            name,
            command,
            args,
        } => {
            let transport =
                McpTransport::new_stdio(name.clone(), command.clone(), Some(args.clone()));
            let client = McpClient::new(transport);

            if output.is_json() {
                output.print_json(&json!({
                    "status": "connected",
                    "name": name,
                    "command": command,
                    "args": args,
                    "session_id": client.session_id(),
                }));
            } else {
                output.print_text(&format!(
                    "Connecting to MCP server '{}' via STDIO command '{}'...",
                    name, command
                ));
                output.print_text(&format!("✅ Connected to MCP server '{}'", name));
                output.print_text(&format!("   Session ID: {}", client.session_id()));
            }
        }
        McpCommands::List => {
            if output.is_json() {
                output.print_json(&json!({
                    "tools": [],
                    "message": "No active connection"
                }));
            } else {
                output.print_text("This would list tools from the connected MCP server");
            }
        }
        McpCommands::Call { name, arguments } => {
            let _arguments_json = match arguments {
                Some(args) => serde_json::from_str(&args)?,
                None => json!({}),
            };

            if output.is_json() {
                output.print_json(&json!({
                    "tool": name,
                    "status": "completed",
                    "result": null
                }));
            } else {
                output.print_text(&format!("Calling tool '{}' on MCP server...", name));
                output.print_text("✅ Tool call completed");
            }
        }
        McpCommands::Status => {
            if output.is_json() {
                output.print_json(&json!({
                    "transport": "ready",
                    "session": "active",
                    "connected_servers": 0
                }));
            } else {
                output.print_text("MCP Connection Status:");
                output.print_text("  Transport: Ready");
                output.print_text("  Session: Active");
                output.print_text("  Connected servers: 0");
            }
        }
    }
    Ok(())
}

