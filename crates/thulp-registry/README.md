# thulp-registry

Async thread-safe **metadata catalog** for `thulp_core::ToolDefinition`.

## Overview

This crate provides a registry for managing tool *definitions* (the metadata
the LLM sees) with support for dynamic registration, tagging, and discovery.
The registry is designed for concurrent access in async environments.

## Intended Use

`thulp-registry` is a metadata-only store. It holds `ToolDefinition` values —
the JSON-schema-shaped descriptions an LLM consumes — plus tags for grouping
and discovery. It is **not** an execution runtime: there is no `Tool` trait
and no `execute()` method.

Use this crate when you need to:

- Publish or serialize a catalog of tools (MCP discovery, skill manifests,
  documentation generation)
- Filter or tag definitions before exposing them to an LLM
- Maintain a cross-process / cross-service tool catalog where the actual
  executors live elsewhere (e.g., in a remote MCP server)

If instead you need an **in-process executable registry** that can dispatch
`(name, args)` to a Rust implementation, you want a different abstraction:
typically a `HashMap<String, Arc<dyn Tool>>` where `Tool` has
`async fn execute(&self, args: Value) -> Result<Value>`. Two existing
examples in the dirmacs stack:

- `pawan::tools::ToolRegistry` — pawan's in-process executable registry
  with 3-tier visibility (Core / Standard / Extended) and scored
  `select_for_query()` for dynamic tool selection
- `ares::tools::registry::ToolRegistry` — ares-server's executable registry
  used by the agent loop

Both wrappers keep their own `Arc<dyn Tool>` storage for execution and use
`thulp-core::ToolDefinition` for the metadata side. They also integrate
`thulp-query` for DSL-driven filtering. The split is intentional: separating
metadata from execution lets the same definitions be published, queried, and
shipped to LLMs without dragging an execution runtime into every consumer.

## Features

- **Dynamic Registration**: Register and unregister tools at runtime
- **Thread-Safe**: Concurrent access via `RwLock` for safe multi-threaded use
- **Tool Discovery**: Find tools by name or tag
- **Tagging System**: Organize tools with custom tags
- **Batch Operations**: Register multiple tools at once
- **Async Design**: Built on tokio for async operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-registry = "0.2"
```

## Usage

### Creating a Registry

```rust
use thulp_registry::ToolRegistry;

let registry = ToolRegistry::new();
```

### Registering Tools

```rust
use thulp_registry::ToolRegistry;
use thulp_core::{ToolDefinition, Parameter, ParameterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ToolRegistry::new();

    // Create a tool definition
    let tool = ToolDefinition::builder("read_file")
        .description("Read file contents")
        .parameter(
            Parameter::builder("path")
                .param_type(ParameterType::String)
                .required(true)
                .build()
        )
        .build();

    // Register the tool
    registry.register(tool).await?;

    // Check if registered
    assert!(registry.contains("read_file").await);

    Ok(())
}
```

### Registering Multiple Tools

```rust
use thulp_registry::ToolRegistry;
use thulp_core::{ToolDefinition, Parameter, ParameterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ToolRegistry::new();

    let tools = vec![
        ToolDefinition::builder("read_file")
            .description("Read file")
            .build(),
        ToolDefinition::builder("write_file")
            .description("Write file")
            .build(),
        ToolDefinition::builder("delete_file")
            .description("Delete file")
            .build(),
    ];

    registry.register_many(tools).await?;

    assert_eq!(registry.count().await, 3);

    Ok(())
}
```

### Retrieving Tools

```rust
use thulp_registry::ToolRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ToolRegistry::new();
    // ... register tools ...

    // Get specific tool
    if let Some(tool) = registry.get("read_file").await? {
        println!("Found tool: {}", tool.name);
    }

    // List all tools
    let tools = registry.list().await?;
    for tool in tools {
        println!("Tool: {}", tool.name);
    }

    Ok(())
}
```

### Tagging Tools

```rust
use thulp_registry::ToolRegistry;
use thulp_core::ToolDefinition;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ToolRegistry::new();

    // Register tools
    registry.register(ToolDefinition::builder("read_file").build()).await?;
    registry.register(ToolDefinition::builder("write_file").build()).await?;
    registry.register(ToolDefinition::builder("http_get").build()).await?;

    // Tag tools by category
    registry.tag("read_file", "filesystem").await?;
    registry.tag("write_file", "filesystem").await?;
    registry.tag("http_get", "network").await?;

    // Find tools by tag
    let fs_tools = registry.find_by_tag("filesystem").await?;
    assert_eq!(fs_tools.len(), 2);

    let net_tools = registry.find_by_tag("network").await?;
    assert_eq!(net_tools.len(), 1);

    Ok(())
}
```

### Unregistering Tools

```rust
use thulp_registry::ToolRegistry;
use thulp_core::ToolDefinition;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ToolRegistry::new();

    registry.register(ToolDefinition::builder("temp_tool").build()).await?;
    assert!(registry.contains("temp_tool").await);

    // Remove the tool
    let removed = registry.unregister("temp_tool").await?;
    assert!(removed.is_some());
    assert!(!registry.contains("temp_tool").await);

    Ok(())
}
```

### Clearing the Registry

```rust
use thulp_registry::ToolRegistry;

#[tokio::main]
async fn main() {
    let registry = ToolRegistry::new();
    // ... register tools ...

    // Clear all tools
    registry.clear().await;
    assert_eq!(registry.count().await, 0);
}
```

## Thread Safety

The registry uses `tokio::sync::RwLock` internally, allowing multiple readers or a single writer at any time. All operations are safe to use from multiple async tasks concurrently.

```rust
use thulp_registry::ToolRegistry;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let registry = Arc::new(ToolRegistry::new());

    // Spawn multiple tasks that access the registry
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let reg = registry.clone();
            tokio::spawn(async move {
                // Safe concurrent access
                let count = reg.count().await;
                println!("Task {} sees {} tools", i, count);
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Testing

```bash
cargo test -p thulp-registry
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
