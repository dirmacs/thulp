use clap::{Subcommand, ValueEnum};
use serde_json::json;
use std::path::{Path, PathBuf};
use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Get a configuration value
    Get {
        /// Configuration key (e.g., servers.github.url)
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: String,

        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Add an MCP server configuration
    AddServer {
        /// Server name
        #[arg(value_name = "NAME")]
        name: String,

        /// Server type (stdio, http)
        #[arg(short, long, value_enum)]
        r#type: ServerType,

        /// Command (for stdio) or URL (for http)
        #[arg(value_name = "TARGET")]
        target: String,

        /// Command arguments (for stdio)
        #[arg(short, long)]
        args: Vec<String>,
    },

    /// List configured servers
    Servers,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ServerType {
    /// STDIO transport (local command)
    Stdio,
    /// HTTP/SSE transport (remote server)
    Http,
}

/// Handle `thulp init` command
pub fn handle_init(
    dir: PathBuf,
    name: Option<String>,
    force: bool,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let thulp_dir = dir.join(".thulp");

    if thulp_dir.exists() && !force {
        if output.is_json() {
            output.print_json(&json!({
                "error": "already_initialized",
                "path": thulp_dir.display().to_string(),
                "hint": "Use --force to reinitialize"
            }));
        } else {
            output.print_text(&format!(
                "❌ Workspace already initialized at {}",
                thulp_dir.display()
            ));
            output.print_text("   Use --force to reinitialize");
        }
        return Ok(());
    }

    // Create directory structure
    std::fs::create_dir_all(&thulp_dir)?;
    std::fs::create_dir_all(thulp_dir.join("skills"))?;
    std::fs::create_dir_all(thulp_dir.join("sessions"))?;
    std::fs::create_dir_all(thulp_dir.join("cache"))?;

    // Determine workspace name
    let workspace_name = name.unwrap_or_else(|| {
        dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace")
            .to_string()
    });

    // Create config.yaml
    let config = json!({
        "name": workspace_name,
        "version": "1.0",
        "servers": {},
        "settings": {
            "default_timeout": 30,
            "max_retries": 3
        }
    });

    let config_path = thulp_dir.join("config.yaml");
    let config_yaml = serde_yaml::to_string(&config)?;
    std::fs::write(&config_path, config_yaml)?;

    // Create .gitignore
    let gitignore = "sessions/\ncache/\n*.log\n";
    std::fs::write(thulp_dir.join(".gitignore"), gitignore)?;

    if output.is_json() {
        output.print_json(&json!({
            "status": "initialized",
            "path": thulp_dir.display().to_string(),
            "name": workspace_name,
            "created": [
                "config.yaml",
                "skills/",
                "sessions/",
                "cache/",
                ".gitignore"
            ]
        }));
    } else {
        output.print_text(&format!(
            "✅ Initialized thulp workspace: {}",
            workspace_name
        ));
        output.print_text(&format!("   Location: {}", thulp_dir.display()));
        output.print_text("");
        output.print_text("   Created:");
        output.print_text("   ├── config.yaml    (workspace configuration)");
        output.print_text("   ├── skills/        (skill definitions)");
        output.print_text("   ├── sessions/      (session storage)");
        output.print_text("   ├── cache/         (temporary files)");
        output.print_text("   └── .gitignore");
        output.print_text("");
        output.print_text("   Next steps:");
        output.print_text(
            "   1. Add MCP servers: thulp config add-server <name> --type stdio <command>",
        );
        output.print_text("   2. List tools:      thulp tools list");
        output.print_text("   3. Run a tool:      thulp run <tool> key=value");
    }

    Ok(())
}

/// Handle `thulp config` subcommands
pub fn handle_config_commands(
    command: ConfigCommands,
    workspace_dir: &Path,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = workspace_dir.join(".thulp/config.yaml");

    match command {
        ConfigCommands::Show => {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                if output.is_json() {
                    let parsed: serde_json::Value = serde_yaml::from_str(&content)?;
                    output.print_json(&parsed);
                } else {
                    output.print_text(&content);
                }
            } else {
                if output.is_json() {
                    output.print_json(&json!({"error": "not_initialized"}));
                } else {
                    output.print_text("❌ No workspace found. Run 'thulp init' first.");
                }
            }
        }
        ConfigCommands::Get { key } => {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: serde_json::Value = serde_yaml::from_str(&content)?;

                let parts: Vec<&str> = key.split('.').collect();
                let mut current = &config;
                for part in &parts {
                    if let Some(next) = current.get(part) {
                        current = next;
                    } else {
                        return Err(format!("Key not found: {}", key).into());
                    }
                }

                if output.is_json() {
                    output.print_json(current);
                } else {
                    output.print_text(&serde_json::to_string_pretty(current)?);
                }
            } else {
                return Err("No workspace found. Run 'thulp init' first.".into());
            }
        }
        ConfigCommands::Set { key, value } => {
            if !config_path.exists() {
                return Err("No workspace found. Run 'thulp init' first.".into());
            }

            let content = std::fs::read_to_string(&config_path)?;
            let mut config: serde_json::Value = serde_yaml::from_str(&content)?;

            let parsed_value: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

            if let Some(obj) = config.as_object_mut() {
                obj.insert(key.clone(), parsed_value);
            }

            let updated = serde_yaml::to_string(&config)?;
            std::fs::write(&config_path, updated)?;

            if output.is_json() {
                output.print_json(&json!({"status": "updated", "key": key}));
            } else {
                output.print_text(&format!("✅ Set {} = {}", key, value));
            }
        }
        ConfigCommands::AddServer {
            name,
            r#type,
            target,
            args,
        } => {
            if !config_path.exists() {
                return Err("No workspace found. Run 'thulp init' first.".into());
            }

            let content = std::fs::read_to_string(&config_path)?;
            let mut config: serde_json::Value = serde_yaml::from_str(&content)?;

            let server_config = match r#type {
                ServerType::Stdio => json!({
                    "type": "stdio",
                    "command": target,
                    "args": args
                }),
                ServerType::Http => json!({
                    "type": "http",
                    "url": target
                }),
            };

            if let Some(obj) = config.as_object_mut() {
                let servers = obj.entry("servers").or_insert_with(|| json!({}));
                if let Some(servers_obj) = servers.as_object_mut() {
                    servers_obj.insert(name.clone(), server_config);
                }
            }

            let updated = serde_yaml::to_string(&config)?;
            std::fs::write(&config_path, updated)?;

            if output.is_json() {
                output.print_json(&json!({"status": "added", "server": name}));
            } else {
                output.print_text(&format!("✅ Added server: {}", name));
            }
        }
        ConfigCommands::Servers => {
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: serde_json::Value = serde_yaml::from_str(&content)?;

                let servers = config.get("servers").cloned().unwrap_or(json!({}));

                if output.is_json() {
                    output.print_json(&servers);
                } else {
                    if let Some(obj) = servers.as_object() {
                        if obj.is_empty() {
                            output.print_text("No servers configured.");
                        } else {
                            output.print_text("Configured servers:");
                            for (name, config) in obj {
                                let server_type = config
                                    .get("type")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("unknown");
                                output.print_text(&format!("  🔌 {} ({})", name, server_type));
                            }
                        }
                    }
                }
            } else {
                if output.is_json() {
                    output.print_json(&json!({"error": "not_initialized"}));
                } else {
                    output.print_text("❌ No workspace found. Run 'thulp init' first.");
                }
            }
        }
    }

    Ok(())
}
