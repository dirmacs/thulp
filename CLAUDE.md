# thulp

Execution context engineering for AI agents. Tool discovery, validation, execution, multi-step skill workflows. 11-crate Rust workspace.

## Build & Test

```bash
cargo build --workspace
cargo test --workspace          # 211 tests
cargo clippy --workspace -- -D warnings
```

## Architecture

- `thulp-core` (70 tests) — types, traits, validation, JSON Schema
- `thulp-mcp` (39 tests) — MCP transport client
- `thulp-skills` (54 tests) — workflow engine, skill execution
- `thulp-skill-files` (23 tests) — SKILL.md parsing
- `thulp-query` (19 tests) — DSL parser (nom-based)
- `thulp-cli` (32 tests) — clap CLI
- `thulp-openapi`, `thulp-cdp`, `thulp-session`, `thulp-ares`, `thulp-registry`

## Conventions

- Git author: `bkataru <baalateja.k@gmail.com>`
- thulp-core is independent — zero deps on other thulp crates
- Feature flags: `ares`, `cdp`, `mcp` for optional integrations
- nom parser for query DSL
- async-trait for trait objects
