# Thulp

**Execution Context Engineering Platform for AI Agents**

Thulp is a Rust-based toolkit for building AI agents with rich execution contexts. It provides abstractions for tool discovery, execution, workspace management, and integration with the Model Context Protocol (MCP).

## Overview

Thulp enables AI agents to interact with external tools and services through a unified interface. It handles the complexity of tool discovery, validation, execution, and result handling while providing extensibility through adapters and custom integrations.

### Key Features

- **Unified Tool Abstraction**: Consistent interface for defining, validating, and executing tools
- **MCP Integration**: First-class support for Model Context Protocol via `rs-utcp`
- **Type-Safe Parameters**: Strongly-typed parameter validation with JSON Schema support
- **Async by Design**: Built on `tokio` for efficient async execution
- **Extensible Adapters**: Connect to various tool providers and execution environments
- **Workspace Management**: Organize and manage execution contexts
- **Browser Automation**: Built-in browser interaction capabilities
- **Comprehensive Testing**: Edge-case coverage with property-based testing
- **Performance Monitoring**: Criterion-based benchmarks for critical paths

## Architecture

Thulp is organized as a Cargo workspace with the following crates:

### Core Crates

- **`thulp-core`**: Core types and traits (`Tool`, `Parameter`, `ToolDefinition`, `ToolCall`)
- **`thulp-mcp`**: MCP transport implementation using `rs-utcp`
- **`thulp-adapter`**: Adapter interfaces for connecting to different tool providers
- **`thulp-registry`**: Tool registration and discovery

### Feature Crates

- **`thulp-workspace`**: Workspace and execution context management
- **`thulp-skills`**: Pre-built skill definitions and utilities
- **`thulp-browser`**: Browser automation and interaction
- **`thulp-guidance`**: Agent guidance and decision-making primitives
- **`thulp-query`**: Query and search capabilities

### CLI

- **`thulp-cli`**: Command-line interface for tool execution and testing

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Cargo

### Installation

Add Thulp to your `Cargo.toml`:

```toml
[dependencies]
thulp-core = "0.1"
thulp-mcp = "0.1"
```

For MCP with Ares server support:

```toml
[dependencies]
thulp-mcp = { version = "0.1", features = ["ares"] }
```

### Quick Start

#### Defining a Tool

```rust
use thulp_core::{ToolDefinition, Parameter, ParameterType};

let tool = ToolDefinition::builder()
    .name("search")
    .description("Search for information")
    .parameter(
        Parameter::builder()
            .name("query")
            .description("Search query")
            .parameter_type(ParameterType::String)
            .required(true)
            .build()
    )
    .build();
```

#### Connecting to an MCP Server

```rust
use thulp_mcp::McpTransport;

// Connect via STDIO
let transport = McpTransport::stdio(
    "path/to/server",
    &["--arg1", "--arg2"],
    None
).await?;

// Or via HTTPS
let transport = McpTransport::https("https://mcp-server.example.com").await?;

// List available tools
let tools = transport.list_tools().await?;

// Execute a tool
let result = transport.call_tool("tool_name", json!({
    "param": "value"
})).await?;
```

## MCP Integration

Thulp provides comprehensive MCP support through the `thulp-mcp` crate:

- **STDIO Transport**: Spawn and communicate with local MCP servers
- **HTTPS Transport**: Connect to remote MCP servers over HTTPS
- **Tool Discovery**: Automatic conversion from MCP tool schemas to Thulp `ToolDefinition`
- **JSON-RPC Communication**: Low-level JSON-RPC 2.0 message handling via `rs-utcp`
- **Schema Parsing**: Automatic parsing of MCP JSON Schema to Thulp parameter types

### Supported MCP Features

- Tool listing (`tools/list`)
- Tool execution (`tools/call`)
- Connection management (connect/disconnect lifecycle)
- Error handling and status reporting

See [`crates/thulp-mcp/README.md`](crates/thulp-mcp/README.md) for detailed MCP usage.

## Feature Flags

### `thulp-mcp`

- **`ares`**: Enable Ares server integration (default: disabled)

## Development

### Building

```bash
# Build all crates
cargo build --workspace

# Build with MCP features
cargo build -p thulp-mcp --features ares

# Build in release mode
cargo build --workspace --release
```

### Examples

Thulp includes several examples demonstrating key functionality:

```bash
# Run tool definition example
cargo run --example tool_definition

# Run MCP integration example (requires MCP feature)
cargo run --example mcp --features mcp

# Run OpenAPI adapter example
cargo run --example adapter
```

See the [`examples/`](examples/) directory for detailed example code.

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p thulp-core

# Run tests with output
cargo test --workspace -- --nocapture
```

### Benchmarking

```bash
# Run benchmarks for thulp-core
cargo bench -p thulp-core

# Run specific benchmark
cargo bench -p thulp-core --bench tool_benchmarks
```

### Code Quality

```bash
# Run clippy
cargo clippy --workspace

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

## Project Status

**Version**: 0.1.0 (Early Development)

### Completed

- Core tool abstraction and validation
- MCP transport implementation (STDIO and HTTPS)
- Parameter type system with JSON Schema support
- Comprehensive test coverage (86 tests)
- Performance benchmarks for critical paths
- Edge-case testing for MCP integration

### In Progress

- Documentation improvements
- Additional adapter implementations
- Workspace management features

### Planned

- Resource management (MCP resources)
- Prompt template support (MCP prompts)
- Additional transport types
- Plugin system for custom tool providers

## Vendor Dependencies

This project currently vendors `ares-server` from `github.com/dirmacs/ares` (commit `cd9d697`) to include necessary patches not yet available on crates.io. See `VENDOR.md` for details on when and how to remove the vendor folder.

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test --workspace`
5. Run clippy: `cargo clippy --workspace`
6. Format code: `cargo fmt --all`
7. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- **rs-utcp**: MCP protocol implementation by rs-utcp contributors
- **Ares**: Server implementation from dirmacs/ares
- **Anthropic**: Model Context Protocol specification

## Links

- [Repository](https://github.com/dirmacs/thulp)
- [MCP Specification](https://modelcontextprotocol.io/)
- [rs-utcp](https://github.com/modelcontextprotocol/rs-utcp)
