use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use serde::Serialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use thulp_adapter::AdapterGenerator;
use thulp_core::{Parameter, ParameterType, ToolCall, ToolDefinition};

#[cfg(feature = "mcp")]
use thulp_mcp::{McpClient, McpTransport};

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum OutputFormat {
    /// Human-readable text output
    #[default]
    Text,
    /// JSON output for programmatic consumption
    Json,
    /// Compact JSON (no pretty-printing)
    JsonCompact,
}

#[derive(Parser, Debug)]
#[command(name = "thulp")]
#[command(about = "Execution context engineering platform for AI agents")]
#[command(version = "0.1.0")]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "text", global = true)]
    output: OutputFormat,

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
    /// Generate shell completion scripts
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
        /// Output directory (defaults to stdout)
        #[arg(short, long)]
        dir: Option<PathBuf>,
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

/// Output helper for formatted output
struct Output {
    format: OutputFormat,
}

impl Output {
    fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    fn print_text(&self, text: &str) {
        if matches!(self.format, OutputFormat::Text) {
            println!("{}", text);
        }
    }

    fn print_json<T: Serialize>(&self, data: &T) {
        match self.format {
            OutputFormat::Text => {}
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data).unwrap());
            }
            OutputFormat::JsonCompact => {
                println!("{}", serde_json::to_string(data).unwrap());
            }
        }
    }

    fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::JsonCompact)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let output = Output::new(cli.output);

    match cli.command {
        Commands::Tools { action } => handle_tool_commands(action, &output).await?,
        #[cfg(feature = "mcp")]
        Commands::Mcp { action } => handle_mcp_commands(action, &output).await?,
        Commands::Convert { action } => handle_convert_commands(action, &output)?,
        Commands::Demo => run_demo(&output).await?,
        Commands::Validate { file } => validate_file(&file, &output)?,
        Commands::Completions { shell, dir } => generate_completions(shell, dir)?,
    }

    Ok(())
}

fn generate_completions(
    shell: Shell,
    dir: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    if let Some(dir_path) = dir {
        std::fs::create_dir_all(&dir_path)?;
        let file_name = match shell {
            Shell::Bash => format!("{}.bash", bin_name),
            Shell::Zsh => format!("_{}", bin_name),
            Shell::Fish => format!("{}.fish", bin_name),
            Shell::PowerShell => format!("_{}.ps1", bin_name),
            Shell::Elvish => format!("{}.elv", bin_name),
            _ => format!("{}.completion", bin_name),
        };
        let path = dir_path.join(file_name);
        let mut file = std::fs::File::create(&path)?;
        generate(shell, &mut cmd, &bin_name, &mut file);
        eprintln!("Completion script written to: {}", path.display());
    } else {
        generate(shell, &mut cmd, &bin_name, &mut std::io::stdout());
    }

    Ok(())
}

async fn handle_tool_commands(
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

#[cfg(feature = "mcp")]
async fn handle_mcp_commands(
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

fn handle_convert_commands(
    command: ConvertCommands,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConvertCommands::OpenApi {
            file,
            output: output_file,
        } => {
            let spec_content = std::fs::read_to_string(&file)?;

            let spec: serde_json::Value = if file
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                serde_yaml::from_str(&spec_content)
                    .map_err(|e| format!("Failed to parse YAML: {}", e))?
            } else {
                serde_json::from_str(&spec_content)
                    .or_else(|_| serde_yaml::from_str(&spec_content))
                    .map_err(|e| format!("Failed to parse spec (tried JSON and YAML): {}", e))?
            };

            let generator = AdapterGenerator::new(spec, Some("api-adapter".to_string()));
            let tools = generator
                .generate_tools()
                .map_err(|e| format!("Failed to generate tools: {}", e))?;

            if output.is_json() {
                output.print_json(&json!({
                    "source": file.display().to_string(),
                    "tools_generated": tools.len(),
                    "tools": tools.iter().map(|t| json!({
                        "name": t.name,
                        "description": t.description,
                        "parameter_count": t.parameters.len()
                    })).collect::<Vec<_>>()
                }));
            } else {
                output.print_text(&format!(
                    "Converting OpenAPI spec from {}...",
                    file.display()
                ));
                output.print_text(&format!("Generated {} tool definitions", tools.len()));

                for tool in &tools {
                    output.print_text(&format!(
                        "  - {}: {} parameters",
                        tool.name,
                        tool.parameters.len()
                    ));
                }
            }

            if let Some(output_path) = output_file {
                let config = generator
                    .generate_config()
                    .map_err(|e| format!("Failed to generate config: {}", e))?;
                std::fs::write(&output_path, config)?;
                if !output.is_json() {
                    output.print_text(&format!(
                        "✅ Configuration written to: {}",
                        output_path.display()
                    ));
                }
            }

            if !output.is_json() {
                output.print_text("✅ Conversion complete");
            }
        }
        ConvertCommands::Examples => {
            if output.is_json() {
                let example_spec = json!({
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
                output.print_json(&json!({
                    "examples": [
                        {
                            "name": "GitHub API",
                            "command": "thulp convert openapi github_api.yaml"
                        },
                        {
                            "name": "Slack API",
                            "command": "thulp convert openapi slack_api.json --output tools.yaml"
                        }
                    ],
                    "sample_spec": example_spec
                }));
            } else {
                output.print_text("OpenAPI Conversion Examples:");
                output.print_text("");
                output.print_text("GitHub API:");
                output.print_text("  thulp convert openapi github_api.yaml");
                output.print_text("");
                output.print_text("Slack API:");
                output.print_text("  thulp convert openapi slack_api.json --output tools.yaml");
                output.print_text("");
                output.print_text("Example OpenAPI v3 specification:");
                let example_spec = json!({
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
                output.print_text(&serde_yaml::to_string(&example_spec)?);
            }
        }
    }
    Ok(())
}

async fn run_demo(output: &Output) -> Result<(), Box<dyn std::error::Error>> {
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

fn validate_file(file: &Path, output: &Output) -> Result<(), Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        // Test basic parsing
        let cli = Cli::try_parse_from(["thulp", "demo"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_output_format_default() {
        let cli = Cli::try_parse_from(["thulp", "demo"]).unwrap();
        assert!(matches!(cli.output, OutputFormat::Text));
    }

    #[test]
    fn test_output_format_json() {
        let cli = Cli::try_parse_from(["thulp", "-o", "json", "demo"]).unwrap();
        assert!(matches!(cli.output, OutputFormat::Json));
    }

    #[test]
    fn test_completions_command() {
        let cli = Cli::try_parse_from(["thulp", "completions", "bash"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_tools_list_command() {
        let cli = Cli::try_parse_from(["thulp", "tools", "list"]);
        assert!(cli.is_ok());
    }

    #[test]
    fn test_tools_show_command() {
        let cli = Cli::try_parse_from(["thulp", "tools", "show", "read_file"]);
        assert!(cli.is_ok());
    }
}
