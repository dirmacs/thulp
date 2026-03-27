use clap::Subcommand;
use serde_json::json;
use std::path::PathBuf;
use thulp_adapter::AdapterGenerator;
use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum ConvertCommands {
    /// Convert OpenAPI spec to tool definitions
    OpenApi {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Output file for generated config
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Show conversion examples
    Examples,
}


pub fn handle_convert_commands(
    command: ConvertCommands,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ConvertCommands::OpenApi {
            file,
            out: output_file,
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


