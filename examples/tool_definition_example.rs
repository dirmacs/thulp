//! Example demonstrating how to define and use tools with Thulp
//!
//! This example shows:
//! - Creating tool definitions with various parameter types
//! - Validating arguments against tool definitions
//! - Executing tool calls
//! - Working with tool results

use serde_json::json;
use thulp_core::{Parameter, ParameterType, ToolCall, ToolDefinition};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔧 Thulp Tool Definition Example");
    println!("==============================\n");

    // Example 1: Simple file reading tool
    println!("1. Creating a file reading tool");
    let read_file_tool = ToolDefinition::builder("read_file")
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
                .default(json!("utf-8"))
                .build(),
        )
        .build();

    println!("   Tool name: {}", read_file_tool.name);
    println!("   Description: {}", read_file_tool.description);
    println!("   Parameters: {}", read_file_tool.parameters.len());
    for param in &read_file_tool.parameters {
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

    // Example 2: API call tool with complex parameters
    println!("\n2. Creating an API call tool");
    let api_call_tool = ToolDefinition::builder("api_call")
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
                .default(json!("GET"))
                .enum_value(json!("GET"))
                .enum_value(json!("POST"))
                .enum_value(json!("PUT"))
                .enum_value(json!("DELETE"))
                .build(),
        )
        .parameter(
            Parameter::builder("headers")
                .param_type(ParameterType::Object)
                .description("HTTP headers as key-value pairs")
                .build(),
        )
        .parameter(
            Parameter::builder("data")
                .param_type(ParameterType::Object)
                .description("Request body data")
                .build(),
        )
        .build();

    println!("   Tool name: {}", api_call_tool.name);
    println!("   Description: {}", api_call_tool.description);
    println!("   Parameters: {}", api_call_tool.parameters.len());

    // Example 3: Validating arguments
    println!("\n3. Validating arguments");

    // Valid arguments for read_file
    let valid_read_args = json!({
        "path": "/etc/hosts",
        "encoding": "utf-8"
    });

    match read_file_tool.validate_args(&valid_read_args) {
        Ok(()) => println!("   ✅ Valid arguments accepted for read_file"),
        Err(e) => println!("   ❌ Unexpected validation error: {}", e),
    }

    // Invalid arguments (wrong type)
    let invalid_read_args = json!({
        "path": 123,  // Should be string
        "encoding": "utf-8"
    });

    match read_file_tool.validate_args(&invalid_read_args) {
        Ok(()) => println!("   ❌ Invalid arguments should have been rejected"),
        Err(e) => println!("   ✅ Invalid arguments correctly rejected: {}", e),
    }

    // Missing required argument
    let missing_arg_args = json!({
        "encoding": "utf-8"
        // Missing required "path" parameter
    });

    match read_file_tool.validate_args(&missing_arg_args) {
        Ok(()) => println!("   ❌ Missing argument should have been rejected"),
        Err(e) => println!("   ✅ Missing argument correctly rejected: {}", e),
    }

    // Example 4: Creating tool calls
    println!("\n4. Creating tool calls");

    let read_call = ToolCall::builder("read_file")
        .arg_str("path", "/tmp/example.txt")
        .arg_str("encoding", "utf-8")
        .build();

    println!("   Created tool call: {}", read_call.tool);
    println!(
        "   Arguments: {}",
        serde_json::to_string_pretty(&read_call.arguments)?
    );

    let api_call = ToolCall::builder("api_call")
        .arg_str("url", "https://api.github.com/users/octocat")
        .arg_str("method", "GET")
        .arg("headers", json!({"Authorization": "Bearer token123"}))
        .build();

    println!("   Created tool call: {}", api_call.tool);
    println!(
        "   Arguments: {}",
        serde_json::to_string_pretty(&api_call.arguments)?
    );

    // Example 5: Working with tool results
    println!("\n5. Working with tool results");

    let success_result = thulp_core::ToolResult::success(json!({
        "content": "Hello, World!",
        "size": 13
    }))
    .with_duration(42);

    println!("   Success result:");
    println!("     Success: {}", success_result.is_success());
    println!(
        "     Data: {}",
        serde_json::to_string_pretty(&success_result.data)?
    );
    println!("     Duration: {:?}", success_result.duration_ms);

    let failure_result = thulp_core::ToolResult::failure("File not found: /tmp/nonexistent.txt");

    println!("   Failure result:");
    println!("     Success: {}", failure_result.is_success());
    println!("     Error: {:?}", failure_result.error);

    println!("\n🎉 Example completed successfully!");
    Ok(())
}
