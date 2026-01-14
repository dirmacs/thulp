# Thulp Testing Strategy

## Overview

Thulp follows a rigorous Test-Driven Development (TDD) and Behavior-Driven Development (BDD) approach. This document outlines the testing strategy, tools, and patterns used throughout the project.

## Testing Philosophy

### TDD Principles
1. **Red**: Write a failing test first
2. **Green**: Write minimal code to pass the test
3. **Refactor**: Improve code while keeping tests green

### BDD Principles
1. **Given**: Initial context/state
2. **When**: Action or event
3. **Then**: Expected outcome

### Test Pyramid

```
        /\
       /  \        E2E Tests (few)
      /────\       - Full CLI workflows
     /      \      - Real MCP servers
    /────────\     
   /          \    Integration Tests (some)
  /────────────\   - Cross-crate interactions
 /              \  - File system operations
/────────────────\ 
        ▼          Unit Tests (many)
                   - Individual functions
                   - Isolated components
```

## Test Categories

### 1. Unit Tests

**Location**: `<crate>/src/**/*.rs` (inline) or `<crate>/tests/unit/`

**Characteristics**:
- Test individual functions/methods
- No I/O, no network, no file system
- Fast execution (<1ms per test)
- Mocked dependencies

**Example**:
```rust
// thulp-core/src/types.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definition_serializes_correctly() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            parameters: vec![],
        };
        
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
    }

    #[test]
    fn tool_call_validates_required_params() {
        let tool = ToolDefinition {
            name: "test".to_string(),
            description: "".to_string(),
            parameters: vec![
                Parameter {
                    name: "required_param".to_string(),
                    required: true,
                    ..Default::default()
                }
            ],
        };
        
        let call = ToolCall {
            tool: "test".to_string(),
            arguments: serde_json::json!({}),
        };
        
        let result = tool.validate_call(&call);
        assert!(result.is_err());
    }
}
```

### 2. Integration Tests

**Location**: `tests/integration/`

**Characteristics**:
- Test cross-crate interactions
- May use file system (temp directories)
- May use test fixtures
- Medium execution time

**Example**:
```rust
// tests/integration/workspace_skill_test.rs
use thulp_workspace::Workspace;
use thulp_skills::Skill;
use tempfile::tempdir;

#[tokio::test]
async fn workspace_loads_skills_from_directory() {
    let dir = tempdir().unwrap();
    
    // Create workspace
    let ws = Workspace::init(dir.path()).await.unwrap();
    
    // Create skill file
    let skill_content = r#"
name: test-skill
steps:
  - tool: echo
    args: {message: "hello"}
"#;
    std::fs::write(
        dir.path().join(".thulp/skills/test.yaml"),
        skill_content
    ).unwrap();
    
    // Load and verify
    let skills = ws.load_skills().await.unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "test-skill");
}
```

### 3. End-to-End Tests

**Location**: `tests/e2e/`

**Characteristics**:
- Test full CLI commands
- May use real MCP servers
- Slower execution
- Test complete user workflows

**Example**:
```rust
// tests/e2e/cli_init_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn cli_init_creates_workspace() {
    let dir = tempdir().unwrap();
    
    Command::cargo_bin("thulp")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized thulp workspace"));
    
    assert!(dir.path().join(".thulp/config.yaml").exists());
}

#[test]
fn cli_init_fails_in_existing_workspace() {
    let dir = tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".thulp")).unwrap();
    
    Command::cargo_bin("thulp")
        .unwrap()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}
```

### 4. BDD Tests (Cucumber)

**Location**: `tests/features/`

**Characteristics**:
- Human-readable scenarios
- Test user-facing behavior
- Document expected behavior
- Stakeholder communication

**Feature Files**:
```gherkin
# tests/features/skill_execution.feature
Feature: Skill Execution
  As a developer
  I want to execute skills with parameters
  So that I can automate repetitive workflows

  Background:
    Given a workspace with a configured GitHub server

  Scenario: Execute a simple skill
    Given a skill "get-repos" defined as:
      """
      name: get-repos
      parameters:
        - name: username
          type: string
          required: true
      steps:
        - tool: github.list_repos
          args:
            owner: "{{ username }}"
      """
    When I execute the skill with username "octocat"
    Then the execution should succeed
    And the result should contain repository data

  Scenario: Skill fails with missing required parameter
    Given a skill "get-repos" with required parameter "username"
    When I execute the skill without parameters
    Then the execution should fail
    And the error should mention "username"

  Scenario: Skill with query transformation
    Given a skill that transforms output with query ".[].name"
    When I execute the skill
    Then the result should be an array of strings
```

**Step Definitions**:
```rust
// tests/features/steps/skill_steps.rs
use cucumber::{given, when, then, World};
use thulp_workspace::Workspace;
use thulp_skills::Skill;

#[derive(Debug, Default, World)]
pub struct SkillWorld {
    workspace: Option<Workspace>,
    skill: Option<Skill>,
    result: Option<Result<Value, Error>>,
}

#[given("a workspace with a configured GitHub server")]
async fn workspace_with_github(world: &mut SkillWorld) {
    let dir = tempdir().unwrap();
    let ws = Workspace::init(dir.path()).await.unwrap();
    // Configure mock GitHub server
    world.workspace = Some(ws);
}

#[given(expr = "a skill {string} defined as:")]
async fn skill_defined(world: &mut SkillWorld, name: String, content: String) {
    let skill = Skill::from_yaml(&content).unwrap();
    world.skill = Some(skill);
}

#[when(expr = "I execute the skill with username {string}")]
async fn execute_skill(world: &mut SkillWorld, username: String) {
    let skill = world.skill.as_ref().unwrap();
    let ws = world.workspace.as_ref().unwrap();
    let params = serde_json::json!({"username": username});
    world.result = Some(skill.execute(&params, ws).await);
}

#[then("the execution should succeed")]
fn execution_succeeds(world: &mut SkillWorld) {
    assert!(world.result.as_ref().unwrap().is_ok());
}
```

## Test Infrastructure

### Directory Structure

```
thulp/
├── tests/
│   ├── common/           # Shared test utilities
│   │   ├── mod.rs
│   │   ├── fixtures.rs   # Test fixture loading
│   │   ├── mocks.rs      # Mock implementations
│   │   └── helpers.rs    # Test helpers
│   │
│   ├── fixtures/         # Test data
│   │   ├── openapi/      # Sample OpenAPI specs
│   │   │   ├── petstore.yaml
│   │   │   └── stripe.yaml
│   │   ├── skills/       # Sample skill definitions
│   │   │   ├── simple.yaml
│   │   │   └── complex.yaml
│   │   └── mcp/          # Mock MCP responses
│   │       └── github_tools.json
│   │
│   ├── integration/      # Integration tests
│   │   ├── mod.rs
│   │   ├── workspace_test.rs
│   │   ├── mcp_test.rs
│   │   └── adapter_test.rs
│   │
│   ├── e2e/             # End-to-end tests
│   │   ├── mod.rs
│   │   ├── cli_test.rs
│   │   └── workflow_test.rs
│   │
│   └── features/        # BDD feature files
│       ├── skill_execution.feature
│       ├── adapter_generation.feature
│       ├── mcp_connection.feature
│       └── steps/
│           ├── mod.rs
│           ├── skill_steps.rs
│           └── mcp_steps.rs
```

### Test Utilities

```rust
// tests/common/mod.rs
pub mod fixtures;
pub mod mocks;
pub mod helpers;

// tests/common/fixtures.rs
use std::path::PathBuf;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .expect(&format!("Failed to load fixture: {}", name))
}

// tests/common/mocks.rs
use thulp_mcp::{McpClient, ToolDefinition, ToolResult};
use async_trait::async_trait;

pub struct MockMcpClient {
    pub tools: Vec<ToolDefinition>,
    pub call_results: HashMap<String, Value>,
}

#[async_trait]
impl McpClientTrait for MockMcpClient {
    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(self.tools.clone())
    }
    
    async fn call_tool(&self, name: &str, _args: Value) -> Result<ToolResult> {
        self.call_results.get(name)
            .cloned()
            .ok_or_else(|| Error::ToolNotFound(name.to_string()))
    }
}

// tests/common/helpers.rs
use tempfile::{tempdir, TempDir};
use thulp_workspace::Workspace;

pub async fn create_test_workspace() -> (TempDir, Workspace) {
    let dir = tempdir().unwrap();
    let ws = Workspace::init(dir.path()).await.unwrap();
    (dir, ws)
}

pub fn assert_json_eq(actual: &Value, expected: &Value) {
    assert_eq!(
        serde_json::to_string_pretty(actual).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}
```

## Testing Patterns

### Pattern 1: Builder Pattern for Test Data

```rust
#[derive(Default)]
pub struct ToolDefinitionBuilder {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
}

impl ToolDefinitionBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }
    
    pub fn parameter(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }
    
    pub fn build(self) -> ToolDefinition {
        ToolDefinition {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
        }
    }
}

// Usage
#[test]
fn test_with_builder() {
    let tool = ToolDefinitionBuilder::new("test")
        .description("A test tool")
        .parameter(Parameter::required("name"))
        .build();
    
    assert_eq!(tool.parameters.len(), 1);
}
```

### Pattern 2: Snapshot Testing

```rust
use insta::assert_snapshot;

#[test]
fn adapter_generates_expected_yaml() {
    let spec = load_fixture("openapi/petstore.yaml");
    let adapter = AdapterGenerator::from_openapi(&spec)
        .unwrap()
        .generate()
        .unwrap();
    
    // Snapshots stored in tests/snapshots/
    assert_snapshot!(adapter.to_yaml());
}
```

### Pattern 3: Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn query_parse_roundtrip(expr in "[.a-z]+") {
        // If it parses, it should produce consistent results
        if let Ok(query) = Query::parse(&expr) {
            let input = serde_json::json!({"a": 1, "b": 2});
            let result1 = query.execute(&input);
            let result2 = query.execute(&input);
            assert_eq!(result1, result2);
        }
    }
    
    #[test]
    fn tool_definition_serialization_roundtrip(
        name in "[a-z_]+",
        desc in ".*"
    ) {
        let tool = ToolDefinition {
            name,
            description: desc,
            parameters: vec![],
        };
        
        let json = serde_json::to_string(&tool).unwrap();
        let parsed: ToolDefinition = serde_json::from_str(&json).unwrap();
        
        assert_eq!(tool.name, parsed.name);
    }
}
```

### Pattern 4: Table-Driven Tests

```rust
#[test]
fn query_operations() {
    let cases = vec![
        // (input, query, expected)
        (json!({"a": 1}), ".a", json!(1)),
        (json!({"a": {"b": 2}}), ".a.b", json!(2)),
        (json!([1, 2, 3]), ".[0]", json!(1)),
        (json!([1, 2, 3]), ".[-1]", json!(3)),
        (json!([1, 2, 3]), "map(. * 2)", json!([2, 4, 6])),
    ];
    
    for (input, query_str, expected) in cases {
        let query = Query::parse(query_str).unwrap();
        let result = query.execute(&input).unwrap();
        assert_eq!(result, expected, "Query: {}", query_str);
    }
}
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache
        uses: Swatinem/rust-cache@v2
      
      - name: Run unit tests
        run: cargo test --lib --all-features
      
      - name: Run integration tests
        run: cargo test --test '*' --all-features
      
      - name: Run BDD tests
        run: cargo test --test cucumber --all-features

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      
      - name: Generate coverage
        run: cargo llvm-cov --all-features --lcov --output-path lcov.info
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
```

## Coverage Requirements

| Crate | Target Coverage | Notes |
|-------|-----------------|-------|
| thulp-core | 90% | Core types must be well-tested |
| thulp-query | 95% | Query engine requires comprehensive tests |
| thulp-mcp | 80% | Integration with rs-utcp |
| thulp-adapter | 80% | Wrapper tests |
| thulp-workspace | 85% | Config handling |
| thulp-skills | 85% | Skill execution |
| thulp-cli | 70% | E2E covers most paths |
| Overall | 80% | Project minimum |

## Testing Checklist

### For Each New Feature

- [ ] Write failing unit tests
- [ ] Implement feature to pass tests
- [ ] Add integration tests if cross-crate
- [ ] Add BDD scenario if user-facing
- [ ] Update test fixtures if needed
- [ ] Run full test suite
- [ ] Check coverage hasn't decreased

### For Bug Fixes

- [ ] Write test that reproduces bug
- [ ] Verify test fails
- [ ] Fix bug
- [ ] Verify test passes
- [ ] Add regression test to prevent recurrence

### Before PR Merge

- [ ] All CI checks pass
- [ ] Coverage meets targets
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] BDD scenarios for new features
- [ ] Documentation updated

## Tools

| Tool | Purpose | Install |
|------|---------|---------|
| cargo-nextest | Fast test runner | `cargo install cargo-nextest` |
| cargo-llvm-cov | Coverage | `cargo install cargo-llvm-cov` |
| cucumber | BDD framework | Cargo dependency |
| insta | Snapshot testing | Cargo dependency |
| proptest | Property testing | Cargo dependency |
| assert_cmd | CLI testing | Cargo dependency |
| mockall | Mocking | Cargo dependency |

## Running Tests

```bash
# Run all tests
cargo test

# Run with nextest (faster)
cargo nextest run

# Run specific crate tests
cargo test -p thulp-core

# Run with output
cargo test -- --nocapture

# Run BDD tests only
cargo test --test cucumber

# Run with coverage
cargo llvm-cov

# Run specific test
cargo test test_name

# Run tests matching pattern
cargo test query_
```
