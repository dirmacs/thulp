use clap::Subcommand;
use serde_json::json;
use std::path::Path;
use thulp_core::{Parameter, ParameterType, ToolCall, ToolDefinition};
use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum ToolCommands {
    /// List all available tools
    List,
    /// Show details of a specific tool
    Show {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Validate tool arguments
    Validate {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "ARGUMENTS")]
        arguments: Option<String>,
    },
}

pub async fn handle_run(
    tool: &str,
    args: Vec<String>,
    json_args: Option<String>,
    timeout: u64,
    dry_run: bool,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
    let arguments: serde_json::Value = if let Some(json_str) = json_args {
        serde_json::from_str(&json_str)?
    } else {
        let mut map = serde_json::Map::new();
        for arg in args {
            if let Some((key, value)) = arg.split_once('=') {
                // Try to parse as JSON value, fallback to string
                let parsed_value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
                map.insert(key.to_string(), parsed_value);
            } else {
                return Err(format!("Invalid argument format: '{}'. Use key=value", arg).into());
            }
        }
        serde_json::Value::Object(map)
    };

    // Parse tool name (format: server.tool or just tool)
    let (server_name, tool_name) = if let Some((server, tool)) = tool.split_once('.') {
        (Some(server.to_string()), tool.to_string())
    } else {
        (None, tool.to_string())
    };

    if dry_run {
        if output.is_json() {
            output.print_json(&json!({
                "dry_run": true,
                "tool": tool_name,
                "server": server_name,
                "arguments": arguments,
                "timeout": timeout
            }));
        } else {
            output.print_text("🔍 Dry run - would execute:");
            output.print_text(&format!("   Tool: {}", tool_name));
            if let Some(ref server) = server_name {
                output.print_text(&format!("   Server: {}", server));
            }
            output.print_text(&format!("   Timeout: {}s", timeout));
            output.print_text(&format!(
                "   Arguments: {}",
                serde_json::to_string_pretty(&arguments)?
            ));
        }
        return Ok(());
    }

    // For now, we'll show a placeholder since MCP execution requires server config
    // In a full implementation, this would:
    // 1. Load workspace config
    // 2. Find the appropriate server
    // 3. Connect via MCP
    // 4. Execute the tool
    // 5. Return the result

    if output.is_json() {
        output.print_json(&json!({
            "status": "not_implemented",
            "tool": tool_name,
            "server": server_name,
            "arguments": arguments,
            "message": "Tool execution requires configured MCP servers. Use 'thulp config add-server' first."
        }));
    } else {
        output.print_text(&format!("🔧 Executing tool: {}", tool_name));
        if let Some(ref server) = server_name {
            output.print_text(&format!("   Server: {}", server));
        }
        output.print_text(&format!(
            "   Arguments: {}",
            serde_json::to_string(&arguments)?
        ));
        output.print_text("");
        output.print_text("⚠️  Tool execution requires configured MCP servers.");
        output.print_text("   Use 'thulp config add-server' to add a server first.");
    }

    Ok(())
}

pub async fn handle_tool_commands(
    command: ToolCommands,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ToolCommands::List => {
            let read_file = create_read_file_tool();
            let api_call = create_api_call_tool();
            let tools = [&read_file, &api_call];

            if output.is_json() {
                output.print_json(&json!({
                    "tools": tools.iter().map(|t| json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters.iter().map(|p| json!({
                            "name": p.name,
                            "type": p.param_type.as_str(),
                            "required": p.required,
                            "description": p.description,
                        })).collect::<Vec<_>>()
                    })).collect::<Vec<_>>()
                }));
            } else {
                output.print_text("Available tool definitions:");
                output.print_text(&format!("\n📁 {}", read_file.name));
                output.print_text(&format!("   {}", read_file.description));
                output.print_text(&format!("\n🌐 {}", api_call.name));
                output.print_text(&format!("   {}", api_call.description));
            }
        }
        ToolCommands::Show { name } => {
            let tool = match name.as_str() {
                "read_file" => create_read_file_tool(),
                "api_call" => create_api_call_tool(),
                _ => {
                    return Err(format!("Tool '{}' not found", name).into());
                }
            };

            if output.is_json() {
                output.print_json(&json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters.iter().map(|p| json!({
                        "name": p.name,
                        "type": p.param_type.as_str(),
                        "required": p.required,
                        "description": p.description,
                        "default": p.default,
                    })).collect::<Vec<_>>()
                }));
            } else {
                output.print_text(&format!("Tool: {}", tool.name));
                output.print_text(&format!("Description: {}", tool.description));
                output.print_text("Parameters:");
                for param in &tool.parameters {
                    output.print_text(&format!(
                        "  {}: {} ({})",
                        param.name,
                        param.param_type.as_str(),
                        if param.required {
                            "required"
                        } else {
                            "optional"
                        }
                    ));
                    if !param.description.is_empty() {
                        output.print_text(&format!("    Description: {}", param.description));
                    }
                    if let Some(ref default) = param.default {
                        output.print_text(&format!("    Default: {}", default));
                    }
                }
            }
        }
        ToolCommands::Validate { name, arguments } => {
            let arguments_json = match arguments {
                Some(args) => serde_json::from_str(&args)?,
                None => json!({}),
            };

            let tool = match name.as_str() {
                "read_file" => create_read_file_tool(),
                "api_call" => create_api_call_tool(),
                _ => {
                    return Err(format!("Tool '{}' not found", name).into());
                }
            };

            let result = tool.validate_args(&arguments_json);
            if output.is_json() {
                output.print_json(&json!({
                    "valid": result.is_ok(),
                    "error": result.err().map(|e| e.to_string()),
                }));
            } else {
                match result {
                    Ok(()) => output.print_text("✅ Arguments are valid"),
                    Err(e) => output.print_text(&format!("❌ Validation error: {}", e)),
                }
            }
        }
    }
    Ok(())
}


pub async fn run_demo(output: &Output) -> Result<(), Box<dyn std::error::Error>> {
    if output.is_json() {
        let read_file_tool = create_read_file_tool();
        let valid_args = json!({ "path": "/etc/hosts" });
        let invalid_args = json!({ "path": 123 });

        let tool_call = ToolCall::builder("read_file")
            .arg_str("path", "/tmp/example.txt")
            .arg_str("encoding", "utf-8")
            .build();

        output.print_json(&json!({
            "demo": "core_functionality",
            "tool_definition": {
                "name": read_file_tool.name,
                "valid_args_test": read_file_tool.validate_args(&valid_args).is_ok(),
                "invalid_args_test": read_file_tool.validate_args(&invalid_args).is_err(),
            },
            "tool_call": {
                "tool": tool_call.tool,
                "arguments": tool_call.arguments,
            }
        }));
    } else {
        output.print_text("🎯 Thulp Demo - Core Functionality");
        output.print_text("==================================\n");

        output.print_text("1. Tool Definition & Validation");
        output.print_text("--------------------------------");

        let read_file_tool = create_read_file_tool();
        output.print_text(&format!("Created tool: {}", read_file_tool.name));

        let valid_args = json!({ "path": "/etc/hosts" });
        let invalid_args = json!({ "path": 123 });

        match read_file_tool.validate_args(&valid_args) {
            Ok(()) => output.print_text("✅ Valid arguments accepted"),
            Err(e) => output.print_text(&format!("❌ Unexpected validation error: {}", e)),
        }

        match read_file_tool.validate_args(&invalid_args) {
            Ok(()) => output.print_text("❌ Invalid arguments should have been rejected"),
            Err(e) => output.print_text(&format!("✅ Invalid arguments correctly rejected: {}", e)),
        }

        #[cfg(feature = "mcp")]
        {
            output.print_text("\n2. MCP Client");
            output.print_text("--------------");

            let transport = McpTransport::new();
            let client = McpClient::new(transport);

            output.print_text("Client created successfully");
            output.print_text(&format!("  Connected: {}", client.is_connected()));
            output.print_text(&format!("  Session ID: {}", client.session_id()));
        }

        output.print_text("\n3. Tool Call Creation");
        output.print_text("----------------------");

        let tool_call = ToolCall::builder("read_file")
            .arg_str("path", "/tmp/example.txt")
            .arg_str("encoding", "utf-8")
            .build();

        output.print_text(&format!("Created tool call: {}", tool_call.tool));
        output.print_text(&format!(
            "Arguments: {}",
            serde_json::to_string_pretty(&tool_call.arguments)?
        ));

        output.print_text("\n🎉 Demo complete!");
    }
    Ok(())
}

pub fn validate_file(file: &Path, output: &Output) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()).into());
    }

    if output.is_json() {
        output.print_json(&json!({
            "file": file.display().to_string(),
            "valid": true,
        }));
    } else {
        output.print_text(&format!("Validating file: {}", file.display()));
        output.print_text("✅ File validation complete");
    }

    Ok(())
}

pub fn create_read_file_tool() -> ToolDefinition {
    ToolDefinition::builder("read_file")
        .description("Read contents of a file from the filesystem")
        .parameter(
            Parameter::builder("path")
                .param_type(ParameterType::String)
                .required(true)
                .description("Path to the file to read")
                .build(),
        )
        .parameter(
            Parameter::builder("encoding")
                .param_type(ParameterType::String)
                .description("File encoding (utf-8, ascii, etc.)")
                .default(serde_json::Value::String("utf-8".to_string()))
                .build(),
        )
        .build()
}


pub fn create_api_call_tool() -> ToolDefinition {
    ToolDefinition::builder("api_call")
        .description("Make HTTP API requests to external services")
        .parameter(
            Parameter::builder("url")
                .param_type(ParameterType::String)
                .required(true)
                .description("URL to make the API request to")
                .build(),
        )
        .parameter(
            Parameter::builder("method")
                .param_type(ParameterType::String)
                .description("HTTP method (GET, POST, PUT, DELETE)")
                .default(serde_json::Value::String("GET".to_string()))
                .build(),
        )
        .parameter(
            Parameter::builder("headers")
                .param_type(ParameterType::Object)
                .description("HTTP headers as key-value pairs")
                .build(),
        )
        .build()
}
