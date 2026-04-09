# thulp — Agent Guidelines

## What This Is

Thulp handles tool discovery, validation, and execution for AI agents. It provides a unified abstraction over local Rust functions, MCP servers, and OpenAPI endpoints. Skills chain tools into reusable workflows.

## For Agents

- Run `cargo test --workspace` — 211 tests must pass
- thulp-core is the foundation — changes there affect everything
- Query DSL uses nom parser — test with `cargo test -p thulp-query`
- SKILL.md parsing is in thulp-skill-files — validate against real skill files
- MCP transport is in thulp-mcp — test with mock servers, not live ones
- Don't add deps between core and other crates — core stays independent
