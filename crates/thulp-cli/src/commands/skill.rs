use clap::{Subcommand, ValueEnum};
use serde_json::json;
use std::path::{Path, PathBuf};
use crate::output::Output;

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    /// List available skills
    List {
        /// Filter by tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Show skills from specific scope
        #[arg(short, long, value_enum)]
        scope: Option<SkillScope>,
    },

    /// Show skill details
    Show {
        /// Skill name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Execute a skill workflow
    Run {
        /// Skill name
        #[arg(value_name = "NAME")]
        name: String,

        /// Input parameters as key=value pairs
        #[arg(value_name = "PARAMS")]
        params: Vec<String>,

        /// Parameters as JSON string
        #[arg(short, long)]
        json: Option<String>,

        /// Timeout in seconds (per step)
        #[arg(short, long, default_value = "60")]
        timeout: u64,

        /// Dry run (show execution plan without running)
        #[arg(long)]
        dry_run: bool,

        /// Continue on step failure
        #[arg(long)]
        continue_on_error: bool,
    },

    /// Validate a skill definition
    Validate {
        /// Path to skill file (SKILL.md or skill.yaml)
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Export skill as shell script
    Export {
        /// Skill name
        #[arg(value_name = "NAME")]
        name: String,

        /// Output file (default: stdout)
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,

        /// Export format
        #[arg(short, long, value_enum, default_value = "shell")]
        format: ExportFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SkillScope {
    /// Global skills (~/.thulp/skills)
    Global,
    /// Workspace skills (.thulp/skills)
    Workspace,
    /// Project skills (./skills)
    Project,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    /// Shell script
    Shell,
    /// JSON workflow
    Json,
    /// YAML workflow
    Yaml,
}

pub async fn handle_skill_commands(
    command: SkillCommands,
    workspace_dir: &Path,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        SkillCommands::List { tag, scope } => {
            handle_skill_list(workspace_dir, tag, scope, output)?;
        }
        SkillCommands::Show { name } => {
            handle_skill_show(workspace_dir, &name, output)?;
        }
        SkillCommands::Run {
            name,
            params,
            json,
            timeout,
            dry_run,
            continue_on_error,
        } => {
            handle_skill_run(SkillRunOpts {
                workspace_dir,
                name: &name,
                params,
                json_params: json,
                timeout,
                dry_run,
                continue_on_error,
                output,
            })
            .await?;
        }
        SkillCommands::Validate { file } => {
            handle_skill_validate(&file, output)?;
        }
        SkillCommands::Export {
            name,
            out: output_file,
            format,
        } => {
            handle_skill_export(workspace_dir, &name, output_file, format, output)?;
        }
    }
    Ok(())
}

pub fn handle_skill_list(
    workspace_dir: &Path,
    tag: Option<String>,
    scope: Option<SkillScope>,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut skills = Vec::new();

    // Collect skills from different scopes
    let scopes_to_check: Vec<(SkillScope, PathBuf)> = match scope {
        Some(s) => vec![(s, get_scope_path(workspace_dir, s))],
        None => vec![
            (SkillScope::Project, workspace_dir.join("skills")),
            (SkillScope::Workspace, workspace_dir.join(".thulp/skills")),
            (
                SkillScope::Global,
                dirs::home_dir().unwrap_or_default().join(".thulp/skills"),
            ),
        ],
    };

    for (scope, path) in scopes_to_check {
        if path.exists() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let skill_md = entry_path.join("SKILL.md");
                        let skill_yaml = entry_path.join("skill.yaml");
                        if skill_md.exists() || skill_yaml.exists() {
                            let name = entry_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            // Simple tag filtering (would parse SKILL.md in full impl)
                            if tag.is_none() {
                                skills.push(json!({
                                    "name": name,
                                    "scope": format!("{:?}", scope).to_lowercase(),
                                    "path": entry_path.display().to_string()
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    if output.is_json() {
        output.print_json(&json!({ "skills": skills }));
    } else {
        if skills.is_empty() {
            output.print_text("No skills found.");
            output.print_text("");
            output.print_text("Create skills in:");
            output.print_text("  - ./skills/           (project)");
            output.print_text("  - .thulp/skills/      (workspace)");
            output.print_text("  - ~/.thulp/skills/    (global)");
        } else {
            output.print_text("Available skills:");
            output.print_text("");
            for skill in &skills {
                let name = skill["name"].as_str().unwrap_or("?");
                let scope = skill["scope"].as_str().unwrap_or("?");
                output.print_text(&format!("  📋 {} ({})", name, scope));
            }
        }
    }

    Ok(())
}

pub fn get_scope_path(workspace_dir: &Path, scope: SkillScope) -> PathBuf {
    match scope {
        SkillScope::Project => workspace_dir.join("skills"),
        SkillScope::Workspace => workspace_dir.join(".thulp/skills"),
        SkillScope::Global => dirs::home_dir().unwrap_or_default().join(".thulp/skills"),
    }
}

pub fn handle_skill_show(
    workspace_dir: &Path,
    name: &str,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    // Search for skill in all scopes
    let scopes = [
        workspace_dir.join("skills").join(name),
        workspace_dir.join(".thulp/skills").join(name),
        dirs::home_dir()
            .unwrap_or_default()
            .join(".thulp/skills")
            .join(name),
    ];

    for skill_dir in scopes {
        let skill_md = skill_dir.join("SKILL.md");
        if skill_md.exists() {
            let content = std::fs::read_to_string(&skill_md)?;
            if output.is_json() {
                output.print_json(&json!({
                    "name": name,
                    "path": skill_dir.display().to_string(),
                    "content": content
                }));
            } else {
                output.print_text(&format!("Skill: {}", name));
                output.print_text(&format!("Path: {}", skill_dir.display()));
                output.print_text("");
                output.print_text(&content);
            }
            return Ok(());
        }

        let skill_yaml = skill_dir.join("skill.yaml");
        if skill_yaml.exists() {
            let content = std::fs::read_to_string(&skill_yaml)?;
            if output.is_json() {
                let parsed: serde_json::Value = serde_yaml::from_str(&content)?;
                output.print_json(&json!({
                    "name": name,
                    "path": skill_dir.display().to_string(),
                    "definition": parsed
                }));
            } else {
                output.print_text(&format!("Skill: {}", name));
                output.print_text(&format!("Path: {}", skill_dir.display()));
                output.print_text("");
                output.print_text(&content);
            }
            return Ok(());
        }
    }

    Err(format!("Skill '{}' not found", name).into())
}

pub struct SkillRunOpts<'a> {
    workspace_dir: &'a Path,
    name: &'a str,
    params: Vec<String>,
    json_params: Option<String>,
    timeout: u64,
    dry_run: bool,
    continue_on_error: bool,
    output: &'a Output,
}

pub async fn handle_skill_run(
    opts: SkillRunOpts<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let SkillRunOpts {
        workspace_dir: _workspace_dir,
        name,
        params,
        json_params,
        timeout,
        dry_run,
        continue_on_error,
        output,
    } = opts;
    // Parse parameters
    let parameters: serde_json::Value = if let Some(json_str) = json_params {
        serde_json::from_str(&json_str)?
    } else {
        let mut map = serde_json::Map::new();
        for param in params {
            if let Some((key, value)) = param.split_once('=') {
                let parsed_value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));
                map.insert(key.to_string(), parsed_value);
            }
        }
        serde_json::Value::Object(map)
    };

    if dry_run {
        if output.is_json() {
            output.print_json(&json!({
                "dry_run": true,
                "skill": name,
                "parameters": parameters,
                "timeout": timeout,
                "continue_on_error": continue_on_error,
                "status": "would_execute"
            }));
        } else {
            output.print_text("🔍 Dry run - would execute skill:");
            output.print_text(&format!("   Skill: {}", name));
            output.print_text(&format!("   Timeout: {}s per step", timeout));
            output.print_text(&format!("   Continue on error: {}", continue_on_error));
            output.print_text(&format!(
                "   Parameters: {}",
                serde_json::to_string_pretty(&parameters)?
            ));
        }
        return Ok(());
    }

    // Placeholder for actual skill execution
    // In full implementation: load skill, create executor, run steps
    if output.is_json() {
        output.print_json(&json!({
            "status": "not_implemented",
            "skill": name,
            "message": "Skill execution requires loaded workspace and MCP connections"
        }));
    } else {
        output.print_text(&format!("🚀 Executing skill: {}", name));
        output.print_text("");
        output.print_text("⚠️  Full skill execution requires:");
        output.print_text("   1. Initialized workspace (thulp init)");
        output.print_text("   2. Configured MCP servers (thulp config add-server)");
        output.print_text("   3. Valid skill definition");
    }

    Ok(())
}

pub fn handle_skill_validate(file: &Path, output: &Output) -> Result<(), Box<dyn std::error::Error>> {
    if !file.exists() {
        return Err(format!("File not found: {}", file.display()).into());
    }

    let content = std::fs::read_to_string(file)?;
    let file_name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let result = if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
        serde_yaml::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("YAML parse error: {}", e))
    } else if file_name == "SKILL.md" {
        // Basic SKILL.md validation - check for YAML frontmatter
        if let Some(after_start) = content.strip_prefix("---") {
            let end = after_start.find("---");
            if let Some(end_idx) = end {
                let frontmatter = &after_start[..end_idx];
                serde_yaml::from_str::<serde_json::Value>(frontmatter)
                    .map_err(|e| format!("Frontmatter parse error: {}", e))
            } else {
                Err("Missing closing --- for frontmatter".to_string())
            }
        } else {
            Err("SKILL.md should start with YAML frontmatter (---)".to_string())
        }
    } else {
        Err("Unknown file type. Expected .yaml, .yml, or SKILL.md".to_string())
    };

    if output.is_json() {
        match result {
            Ok(parsed) => output.print_json(&json!({
                "valid": true,
                "file": file.display().to_string(),
                "parsed": parsed
            })),
            Err(e) => output.print_json(&json!({
                "valid": false,
                "file": file.display().to_string(),
                "error": e
            })),
        }
    } else {
        match result {
            Ok(_) => {
                output.print_text(&format!("✅ Valid: {}", file.display()));
            }
            Err(e) => {
                output.print_text(&format!("❌ Invalid: {}", file.display()));
                output.print_text(&format!("   Error: {}", e));
            }
        }
    }

    Ok(())
}

pub fn handle_skill_export(
    _workspace_dir: &Path,
    name: &str,
    output_file: Option<PathBuf>,
    format: ExportFormat,
    output: &Output,
) -> Result<(), Box<dyn std::error::Error>> {
    // For now, generate a placeholder shell script
    let shell_script = format!(
        r#"#!/bin/bash
# Skill: {}
# Exported by thulp

set -euo pipefail

echo "Executing skill: {}"

# TODO: Add actual tool calls here
# This is a placeholder export

echo "Skill execution complete"
"#,
        name, name
    );

    let exported = match format {
        ExportFormat::Shell => shell_script,
        ExportFormat::Json => serde_json::to_string_pretty(&json!({
            "skill": name,
            "steps": [],
            "note": "Placeholder export"
        }))?,
        ExportFormat::Yaml => serde_yaml::to_string(&json!({
            "skill": name,
            "steps": [],
            "note": "Placeholder export"
        }))?,
    };

    if let Some(path) = output_file {
        std::fs::write(&path, &exported)?;
        if !output.is_json() {
            output.print_text(&format!("✅ Exported to: {}", path.display()));
        }
    } else {
        println!("{}", exported);
    }

    Ok(())
}
