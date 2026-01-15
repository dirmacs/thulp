# Thulp - Execution Context Engineering Platform

## Vision

Thulp is a Rust implementation of an execution context engineering platform that sits between AI agents and APIs. It transforms exploratory interactions into reusable, deterministic workflows, dramatically reducing token consumption and execution time for repeated operations.

**Name Origin**: "Thulp" - a playful Dutch-inspired name suggesting "help" with character.

## Problem Statement

AI agents interacting with APIs face several challenges:

1. **Redundant Discovery**: Each interaction requires re-discovering API capabilities
2. **Token Waste**: Exploratory calls consume tokens without producing value
3. **Non-Deterministic**: Same queries may produce different execution paths
4. **No Reusability**: Successful workflows cannot be easily captured and replayed
5. **Integration Friction**: Each API requires custom integration work

## Solution

Thulp provides:

- **MCP Protocol Support**: Native Model Context Protocol client for tool execution
- **Adapter Framework**: Transform any REST/GraphQL/gRPC API into MCP-compatible tools
- **Skills System**: Parameterized, reusable workflows with dependency management
- **Flow Export**: Convert explorations into deterministic shell scripts
- **Embedded Guidance**: Self-contained documentation and how-to guides
- **Query Engine**: 98% jq-compatible query language for response transformation

## Performance Impact

Performance benchmarks:
- **First run**: ~30 seconds, ~8,400 tokens
- **Subsequent runs**: ~2 seconds, ~250 tokens
- **Savings**: 97% token reduction, 93% time reduction

## Target Users

1. **AI Agent Developers**: Building agents that interact with external APIs
2. **DevOps Engineers**: Automating API interactions in CI/CD pipelines
3. **Platform Teams**: Creating standardized API access patterns
4. **Individual Developers**: Exploring and automating API workflows

## Project Context

Thulp is developed under the **Dirmacs** organization (github.com/dirmacs), which focuses on:
- **DIRMACS**: "Democratizing Innovation through Resource Mobility and Access Creation for Sustainability"

Related Dirmacs projects:
- **ares**: Agentic chatbot server
- **daedra**: MCP web search tools
- **lancor**: llama.cpp client library
- **dcrm**: CRM system

## Core Principles

1. **Determinism**: Same inputs produce same outputs
2. **Composability**: Small tools combine into complex workflows
3. **Observability**: Full visibility into execution paths
4. **Portability**: Export to standard formats (shell scripts, etc.)
5. **Performance**: Minimize redundant work through caching and skills

## Key Differentiators

| Feature | Traditional API Clients | Thulp |
|---------|------------------------|-------|
| Discovery | Manual/per-request | Cached, skill-based |
| Workflows | Code-based | Declarative YAML |
| Token Usage | Linear with complexity | Constant after first run |
| Reusability | Copy-paste | Skills with parameters |
| Export | None | Shell scripts, flows |

## Success Metrics

- **Token Efficiency**: 90%+ reduction in tokens for repeated workflows
- **Time Efficiency**: 90%+ reduction in execution time
- **Adoption**: Wide adoption potential
- **Extensibility**: Support for new API types without core changes

## License

MIT/Apache-2.0 dual license (standard Rust ecosystem licensing)

## Links

- **Dirmacs Organization**: <https://github.com/dirmacs>
- **rs-utcp** (core dependency): <https://github.com/universal-tool-calling-protocol/rs-utcp>
