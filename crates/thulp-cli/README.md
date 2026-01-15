# thulp-cli

Command-line interface for the Thulp execution context engineering platform.

## Overview

This crate provides a full-featured CLI for interacting with Thulp tools, MCP servers, and OpenAPI adapters. It supports multiple output formats (text, JSON) and shell completion generation.

## Features

- **Tool Management**: List, show, and validate tool definitions
- **MCP Integration**: Connect to MCP servers via STDIO or HTTP
- **OpenAPI Conversion**: Convert OpenAPI specs to tool definitions
- **Multiple Output Formats**: Human-readable text or JSON output
- **Shell Completions**: Generate completions for Bash, Zsh, Fish, PowerShell, and Elvish
- **Demo Mode**: Interactive demonstration of core functionality

## Installation

Install from crates.io:

```bash
cargo install thulp-cli
```

Or build from source:

```bash
cargo build --release -p thulp-cli
```

With MCP support:

```bash
cargo install thulp-cli --features mcp
```

## Usage

### List Tools

```bash
# Human-readable output
thulp tools list

# JSON output
thulp tools list --output json
```

### Show Tool Details

```bash
thulp tools show read_file
thulp tools show read_file --output json
```

### Validate Tool Arguments

```bash
# Validate with JSON arguments
thulp tools validate read_file --args '{"path": "/etc/hosts"}'

# JSON output for scripting
thulp tools validate read_file --args '{}' --output json
```

### MCP Server Connection (requires `mcp` feature)

```bash
# Connect via HTTP
thulp mcp connect-http myserver http://localhost:8080

# Connect via STDIO
thulp mcp connect-stdio myserver /path/to/mcp-server -- --verbose

# List tools from MCP server
thulp mcp list

# Call MCP tool
thulp mcp call search --args '{"query": "test"}'

# Check connection status
thulp mcp status
```

### OpenAPI Conversion

```bash
# Convert OpenAPI spec to tool definitions
thulp convert openapi spec.yaml
thulp convert openapi spec.json --output tools.yaml

# Show conversion examples
thulp convert examples
```

### Generate Shell Completions

```bash
# Bash
thulp completions bash > ~/.local/share/bash-completion/completions/thulp

# Zsh
thulp completions zsh > ~/.zfunc/_thulp

# Fish
thulp completions fish > ~/.config/fish/completions/thulp.fish

# PowerShell
thulp completions powershell >> $PROFILE

# Output to directory
thulp completions bash --dir ~/.local/share/bash-completion/completions
```

### Run Demo

```bash
# Interactive demo
thulp demo

# JSON output for testing
thulp demo --output json
```

### Validate Configuration

```bash
thulp validate config.yaml
```

## Output Formats

The CLI supports three output formats:

| Format | Flag | Description |
|--------|------|-------------|
| Text | `--output text` | Human-readable output (default) |
| JSON | `--output json` | Pretty-printed JSON |
| JSON Compact | `--output json-compact` | Single-line JSON |

## Commands

| Command | Description |
|---------|-------------|
| `tools list` | List all available tools |
| `tools show <name>` | Show details of a specific tool |
| `tools validate <name>` | Validate tool arguments |
| `mcp connect-http` | Connect to MCP server via HTTP |
| `mcp connect-stdio` | Connect to MCP server via STDIO |
| `mcp list` | List tools from MCP server |
| `mcp call` | Call a tool on the MCP server |
| `mcp status` | Show connection status |
| `convert openapi` | Convert OpenAPI spec to tools |
| `convert examples` | Show conversion examples |
| `demo` | Run interactive demo |
| `validate` | Validate configuration files |
| `completions` | Generate shell completions |

## Feature Flags

| Flag | Description |
|------|-------------|
| `mcp` | Enable MCP server integration |

## Testing

```bash
cargo test -p thulp-cli
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
