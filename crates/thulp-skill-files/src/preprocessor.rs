//! Preprocessor for skill content.
//!
//! Handles substitution of:
//! - `$ARGUMENTS` - replaced with skill invocation arguments
//! - `!`command`` - replaced with shell command output
//! - `{{variable}}` - replaced with context values
//! - `${ENV_VAR}` - replaced with environment variables

use crate::error::{Result, SkillFileError};
use regex::Regex;
use std::collections::HashMap;

/// Preprocessor for skill content.
#[derive(Debug, Clone)]
pub struct SkillPreprocessor {
    /// Enable shell command execution.
    pub enable_commands: bool,
    /// Timeout for shell commands (seconds).
    pub command_timeout: u64,
    /// Enable environment variable substitution.
    pub enable_env_vars: bool,
}

impl Default for SkillPreprocessor {
    fn default() -> Self {
        Self {
            enable_commands: true,
            command_timeout: 30,
            enable_env_vars: true,
        }
    }
}

impl SkillPreprocessor {
    /// Create a new preprocessor with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a preprocessor with commands disabled (safe mode).
    pub fn safe() -> Self {
        Self {
            enable_commands: false,
            enable_env_vars: false,
            ..Default::default()
        }
    }

    /// Preprocess skill content with arguments and commands.
    ///
    /// Processing order:
    /// 1. `$ARGUMENTS` substitution
    /// 2. `!`command`` execution (if enabled)
    /// 3. `{{variable}}` context substitution
    /// 4. `${ENV_VAR}` environment substitution (if enabled)
    pub fn preprocess(
        &self,
        content: &str,
        arguments: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let mut result = content.to_string();

        // Step 1: Replace $ARGUMENTS
        result = self.substitute_arguments(&result, arguments);

        // Step 2: Execute !`command` blocks
        if self.enable_commands {
            result = self.execute_commands(&result)?;
        }

        // Step 3: Replace {{variable}} placeholders from context
        result = self.substitute_variables(&result, context)?;

        // Step 4: Replace ${ENV_VAR} placeholders
        if self.enable_env_vars {
            result = self.substitute_env_vars(&result);
        }

        Ok(result)
    }

    /// Substitute $ARGUMENTS placeholder.
    fn substitute_arguments(&self, content: &str, arguments: &str) -> String {
        content.replace("$ARGUMENTS", arguments)
    }

    /// Execute !`command` blocks and replace with output.
    fn execute_commands(&self, content: &str) -> Result<String> {
        let re = Regex::new(r"!`([^`]+)`")?;
        let mut result = content.to_string();
        let mut errors = Vec::new();

        // Collect all matches first to avoid borrowing issues
        let matches: Vec<_> = re
            .captures_iter(content)
            .map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().to_string(),
                )
            })
            .collect();

        for (full_match, command) in matches {
            match self.run_shell_command(&command) {
                Ok(output) => {
                    result = result.replace(&full_match, &output);
                }
                Err(e) => {
                    errors.push(format!("Command '{}': {}", command, e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(SkillFileError::CommandExecution(errors.join("; ")));
        }

        Ok(result)
    }

    /// Run a shell command and return its output.
    fn run_shell_command(&self, command: &str) -> Result<String> {
        #[cfg(target_os = "windows")]
        let output = std::process::Command::new("cmd")
            .args(["/C", command])
            .output()
            .map_err(|e| SkillFileError::CommandExecution(e.to_string()))?;

        #[cfg(not(target_os = "windows"))]
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| SkillFileError::CommandExecution(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SkillFileError::CommandExecution(format!(
                "Command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Substitute {{variable}} placeholders from context.
    fn substitute_variables(
        &self,
        content: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let re = Regex::new(r"\{\{([^}]+)\}\}")?;
        let mut result = content.to_string();

        // Collect all matches first
        let matches: Vec<_> = re
            .captures_iter(content)
            .map(|cap| {
                (
                    cap.get(0).unwrap().as_str().to_string(),
                    cap.get(1).unwrap().as_str().trim().to_string(),
                )
            })
            .collect();

        for (full_match, var_path) in matches {
            if let Some(value) = self.resolve_path(&var_path, context) {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Null => String::new(),
                    other => serde_json::to_string(other).unwrap_or_default(),
                };
                result = result.replace(&full_match, &value_str);
            }
            // Leave unresolved variables as-is (they might be for later processing)
        }

        Ok(result)
    }

    /// Resolve a dotted path like "step_name.output.field".
    fn resolve_path<'a>(
        &self,
        path: &str,
        context: &'a HashMap<String, serde_json::Value>,
    ) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();

        if parts.is_empty() {
            return None;
        }

        let mut current = context.get(parts[0])?;

        for part in &parts[1..] {
            current = current.get(*part)?;
        }

        Some(current)
    }

    /// Substitute ${ENV_VAR} environment variables.
    fn substitute_env_vars(&self, content: &str) -> String {
        let re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();

        re.replace_all(content, |caps: &regex::Captures| {
            let var_name = caps.get(1).unwrap().as_str();
            std::env::var(var_name).unwrap_or_default()
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_substitute_arguments() {
        let pp = SkillPreprocessor::new();
        let result = pp.substitute_arguments("Process: $ARGUMENTS", "file.txt");
        assert_eq!(result, "Process: file.txt");
    }

    #[test]
    fn test_substitute_arguments_multiple() {
        let pp = SkillPreprocessor::new();
        let result = pp.substitute_arguments("First: $ARGUMENTS, Second: $ARGUMENTS", "hello");
        assert_eq!(result, "First: hello, Second: hello");
    }

    #[test]
    fn test_substitute_variables_simple() {
        let pp = SkillPreprocessor::new();
        let mut context = HashMap::new();
        context.insert("name".to_string(), json!("Alice"));

        let result = pp
            .substitute_variables("Hello, {{name}}!", &context)
            .unwrap();
        assert_eq!(result, "Hello, Alice!");
    }

    #[test]
    fn test_substitute_variables_nested() {
        let pp = SkillPreprocessor::new();
        let mut context = HashMap::new();
        context.insert("user".to_string(), json!({"name": "Bob", "age": 30}));

        let result = pp
            .substitute_variables("Name: {{user.name}}, Age: {{user.age}}", &context)
            .unwrap();
        assert_eq!(result, "Name: Bob, Age: 30");
    }

    #[test]
    fn test_substitute_variables_missing() {
        let pp = SkillPreprocessor::new();
        let context = HashMap::new();

        let result = pp
            .substitute_variables("Hello, {{missing}}!", &context)
            .unwrap();
        // Missing variables are left as-is
        assert_eq!(result, "Hello, {{missing}}!");
    }

    #[test]
    fn test_substitute_env_vars() {
        let pp = SkillPreprocessor::new();
        std::env::set_var("TEST_SKILL_VAR", "test_value");

        let result = pp.substitute_env_vars("Value: ${TEST_SKILL_VAR}");
        assert_eq!(result, "Value: test_value");

        std::env::remove_var("TEST_SKILL_VAR");
    }

    #[test]
    fn test_preprocess_combined() {
        let pp = SkillPreprocessor::safe(); // Disable commands for test
        let mut context = HashMap::new();
        context.insert("project".to_string(), json!("myapp"));

        let content = "Project: {{project}}\nArgs: $ARGUMENTS";
        let result = pp.preprocess(content, "build --release", &context).unwrap();

        assert_eq!(result, "Project: myapp\nArgs: build --release");
    }

    #[test]
    fn test_command_execution_disabled() {
        let pp = SkillPreprocessor::safe();
        let context = HashMap::new();

        // Command should not be executed, left as-is
        let content = "Output: !`echo hello`";
        let result = pp.preprocess(content, "", &context).unwrap();
        assert_eq!(result, "Output: !`echo hello`");
    }
}
