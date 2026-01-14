use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--package", "thulp-cli", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Execution context engineering platform"));
}

#[test]
fn test_cli_tools_list() {
    let output = Command::new("cargo")
        .args(["run", "--package", "thulp-cli", "--", "tools", "list"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available tool definitions"));
    assert!(stdout.contains("read_file"));
    assert!(stdout.contains("api_call"));
}

#[test]
fn test_cli_tools_show() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "thulp-cli",
            "--",
            "tools",
            "show",
            "read_file",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tool: read_file"));
    assert!(stdout.contains("path"));
    assert!(stdout.contains("encoding"));
}

#[test]
fn test_cli_tools_validate() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "thulp-cli",
            "--",
            "tools",
            "validate",
            "read_file",
            "{\"path\": \"/tmp/test.txt\"}",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ Arguments are valid"));
}

#[test]
#[cfg(feature = "mcp")]
fn test_cli_mcp_status() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "thulp-cli",
            "--features",
            "mcp",
            "--",
            "mcp",
            "status",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MCP Connection Status"));
}

#[test]
fn test_cli_demo() {
    let output = Command::new("cargo")
        .args(["run", "--package", "thulp-cli", "--", "demo"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Thulp Demo"));
    assert!(stdout.contains("Tool Definition & Validation"));
}

#[test]
fn test_cli_convert_examples() {
    let output = Command::new("cargo")
        .args(["run", "--package", "thulp-cli", "--", "convert", "examples"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("OpenAPI Conversion Examples"));
    assert!(stdout.contains("GitHub API"));
}
