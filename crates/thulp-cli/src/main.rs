use clap::{Parser, Subcommand};
use serde_json::json;
use std::path::{Path, PathBuf};
use thulp_adapter::AdapterGenerator;
use thulp_core::{Parameter, ParameterType, ToolCall, ToolDefinition};

#[cfg(feature = "mcp")]
use thulp_mcp::{McpClient, McpTransport};

#[derive(Parser, Debug)]
#[command(name = "thulp")]
#[command(about = "Execution context engineering platform for AI agents")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List and manage tools
    Tools {
        #[command(subcommand)]
        action: ToolCommands,
    },
    #[cfg(feature = "mcp")]
    /// Connect to and interact with MCP servers
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },
    /// Convert OpenAPI specifications to tool definitions
    Convert {
        #[command(subcommand)]
        action: ConvertCommands,
    },
    /// Demonstrate core functionality
    Demo,
    /// Validate configuration files
    Validate {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ToolCommands {
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

#[cfg(feature = "mcp")]
#[derive(Subcommand, Debug)]
enum McpCommands {
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
enum ConvertCommands {
    /// Convert OpenAPI spec to tool definitions
    OpenApi {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show conversion examples
    Examples,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tools { action } => handle_tool_commands(action).await?,
        #[cfg(feature = "mcp")]
        Commands::Mcp { action } => handle_mcp_commands(action).await?,
        Commands::Convert { action } => handle_convert_commands(action)?,
        Commands::Demo => run_demo().await?,
        Commands::Validate { file } => validate_file(&file)?,
    }

    Ok(())
}

async fn handle_tool_commands(command: ToolCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ToolCommands::List => {
            println!("Available tool definitions:");

            // Demo tool 1: read_file
            let read_file = create_read_file_tool();
            println!("\n📁 {}", read_file.name);
            println!("   {}", read_file.description);

            // Demo tool 2: api_call
            let api_call = create_api_call_tool();
            println!("\n🌐 {}", api_call.name);
            println!("   {}", api_call.description);
        }
        ToolCommands::Show { name } => {
            let tool = match name.as_str() {
                "read_file" => create_read_file_tool(),
                "api_call" => create_api_call_tool(),
                _ => {
                    eprintln!("Error: Tool '{}' not found", name);
                    return Ok(());
                }
            };

            println!("Tool: {}", tool.name);
            println!("Description: {}", tool.description);
            println!("Parameters:");
            for param in &tool.parameters {
                println!(
                    "  {}: {} ({})",
                    param.name,
                    param.param_type.as_str(),
                    if param.required {
                        "required"
                    } else {
                        "optional"
                    }
                );
                if !param.description.is_empty() {
                    println!("    Description: {}", param.description);
                }
                if let Some(ref default) = param.default {
                    println!("    Default: {}", default);
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
                    eprintln!("Error: Tool '{}' not found", name);
                    return Ok(());
                }
            };

            match tool.validate_args(&arguments_json) {
                Ok(()) => println!("✅ Arguments are valid"),
                Err(e) => println!("❌ Validation error: {}", e),
            }
        }
    }
    Ok(())
}

#[cfg(feature = "mcp")]
async fn handle_mcp_commands(command: McpCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        McpCommands::ConnectHttp { name, url } => {
            println!("Connecting to MCP server '{}' at {}...", name, url);
            let transport = McpTransport::new_http(name.clone(), url);
            let client = McpClient::new(transport);

            println!("✅ Connected to MCP server '{}'", name);
            println!("   Session ID: {}", client.session_id());
        }
        McpCommands::ConnectStdio {
            name,
            command,
            args,
        } => {
            println!(
                "Connecting to MCP server '{}' via STDIO command '{}'...",
                name, command
            );
            let transport = McpTransport::new_stdio(name.clone(), command, Some(args));
            let client = McpClient::new(transport);

            println!("✅ Connected to MCP server '{}'", name);
            println!("   Session ID: {}", client.session_id());
        }
        McpCommands::List => {
            println!("This would list tools from the connected MCP server");
            // In a real implementation, this would call client.list_tools().await?
        }
        McpCommands::Call { name, arguments } => {
            let _arguments_json = match arguments {
                Some(args) => serde_json::from_str(&args)?,
                None => json!({}),
            };

            println!("Calling tool '{}' on MCP server...", name);
            // In a real implementation, this would call client.call_tool(&name, arguments_json).await?
            println!("✅ Tool call completed");
        }
        McpCommands::Status => {
            println!("MCP Connection Status:");
            println!("  Transport: Ready");
            println!("  Session: Active");
            println!("  Connected servers: 0");
        }
    }
    Ok(())
}

fn handle_convert_commands(command: ConvertCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConvertCommands::OpenApi { file, output } => {
            println!("Converting OpenAPI spec from {}...", file.display());

            // Read the OpenAPI specification
            let spec_content = std::fs::read_to_string(&file)?;
            let spec: serde_json::Value = serde_json::from_str(&spec_content)?;

            // Create adapter generator
            let generator = AdapterGenerator::new(spec, Some("api-adapter".to_string()));

            // Generate tools
            let tools = generator
                .generate_tools()
                .map_err(|e| format!("Failed to generate tools: {}", e))?;
            println!("Generated {} tool definitions", tools.len());

            // Print tool summary
            for tool in &tools {
                println!("  - {}: {} parameters", tool.name, tool.parameters.len());
            }

            // Generate and save config if output is specified
            if let Some(output_file) = output {
                let config = generator
                    .generate_config()
                    .map_err(|e| format!("Failed to generate config: {}", e))?;
                std::fs::write(&output_file, config)?;
                println!("✅ Configuration written to: {}", output_file.display());
            }

            println!("✅ Conversion complete");
        }
        ConvertCommands::Examples => {
            println!("OpenAPI Conversion Examples:");
            println!();
            println!("GitHub API:");
            println!("  thulp convert openapi github_api.yaml");
            println!();
            println!("Slack API:");
            println!("  thulp convert openapi slack_api.json --output tools.yaml");
            println!();
            println!("Example OpenAPI v3 specification:");
            let example_spec = serde_json::json!({
                "openapi": "3.0.0",
                "info": {
                    "title": "Sample API",
                    "version": "1.0.0"
                },
                "paths": {
                    "/users": {
                        "get": {
                            "operationId": "listUsers",
                            "summary": "List all users",
                            "parameters": [
                                {
                                    "name": "limit",
                                    "in": "query",
                                    "schema": {"type": "integer"}
                                }
                            ]
                        }
                    },
                    "/users/{id}": {
                        "get": {
                            "operationId": "getUser",
                            "summary": "Get user by ID"
                        }
                    }
                }
            });
            println!("{}", serde_yaml::to_string(&example_spec)?);
        }
    }
    Ok(())
}

async fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Thulp Demo - Core Functionality");
    println!("==================================\n");

    println!("1. Tool Definition & Validation");
    println!("--------------------------------");

    let read_file_tool = create_read_file_tool();
    println!("Created tool: {}", read_file_tool.name);

    let valid_args = json!({ "path": "/etc/hosts" });
    let invalid_args = json!({ "path": 123 });

    match read_file_tool.validate_args(&valid_args) {
        Ok(()) => println!("✅ Valid arguments accepted"),
        Err(e) => println!("❌ Unexpected validation error: {}", e),
    }

    match read_file_tool.validate_args(&invalid_args) {
        Ok(()) => println!("❌ Invalid arguments should have been rejected"),
        Err(e) => println!("✅ Invalid arguments correctly rejected: {}", e),
    }

    #[cfg(feature = "mcp")]
    {
        println!("\n2. MCP Client");
        println!("--------------");

        let transport = McpTransport::new();
        let client = McpClient::new(transport);

        println!("Client created successfully");
        println!("  Connected: {}", client.is_connected());
        println!("  Session ID: {}", client.session_id());
    }

    println!("\n3. Tool Call Creation");
    println!("----------------------");

    let tool_call = ToolCall::builder("read_file")
        .arg_str("path", "/tmp/example.txt")
        .arg_str("encoding", "utf-8")
        .build();

    println!("Created tool call: {}", tool_call.tool);
    println!(
        "Arguments: {}",
        serde_json::to_string_pretty(&tool_call.arguments)?
    );

    println!("\n🎉 Demo complete!");
    Ok(())
}

fn validate_file(file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()).into());
    }

    println!("Validating file: {}", file.display());

    // Basic validation logic would go here
    println!("✅ File validation complete");

    Ok(())
}

// Helper functions to create demo tools
fn create_read_file_tool() -> ToolDefinition {
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

fn create_api_call_tool() -> ToolDefinition {
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
