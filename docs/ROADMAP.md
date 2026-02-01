# Thulp Development Roadmap

## Overview

This roadmap outlines the phased development of Thulp with TDD/BDD methodology. The integration of **rs-utcp** significantly reduces development time for Phase 2 (MCP and Adapters).

**Total Estimated Duration**: 14-16 weeks (reduced from original 19+ weeks due to rs-utcp)

## Development Principles

1. **Test-First**: Write tests before implementation
2. **Red-Green-Refactor**: TDD cycle for all features
3. **BDD for User-Facing Features**: Cucumber/Gherkin scenarios
4. **Incremental Delivery**: Each phase produces usable functionality
5. **Documentation as Code**: Keep docs updated with implementation

---

## Phase 1: Foundation (Weeks 1-3)

### Goals
- Establish project structure
- Implement core types and traits
- Set up testing infrastructure

### Deliverables

#### Week 1: Project Setup
- [ ] Initialize Cargo workspace
- [ ] Set up CI/CD (GitHub Actions)
- [ ] Configure test infrastructure (unit, integration, BDD)
- [ ] Create `thulp-core` crate skeleton

#### Week 2: Core Types
- [ ] `ToolDefinition`, `ToolCall`, `ToolResult` types
- [ ] `Error` types with `thiserror`
- [ ] Core traits: `Tool`, `Transport`
- [ ] Serialization with `serde`

#### Week 3: Query Engine Foundation
- [ ] `thulp-query` crate setup
- [ ] Parser foundation with `nom`
- [ ] Basic operations: `.`, `.field`, `.[n]`
- [ ] Pipe operator `|`

### Testing Focus
```
Unit tests: 90%+ coverage for thulp-core
Unit tests: Parser tests for thulp-query
```

### Exit Criteria
- [ ] All core types compile and serialize correctly
- [ ] Basic query expressions parse and execute
- [ ] CI pipeline green with >80% coverage

---

## Phase 2: MCP & Adapters (Weeks 4-6)

### Goals
- Integrate rs-utcp for MCP protocol
- Wrap rs-utcp's OpenAPI converter
- Establish abstraction layer

### Deliverables

#### Week 4: MCP Client (rs-utcp Integration)
- [ ] `thulp-mcp` crate with rs-utcp dependency
- [ ] `McpClient` wrapper around `rs_utcp::transports::mcp`
- [ ] STDIO transport: `connect_stdio()`
- [ ] Tool discovery: `list_tools()`
- [ ] Tool execution: `call_tool()`

#### Week 5: MCP Client (Continued)
- [ ] SSE transport: `connect_sse()`
- [ ] Resource access: `list_resources()`, `read_resource()`
- [ ] Connection lifecycle management
- [ ] Error handling and reconnection

#### Week 6: Adapter Framework
- [ ] `thulp-adapter` crate with rs-utcp dependency
- [ ] `AdapterGenerator` wrapper around `rs_utcp::openapi::OpenApiConverter`
- [ ] OpenAPI 3.x support
- [ ] Auth configuration extraction
- [ ] Adapter serialization to YAML

### rs-utcp Integration Points

```rust
// thulp-mcp/Cargo.toml
[dependencies]
rs-utcp = { version = "0.3", default-features = false, features = ["mcp"] }

// thulp-adapter/Cargo.toml  
[dependencies]
rs-utcp = { version = "0.3", default-features = false, features = ["http"] }
```

### Testing Focus
```
Integration tests: Real MCP server connections (filesystem server)
Unit tests: Adapter generation from sample OpenAPI specs
BDD: Connection and tool call scenarios
```

### BDD Scenarios

```gherkin
Feature: MCP Connection
  Scenario: Connect to filesystem MCP server
    Given the filesystem MCP server is available
    When I connect via STDIO
    Then I should see available tools including "read_file"

Feature: Adapter Generation
  Scenario: Generate adapter from OpenAPI
    Given an OpenAPI spec for a REST API
    When I generate an adapter
    Then I should get valid tool definitions
```

### Exit Criteria
- [ ] Can connect to real MCP servers (stdio and sse)
- [ ] Can execute tool calls and receive results
- [ ] Can generate adapters from OpenAPI specs
- [ ] Integration tests pass with real servers

---

## Phase 3: Workspace & Skills (Weeks 7-10)

### Goals
- Project structure management
- Skill definition and execution
- Query engine completion

### Deliverables

#### Week 7: Workspace Management
- [x] `thulp-workspace` crate
- [x] `.thulp/` directory structure
- [x] `config.yaml` parsing
- [x] Server configuration loading
- [x] Session management basics
- [x] SessionManager with file-based persistence
- [x] Session turn counting (`turn_count()`)
- [x] SessionConfig with max_turns, max_entries, max_duration

#### Week 8: Query Engine Completion
- [ ] Array operations: `map()`, `select()`, `sort_by()`
- [ ] Object construction: `{key: .value}`
- [ ] Conditionals: `if-then-else`, `//`
- [ ] String functions: `split()`, `join()`, `test()`
- [ ] 98% jq compatibility target

#### Week 9: Skills System Core
- [x] `thulp-skills` crate
- [x] Skill YAML parsing
- [x] Parameter validation
- [x] Step execution engine
- [x] Variable interpolation with `tera`
- [x] SkillExecutor trait (pluggable execution strategies)
- [x] DefaultSkillExecutor implementation
- [x] ExecutionHooks trait for lifecycle callbacks

#### Week 10: Skills System Advanced
- [x] Query step integration
- [x] Dependency checking
- [x] Execution context management
- [x] Dry-run mode
- [x] Basic flow export (shell)
- [x] Per-step and per-skill timeout support
- [x] Retry logic with configurable attempts
- [x] `thulp-skill-files` crate for SKILL.md parsing
- [x] SkillLoader with scope-based priority (Global/Workspace/Project)
- [x] SkillPreprocessor for variable substitution

### Testing Focus
```
Unit tests: Workspace config parsing
Unit tests: Query operations (comprehensive jq compatibility)
Integration tests: Skill execution with mock MCP
BDD: Full skill execution scenarios
```

### BDD Scenarios

```gherkin
Feature: Workspace Management
  Scenario: Initialize new workspace
    Given I am in an empty directory
    When I run "thulp init"
    Then a .thulp directory should be created
    And it should contain a config.yaml

Feature: Skill Execution
  Scenario: Execute multi-step skill
    Given a skill with 3 steps
    And all required servers are configured
    When I execute the skill
    Then all steps should complete in order
    And I should receive the final output
```

### Exit Criteria
- [x] Workspace initialization and loading works
- [x] Query engine passes jq compatibility tests
- [x] Skills execute with real MCP servers
- [x] Flow export produces valid shell scripts
- [x] SkillExecutor trait allows pluggable execution strategies
- [x] Session management with turn counting and persistence
- [x] SKILL.md file parsing with YAML frontmatter

---

## Phase 4: CLI & Integration (Weeks 11-13)

### Goals
- Full CLI implementation
- End-to-end functionality
- Documentation

### Deliverables

#### Week 11: CLI Core
- [ ] `thulp-cli` crate with `clap`
- [ ] `thulp init` command
- [ ] `thulp run <tool>` command
- [ ] `thulp skill run` command
- [ ] Output formatting (json, table, plain)

#### Week 12: CLI Complete
- [ ] `thulp adapter generate` command
- [ ] `thulp flow export` command
- [ ] `thulp config` commands
- [ ] Error presentation
- [ ] Progress indicators

#### Week 13: Integration & Polish
- [ ] End-to-end testing
- [ ] Performance optimization
- [ ] Error message improvements
- [ ] Shell completions

### Testing Focus
```
E2E tests: Full CLI command execution
Integration tests: Real-world workflows
BDD: User journey scenarios
```

### BDD Scenarios

```gherkin
Feature: CLI Usage
  Scenario: First-time user experience
    Given I have thulp installed
    When I run "thulp init"
    And I configure a GitHub MCP server
    And I run "thulp run github.list_repos owner=octocat"
    Then I should see repository information

Feature: Skill Workflow
  Scenario: Create and run custom skill
    Given an initialized workspace
    When I create a skill file
    And I run "thulp skill run my-skill --param value"
    Then the skill should execute successfully
```

### Exit Criteria
- [ ] All CLI commands functional
- [ ] E2E tests cover main user journeys
- [ ] Performance targets met
- [ ] Documentation complete

---

## Phase 5: Advanced Features (Weeks 14-16)

### Goals
- Browser automation
- Guidance system
- Registry integration

### Deliverables

#### Week 14: Browser Automation
- [ ] `thulp-browser` crate
- [ ] Chrome DevTools Protocol integration
- [ ] Basic navigation and interaction
- [ ] Screenshot capability
- [ ] MCP tool integration

#### Week 15: Guidance & Registry
- [ ] `thulp-guidance` crate
- [ ] Embedded documentation
- [ ] `thulp how` command
- [ ] `thulp-registry` crate
- [ ] Registry client basics

#### Week 16: Polish & Release
- [ ] Registry publish/fetch
- [ ] Final testing
- [ ] Documentation review
- [ ] Release preparation
- [ ] v0.1.0 release

### Testing Focus
```
Integration tests: Browser automation
E2E tests: Registry workflows
Documentation review
```

### Exit Criteria
- [ ] Browser automation works for basic scenarios
- [ ] Guidance system provides useful help
- [ ] Registry allows publish/fetch
- [ ] v0.1.0 ready for release

---

## Milestone Summary

| Milestone | Week | Key Deliverable |
|-----------|------|-----------------|
| M1: Foundation | 3 | Core types, basic query |
| M2: MCP Working | 6 | Connect to MCP servers, generate adapters |
| M3: Skills Working | 10 | Execute skills, export flows |
| M4: CLI Complete | 13 | Full CLI, e2e tests passing |
| M5: v0.1.0 | 16 | Browser, guidance, registry, release |

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| rs-utcp API changes | Pin version, maintain abstraction layer |
| jq compatibility complexity | Prioritize common operations, document gaps |
| Browser automation complexity | Start with chromiumoxide, fallback to CDP |
| Registry infrastructure | Use existing Dirmacs infra, defer if needed |

## Dependencies

### External Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| rs-utcp | 0.3.x | MCP protocol, OpenAPI conversion |
| tokio | 1.x | Async runtime |
| clap | 4.x | CLI parsing |
| serde | 1.x | Serialization |
| serde_yaml | 0.9.x | YAML support |
| nom | 7.x | Parser combinators |
| tera | 1.x | Templating |
| thiserror | 1.x | Error types |
| tracing | 0.1.x | Logging/tracing |
| reqwest | 0.12.x | HTTP client |

### Development Tools

| Tool | Purpose |
|------|---------|
| cargo-nextest | Fast test runner |
| cargo-llvm-cov | Coverage reporting |
| cucumber | BDD test framework |

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | >80% | cargo-llvm-cov |
| Build Time | <60s | CI metrics |
| Binary Size | <20MB | Release build |
| CLI Startup | <50ms | Benchmark |
| Documentation | 100% public API | cargo doc warnings |
