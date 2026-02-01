# thulp

[![CI](https://github.com/dirmacs/thulp/actions/workflows/ci.yml/badge.svg)](https://github.com/dirmacs/thulp/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/thulp-core.svg)](https://crates.io/crates/thulp-core)
[![docs.rs](https://docs.rs/thulp-core/badge.svg)](https://docs.rs/thulp-core)
[![License](https://img.shields.io/crates/l/thulp-core.svg)](LICENSE-MIT)

## Execution Context Engineering Platform for AI Agents

Thulp is a Rust-based toolkit for building AI agents with rich execution
contexts. It provides abstractions for tool discovery, execution, workspace
management, and integration with the Model Context Protocol (MCP) via the
[UTCP](https://github.com/universal-tool-calling-protocol/rs-utcp)
(Universal Tool Calling Protocol) implementation.

## Overview

Thulp enables AI agents to interact with external tools and services through a
unified interface. It handles the complexity of tool discovery, validation,
execution, and result handling while providing extensibility through adapters
and custom integrations.

### Key Features

- **Unified Tool Abstraction**: Consistent interface for defining, validating,
  and executing tools
- **MCP Integration**: Full Model Context Protocol support via `rs-utcp` UTCP
  implementation (tools, resources, prompts)
- **Type-Safe Parameters**: Strongly-typed parameter validation with JSON
  Schema support
- **Query DSL**: Powerful query language for filtering and searching tools
- **Skill Workflows**: Compose multi-step tool workflows with variable interpolation
- **SkillExecutor Trait**: Pluggable execution strategies with timeout/retry support
- **Execution Hooks**: Lifecycle callbacks for observability (before/after skill/step, on_error, on_retry, on_timeout)
- **Session Management**: Persistent sessions with turn counting, configurable limits, and file-based storage
- **SKILL.md Parsing**: Load skills from markdown files with YAML frontmatter and scope-based priority
- **Async by Design**: Built on `tokio` for efficient async execution
- **CLI with Shell Completions**: Full-featured CLI with JSON output and completions
- **Browser Automation**: Web fetching and optional CDP support
- **Comprehensive Testing**: 183 tests with edge-case coverage

## Architecture

Thulp is organized as a Cargo workspace with 11 crates:

### Core Crates

| Crate              | Description                                       |
| ------------------ | ------------------------------------------------- |
| **thulp-core**     | Core types and traits (`ToolDefinition`, etc.)    |
| **thulp-mcp**      | MCP transport (STDIO/HTTP, tools, resources)      |
| **thulp-adapter**  | OpenAPI v2/v3 to tool definition conversion       |
| **thulp-registry** | Async thread-safe tool registry with tagging      |

### Feature Crates

| Crate                | Description                                       |
| -------------------- | ------------------------------------------------- |
| **thulp-query**      | Query DSL for searching and filtering tools       |
| **thulp-skills**     | Multi-step workflow composition and execution     |
| **thulp-skill-files**| SKILL.md file parsing with YAML frontmatter       |
| **thulp-workspace**  | Workspace, session management, and persistence    |
| **thulp-browser**    | Web fetching, HTML parsing, optional CDP support  |
| **thulp-guidance**   | Template rendering and LLM guidance primitives    |

### CLI

| Crate         | Description                                        |
| ------------- | -------------------------------------------------- |
| **thulp-cli** | CLI with JSON output and shell completions         |

## Installation

### Install CLI

```bash
cargo install thulp-cli
```

### Add as Dependency

```toml
[dependencies]
thulp-core = "0.2"
thulp-mcp = "0.2"
thulp-query = "0.2"
```

For MCP with Ares server support:

```toml
[dependencies]
thulp-mcp = { version = "0.2", features = ["ares"] }
```

## Quick Start

### Defining a Tool

```rust
use thulp_core::{ToolDefinition, Parameter, ParameterType};

let tool = ToolDefinition::builder("search")
    .description("Search for information")
    .parameter(
        Parameter::builder("query")
            .description("Search query")
            .param_type(ParameterType::String)
            .required(true)
            .build()
    )
    .build();
```

### Connecting to an MCP Server

```rust
use thulp_mcp::McpClient;

// Connect via HTTP
let client = McpClient::connect_http("server", "http://localhost:8080".to_string()).await?;

// Or via STDIO
let client = McpClient::connect_stdio(
    "server",
    "path/to/mcp-server".to_string(),
    None
).await?;

// List available tools
let tools = client.list_tools().await?;

// Execute a tool
let result = client.call(&ToolCall::builder("tool_name")
    .arg_str("param", "value")
    .build()).await?;
```

### Query DSL

```rust
use thulp_query::{parse_query, QueryBuilder};

// Parse natural language query
let criteria = parse_query("name:search and min:2")?;
let matches: Vec<_> = tools.iter().filter(|t| criteria.matches(t)).collect();

// Or use the builder
let query = QueryBuilder::new()
    .name("file")
    .min_parameters(1)
    .build();
let results = query.execute(&tools);
```

### Skill Workflows

```rust
use thulp_skills::{Skill, SkillStep};

let skill = Skill::new("search_and_summarize", "Search and summarize results")
    .with_input("query")
    .with_step(SkillStep {
        name: "search".to_string(),
        tool: "web_search".to_string(),
        arguments: json!({"query": "{{query}}"}),
        continue_on_error: false,
    })
    .with_step(SkillStep {
        name: "summarize".to_string(),
        tool: "summarize".to_string(),
        arguments: json!({"text": "{{search.results}}"}),
        continue_on_error: false,
    });
```

## CLI Usage

```bash
# List tools
thulp tools list
thulp tools list --output json

# Show tool details
thulp tools show <tool-name>

# Validate tool arguments
thulp tools validate <tool-name> --args '{"param": "value"}'

# Convert OpenAPI spec to tools
thulp convert openapi spec.yaml --output tools.yaml

# Run demo
thulp demo

# Generate shell completions
thulp completions bash > ~/.local/share/bash-completion/completions/thulp
thulp completions powershell >> $PROFILE
```

## Examples

Thulp includes 6 comprehensive examples:

```bash
# Core tool types
cargo run --example tool_definition

# OpenAPI conversion
cargo run --example adapter

# MCP integration
cargo run --example mcp --features mcp

# Query DSL
cargo run --example query

# Skill workflows
cargo run --example skills

# Async registry
cargo run --example registry
```

## Development

### Building

```bash
# Build all crates
cargo build --workspace

# Build with CDP feature
cargo build -p thulp-browser --features cdp

# Build in release mode
cargo build --workspace --release
```

### Testing

```bash
# Run all tests (183 tests)
cargo test --workspace

# Run with verbose output
cargo test --workspace -- --nocapture
```

### Benchmarking

```bash
cargo bench -p thulp-core --bench tool_benchmarks
```

### Code Quality

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Project Status

**Version**: 0.2.0

### Complete Features

- Core tool abstraction and validation
- MCP transport (STDIO and HTTP) with tools, resources, prompts
- Parameter type system with JSON Schema support
- Query DSL with wildcards and boolean operators
- Skill workflows with variable interpolation
- SkillExecutor trait with pluggable execution strategies
- DefaultSkillExecutor with timeout and retry support
- ExecutionHooks for lifecycle callbacks and observability
- Session management with turn counting and persistence
- SessionManager for file-based session storage
- SKILL.md file parsing with YAML frontmatter (thulp-skill-files)
- SkillLoader with scope-based priority (Global/Workspace/Project)
- OpenAPI v2/v3 conversion (JSON and YAML)
- CLI with JSON output and shell completions
- Async thread-safe tool registry with tagging
- Browser web fetching and HTML parsing
- 183 tests with comprehensive coverage

### Feature Flags

| Crate          | Flag   | Description                       |
| -------------- | ------ | --------------------------------- |
| thulp-mcp      | `ares` | Ares server integration           |
| thulp-browser  | `cdp`  | Chrome DevTools Protocol support  |
| thulp-examples | `mcp`  | MCP example                       |

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test --workspace`
5. Run clippy: `cargo clippy --workspace -- -D warnings`
6. Format code: `cargo fmt --all`
7. Submit a pull request

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Links

- [Repository](https://github.com/dirmacs/thulp)
- [Crates.io](https://crates.io/search?q=thulp)
- [Documentation](https://docs.rs/thulp-core)
- [MCP Specification](https://modelcontextprotocol.io/)
- [rs-utcp](https://github.com/universal-tool-calling-protocol/rs-utcp)

## Acknowledgments

- **rs-utcp**: UTCP protocol implementation (includes MCP transport)
- **Anthropic**: Model Context Protocol specification
- **UTCP**: Universal Tool Calling Protocol
