<p align="center">
  <img src="docs/static/img/thulp-logo.svg" width="128" alt="thulp">
</p>

<h1 align="center">Thulp</h1>

<p align="center">
  Execution context engineering for AI agents.<br>
  One interface for tool discovery, validation, execution, and multi-step workflows.
</p>

<p align="center">
  <a href="https://crates.io/crates/thulp-core"><img src="https://img.shields.io/crates/v/thulp-core.svg" alt="crates.io"></a>
  <a href="https://github.com/dirmacs/thulp/actions/workflows/ci.yml"><img src="https://github.com/dirmacs/thulp/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://docs.rs/thulp-core"><img src="https://docs.rs/thulp-core/badge.svg" alt="docs.rs"></a>
  <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-yellow.svg" alt="License">
</p>

---

**Thulp** gives AI agents a unified way to discover, validate, and execute tools — whether they're local functions, [MCP](https://modelcontextprotocol.io/) servers, or OpenAPI endpoints. It handles parameter validation, multi-step skill workflows, session persistence, and query-based tool filtering. Built on [rs-utcp](https://github.com/universal-tool-calling-protocol/rs-utcp) for protocol transport.

Built by [DIRMACS](https://dirmacs.com).

## Install

```bash
cargo install thulp-cli
```

```toml
# Or as a library
[dependencies]
thulp-core = "0.3"
thulp-mcp = "0.3"
thulp-skills = "0.3"
```

## Why Thulp?

Every AI agent needs tools. But each agent framework invents its own tool format, validation, and execution layer. Thulp standardizes this:

- **One tool definition** works across local functions, MCP servers, and OpenAPI specs
- **Type-safe parameters** with JSON Schema validation — catch errors before execution
- **Skill workflows** compose multi-step tool chains with `{{variable}}` interpolation, timeout/retry, and hooks
- **Query DSL** finds tools by name, parameter count, or tags — `name:search and min:2`
- **Session persistence** tracks turns, enforces limits, stores results to disk

No runtime overhead. No framework lock-in. Pure Rust async.

## Workspace (11 crates)

| Crate | What | Tests |
|-------|------|-------|
| **thulp-core** | Types, traits, parameter validation, JSON Schema | 70 |
| **thulp-mcp** | MCP transport (STDIO/HTTP), tools, resources, prompts | 39 |
| **thulp-skills** | Multi-step workflows, executor trait, hooks, retry | 54 |
| **thulp-skill-files** | SKILL.md parsing, YAML frontmatter, scope priority | 23 |
| **thulp-query** | Query DSL with nom parser, wildcards, boolean ops | 19 |
| **thulp-workspace** | Sessions, turn counting, file persistence | 6 |
| **thulp-adapter** | OpenAPI v2/v3 to tool conversion (JSON + YAML) | 10 |
| **thulp-registry** | Async thread-safe tool registry with tagging | 8 |
| **thulp-browser** | Web fetching, HTML parsing, optional CDP | 7 |
| **thulp-guidance** | Template rendering, LLM guidance primitives | 6 |
| **thulp-cli** | CLI with JSON output, shell completions, init/run/skill/config commands | 32 |

## Quick Start

### Define a tool

```rust
use thulp_core::{ToolDefinition, Parameter, ParameterType};

let tool = ToolDefinition::builder("search")
    .description("Search for information")
    .parameter(
        Parameter::builder("query")
            .param_type(ParameterType::String)
            .required(true)
            .build()
    )
    .build();
```

### Connect to an MCP server

```rust
use thulp_mcp::McpClient;

let client = McpClient::connect_stdio("server", "mcp-server", None).await?;
let tools = client.list_tools().await?;
let result = client.call(&ToolCall::builder("search")
    .arg_str("query", "rust async")
    .build()).await?;
```

### Compose a skill workflow

```rust
use thulp_skills::{Skill, SkillStep};

let skill = Skill::new("search_and_summarize", "Search and summarize")
    .with_input("query")
    .with_step(SkillStep {
        name: "search".into(),
        tool: "web_search".into(),
        arguments: json!({"query": "{{query}}"}),
        continue_on_error: false,
    })
    .with_step(SkillStep {
        name: "summarize".into(),
        tool: "summarize".into(),
        arguments: json!({"text": "{{search.results}}"}),
        continue_on_error: false,
    });
```

### Query DSL

```rust
use thulp_query::{parse_query, QueryBuilder};

let criteria = parse_query("name:search and min:2")?;
let matches: Vec<_> = tools.iter().filter(|t| criteria.matches(t)).collect();
```

## CLI

```bash
thulp tools list                                      # list available tools
thulp tools list --output json                        # JSON output
thulp tools show <name>                               # tool details
thulp tools validate <name> --args '{"q": "rust"}'    # validate arguments
thulp convert openapi spec.yaml --output tools.yaml   # OpenAPI conversion
thulp demo                                            # interactive demo
thulp completions bash > ~/.local/share/bash-completion/completions/thulp
```

## Architecture

```
thulp/
  crates/
    thulp-core/        # types, traits, validation (zero dependencies on other thulp crates)
    thulp-mcp/         # MCP client (rs-utcp, STDIO + HTTP transport)
    thulp-skills/      # workflow engine (executor, hooks, retry, timeout)
    thulp-skill-files/ # SKILL.md parser (YAML frontmatter, scope priority)
    thulp-query/       # DSL parser (nom, wildcards, boolean operators)
    thulp-workspace/   # sessions, persistence, turn counting
    thulp-adapter/     # OpenAPI v2/v3 → ToolDefinition converter
    thulp-registry/    # async thread-safe tool registry with tags
    thulp-browser/     # web fetching, HTML parsing, optional CDP
    thulp-guidance/    # template rendering, LLM guidance
    thulp-cli/         # clap CLI with JSON output + shell completions
  examples/            # 6 runnable examples
```

### Feature flags

| Crate | Flag | Description |
|-------|------|-------------|
| thulp-mcp | `ares` | Ares server integration |
| thulp-browser | `cdp` | Chrome DevTools Protocol support |
| thulp-skills | `mcp` | MCP support in skill execution |

## Development

```bash
cargo build --workspace                     # build all
cargo test --workspace
cargo clippy --workspace -- -D warnings     # lint (currently clean)
cargo bench -p thulp-core --bench tool_benchmarks  # benchmarks
```

## Ecosystem

| Project | What |
|---------|------|
| [pawan](https://github.com/dirmacs/pawan) | CLI coding agent — uses thulp for tool abstraction |
| [ares](https://github.com/dirmacs/ares) | Agentic retrieval-enhanced server |
| [eruka](https://eruka.dirmacs.com) | Context intelligence engine |
| [daedra](https://dirmacs.github.io/daedra) | Self-contained web search MCP server |
| [doltares](https://github.com/dirmacs/doltares) | Orchestration daemon (DAG workflows) |

## License

MIT OR Apache-2.0

## Links

- [Documentation](https://dirmacs.github.io/thulp)
- [API Reference](https://docs.rs/thulp-core)
- [crates.io](https://crates.io/search?q=thulp)
- [MCP Specification](https://modelcontextprotocol.io/)
- [rs-utcp](https://github.com/universal-tool-calling-protocol/rs-utcp)
