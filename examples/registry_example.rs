//! Registry example - async thread-safe tool registry with tagging
//!
//! Run: `cargo run --example registry`

use std::sync::Arc;
use thulp_core::{Parameter, ToolDefinition};
use thulp_registry::ToolRegistry;

#[tokio::main]
async fn main() {
    println!("=== Thulp Registry Example ===\n");

    // Create a thread-safe registry (uses RwLock internally)
    let registry = Arc::new(ToolRegistry::new());

    // Register tools
    let weather_tool = ToolDefinition::builder("get_weather")
        .description("Get current weather for a location")
        .parameter(Parameter::required_string("location"))
        .build();

    let file_tool = ToolDefinition::builder("read_file")
        .description("Read file contents")
        .parameter(Parameter::required_string("path"))
        .build();

    let search_tool = ToolDefinition::builder("search_web")
        .description("Search the web")
        .parameter(Parameter::required_string("query"))
        .parameter(
            Parameter::builder("limit")
                .param_type(thulp_core::ParameterType::Integer)
                .description("Max results")
                .build(),
        )
        .build();

    // All operations are async
    registry.register(weather_tool).await.unwrap();
    registry.register(file_tool).await.unwrap();
    registry.register(search_tool).await.unwrap();

    println!("Registered {} tools\n", registry.count().await);

    // List all tools
    println!("All tools:");
    for tool in registry.list().await.unwrap() {
        println!(
            "  {} - {} ({} params)",
            tool.name,
            tool.description,
            tool.parameters.len()
        );
    }
    println!();

    // Get tool by name
    println!("Get by name 'get_weather':");
    if let Some(tool) = registry.get("get_weather").await.unwrap() {
        println!("  Found: {}", tool.description);
    }
    println!();

    // Tag tools for discovery
    println!("Tagging tools for discovery...");
    registry.tag("get_weather", "api").await.unwrap();
    registry.tag("search_web", "api").await.unwrap();
    registry.tag("read_file", "filesystem").await.unwrap();

    // Find by tag
    println!("\nFind by tag 'api':");
    for tool in registry.find_by_tag("api").await.unwrap() {
        println!("  - {}", tool.name);
    }

    println!("\nFind by tag 'filesystem':");
    for tool in registry.find_by_tag("filesystem").await.unwrap() {
        println!("  - {}", tool.name);
    }
    println!();

    // Demonstrate thread-safety with concurrent access
    println!("Thread-safety demo:");
    let registry_clone = Arc::clone(&registry);

    let handle = tokio::spawn(async move {
        let async_tool = ToolDefinition::builder("async_task")
            .description("Added from another task")
            .build();
        registry_clone.register(async_tool).await.unwrap();
        println!("  Added tool from async task");
    });

    handle.await.unwrap();
    println!("  Registry now has {} tools", registry.count().await);

    // Check if tool exists
    println!(
        "\n  Contains 'async_task': {}",
        registry.contains("async_task").await
    );

    // Unregister
    registry.unregister("async_task").await.unwrap();
    println!("  After unregister: {} tools", registry.count().await);

    // Clear all
    registry.clear().await;
    println!("\n  After clear: {} tools", registry.count().await);
}
