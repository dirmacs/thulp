# Thulp Features Specification

## Feature Overview

| Feature | Description | Implementation | rs-utcp? |
|---------|-------------|----------------|----------|
| MCP Client | Connect to MCP servers | `thulp-mcp` | Yes |
| Adapter Generation | OpenAPI/GraphQL → MCP | `thulp-adapter` | Yes |
| Skills System | Reusable workflows | `thulp-skills` | No |
| Query Engine | jq-compatible transforms | `thulp-query` | No |
| Flow Export | Export to shell scripts | `thulp-skills` | No |
| Browser Automation | Playwright integration | `thulp-browser` | No |
| Guidance System | Embedded documentation | `thulp-guidance` | No |
| Registry | Artifact sharing | `thulp-registry` | No |

## Feature Details

---

### F1: MCP Protocol Client

**Crate**: `thulp-mcp`  
**Priority**: Critical  
**rs-utcp Integration**: Yes - wraps `rs_utcp::transports::mcp`

#### Description
Full Model Context Protocol client supporting multiple transport mechanisms.

#### Capabilities

| Capability | Description |
|------------|-------------|
| STDIO Transport | Launch and communicate with local MCP server processes |
| HTTP/SSE Transport | Connect to remote MCP servers via HTTP with SSE |
| Tool Discovery | List available tools from connected servers |
| Tool Execution | Call tools with arguments, receive structured results |
| Resource Access | Read resources exposed by MCP servers |
| Prompt Templates | Use server-provided prompt templates |

#### API Surface

```rust
// Connection
McpClient::connect_stdio(command, args, env) -> Result<McpClient>
McpClient::connect_sse(url, headers) -> Result<McpClient>

// Discovery
McpClient::list_tools() -> Result<Vec<ToolDefinition>>
McpClient::list_resources() -> Result<Vec<Resource>>
McpClient::list_prompts() -> Result<Vec<Prompt>>

// Execution
McpClient::call_tool(name, args) -> Result<ToolResult>
McpClient::read_resource(uri) -> Result<ResourceContent>
McpClient::get_prompt(name, args) -> Result<PromptContent>

// Lifecycle
McpClient::close() -> Result<()>
```

#### Configuration

```yaml
servers:
  - name: github
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
      
  - name: remote-api
    transport: sse
    url: https://api.example.com/mcp
    headers:
      Authorization: "Bearer ${API_TOKEN}"
```

#### BDD Scenarios

```gherkin
Feature: MCP Client Connection
  
  Scenario: Connect to STDIO MCP server
    Given a valid MCP server command "npx -y @modelcontextprotocol/server-filesystem /tmp"
    When I connect via STDIO transport
    Then the connection should succeed
    And I should be able to list tools
  
  Scenario: Connect to SSE MCP server
    Given a remote MCP server at "https://api.example.com/mcp"
    When I connect via SSE transport
    Then the connection should succeed
    
  Scenario: Execute tool call
    Given I am connected to an MCP server
    When I call tool "read_file" with args {"path": "/tmp/test.txt"}
    Then I should receive a successful result
```

---

### F2: Adapter Framework

**Crate**: `thulp-adapter`  
**Priority**: High  
**rs-utcp Integration**: Yes - wraps `rs_utcp::openapi::OpenApiConverter`

#### Description
Generate MCP-compatible tool definitions from API specifications.

#### Supported Formats

| Format | Support Level | Notes |
|--------|--------------|-------|
| OpenAPI 3.x | Full | Via rs-utcp |
| OpenAPI 2.0 (Swagger) | Full | Auto-converted |
| GraphQL | Planned | Custom implementation |
| gRPC | Planned | Via protobuf reflection |

#### API Surface

```rust
// Generation
AdapterGenerator::from_openapi(spec: &str) -> Result<AdapterGenerator>
AdapterGenerator::from_openapi_url(url: &str) -> Result<AdapterGenerator>
AdapterGenerator::from_graphql(schema: &str) -> Result<AdapterGenerator>

// Configuration
AdapterGenerator::with_auth(auth: AuthConfig) -> Self
AdapterGenerator::with_base_url(url: &str) -> Self
AdapterGenerator::filter_operations(filter: OperationFilter) -> Self

// Output
AdapterGenerator::generate() -> Result<GeneratedAdapter>
GeneratedAdapter::tools() -> &[ToolDefinition]
GeneratedAdapter::save(path: &Path) -> Result<()>
```

#### rs-utcp Feature Mapping

| Thulp Feature | rs-utcp Implementation |
|---------------|----------------------|
| OpenAPI parsing | `OpenApiConverter::from_spec()` |
| Auth extraction | `OpenApiConverter` extracts security schemes |
| Tool generation | `OpenApiConverter::convert()` → `Vec<Tool>` |
| Parameter mapping | Automatic from OpenAPI parameters |

#### Generated Adapter Structure

```yaml
# .thulp/adapters/stripe.yaml
name: stripe
version: "2023-10-16"
base_url: https://api.stripe.com/v1
auth:
  type: bearer
  env_var: STRIPE_API_KEY
tools:
  - name: stripe.customers.list
    description: Returns a list of customers
    parameters:
      - name: limit
        type: integer
        required: false
      - name: starting_after
        type: string
        required: false
    endpoint: GET /customers
```

#### BDD Scenarios

```gherkin
Feature: Adapter Generation

  Scenario: Generate adapter from OpenAPI spec
    Given an OpenAPI 3.0 specification for "Stripe API"
    When I generate an adapter
    Then I should get tool definitions for all operations
    And each tool should have proper parameter types
    
  Scenario: Generate adapter with authentication
    Given an OpenAPI spec with Bearer authentication
    When I generate an adapter
    Then the adapter should include auth configuration
    And the auth should reference environment variables
```

---

### F3: Skills System

**Crate**: `thulp-skills`  
**Priority**: High  
**rs-utcp Integration**: No - custom implementation

#### Description
Parameterized, reusable workflows that compose multiple tool calls with data transformations.

#### Skill Definition Format

```yaml
name: get-user-activity
version: "1.0.0"
description: Get recent activity for a GitHub user
author: dirmacs

parameters:
  - name: username
    type: string
    required: true
    description: GitHub username
  - name: limit
    type: integer
    required: false
    default: 10
    description: Maximum events to return

dependencies:
  servers:
    - github

steps:
  - id: get_user
    tool: github.get_user
    args:
      username: "{{ username }}"
    output: user
    
  - id: get_events
    tool: github.list_user_events
    args:
      username: "{{ username }}"
      per_page: "{{ limit }}"
    output: events
    
  - id: transform
    query: |
      {
        user: .user.login,
        name: .user.name,
        recent_events: .events[:{{ limit }}] | map({
          type: .type,
          repo: .repo.name,
          created: .created_at
        })
      }
    output: result

outputs:
  - name: result
    description: User activity summary
```

#### API Surface

```rust
// Loading
Skill::load(path: &Path) -> Result<Skill>
Skill::from_yaml(content: &str) -> Result<Skill>

// Validation
Skill::validate() -> Result<()>
Skill::check_dependencies(workspace: &Workspace) -> Result<()>

// Execution
Skill::execute(params: &Map, context: &ExecutionContext) -> Result<SkillResult>
Skill::dry_run(params: &Map) -> Result<ExecutionPlan>

// Export
Skill::export_shell() -> Result<String>
Skill::export_json() -> Result<Value>
```

#### Execution Context

```rust
pub struct ExecutionContext {
    workspace: Workspace,
    variables: HashMap<String, Value>,
    outputs: HashMap<String, Value>,
    trace: ExecutionTrace,
}
```

#### BDD Scenarios

```gherkin
Feature: Skills Execution

  Scenario: Execute a simple skill
    Given a skill "get-user-repos" with parameter "username"
    When I execute the skill with username "octocat"
    Then all steps should complete successfully
    And I should receive the expected output
    
  Scenario: Skill with dependencies
    Given a skill that requires the "github" server
    And the github server is not configured
    When I try to execute the skill
    Then I should get a dependency error
    
  Scenario: Export skill to shell script
    Given a skill "get-user-repos"
    When I export to shell format
    Then I should get a valid bash script
    And the script should be executable
```

---

### F4: Query Engine

**Crate**: `thulp-query`  
**Priority**: High  
**rs-utcp Integration**: No - custom implementation

#### Description
98% jq-compatible query language for JSON transformation.

#### Supported Operations

| Category | Operations |
|----------|------------|
| Identity | `.` |
| Object access | `.foo`, `.foo.bar`, `.["key"]` |
| Array access | `.[0]`, `.[-1]`, `.[2:5]` |
| Iteration | `.[]`, `.foo[]` |
| Pipe | `\|` |
| Filters | `select()`, `map()`, `sort_by()`, `group_by()` |
| Constructors | `[]`, `{}`, `{foo: .bar}` |
| Conditionals | `if-then-else`, `//` (alternative) |
| Comparisons | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Boolean | `and`, `or`, `not` |
| String | `split()`, `join()`, `test()`, `capture()` |
| Math | `+`, `-`, `*`, `/`, `%`, `length`, `add` |
| Type | `type`, `keys`, `values`, `has()` |

#### API Surface

```rust
// Parsing
Query::parse(expr: &str) -> Result<Query>
Query::is_valid(expr: &str) -> bool

// Execution
Query::execute(&self, input: &Value) -> Result<Value>
Query::execute_iter(&self, input: &Value) -> Result<impl Iterator<Item = Value>>

// Utilities
Query::explain(&self) -> String  // Human-readable explanation
```

#### BDD Scenarios

```gherkin
Feature: Query Engine

  Scenario: Simple object access
    Given JSON input {"name": "Alice", "age": 30}
    When I execute query ".name"
    Then the result should be "Alice"
    
  Scenario: Array mapping
    Given JSON input [{"name": "a"}, {"name": "b"}]
    When I execute query "map(.name)"
    Then the result should be ["a", "b"]
    
  Scenario: Complex pipeline
    Given JSON input from GitHub API
    When I execute query ".[] | select(.stargazers_count > 100) | .name"
    Then I should get filtered repository names
```

---

### F5: Flow Export

**Crate**: `thulp-skills` (submodule)  
**Priority**: Medium  
**rs-utcp Integration**: No

#### Description
Convert skill executions or interactive sessions into deterministic shell scripts.

#### Export Formats

| Format | Description |
|--------|-------------|
| Bash | POSIX-compatible shell script |
| PowerShell | Windows PowerShell script |
| JSON | Structured execution plan |

#### Generated Script Features

- Environment variable handling
- Error checking (`set -e`)
- Intermediate result storage
- jq for JSON processing
- curl for HTTP calls

#### Example Export

```bash
#!/usr/bin/env bash
set -euo pipefail

# Generated by thulp from skill: get-user-repos
# Parameters: username=octocat

USERNAME="${1:?Usage: $0 <username>}"

# Step 1: Get user info
USER_RESPONSE=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
  "https://api.github.com/users/${USERNAME}")

# Step 2: Get repositories
REPOS_RESPONSE=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
  "https://api.github.com/users/${USERNAME}/repos?per_page=10")

# Step 3: Transform output
echo "$REPOS_RESPONSE" | jq '[.[] | {name: .name, stars: .stargazers_count}]'
```

---

### F6: Browser Automation

**Crate**: `thulp-browser`  
**Priority**: Medium  
**rs-utcp Integration**: No

#### Description
Playwright-style browser automation for web interactions.

#### Capabilities

| Capability | Description |
|------------|-------------|
| Navigation | Go to URL, back, forward, reload |
| Interaction | Click, fill, select, hover |
| Waiting | Wait for selector, text, navigation |
| Screenshots | Full page or element screenshots |
| Evaluation | Execute JavaScript in page context |

#### API Surface

```rust
// Browser lifecycle
Browser::launch(options: LaunchOptions) -> Result<Browser>
Browser::close(&self) -> Result<()>

// Page management
Browser::new_page(&self) -> Result<Page>
Page::close(&self) -> Result<()>

// Navigation
Page::goto(&self, url: &str) -> Result<()>
Page::back(&self) -> Result<()>
Page::forward(&self) -> Result<()>
Page::reload(&self) -> Result<()>

// Interaction
Page::click(&self, selector: &str) -> Result<()>
Page::fill(&self, selector: &str, value: &str) -> Result<()>
Page::select(&self, selector: &str, value: &str) -> Result<()>

// Waiting
Page::wait_for_selector(&self, selector: &str) -> Result<Element>
Page::wait_for_text(&self, text: &str) -> Result<()>

// Content
Page::content(&self) -> Result<String>
Page::screenshot(&self, options: ScreenshotOptions) -> Result<Vec<u8>>
Page::evaluate<T>(&self, script: &str) -> Result<T>
```

#### BDD Scenarios

```gherkin
Feature: Browser Automation

  Scenario: Navigate and screenshot
    Given a browser instance
    When I navigate to "https://example.com"
    And I take a screenshot
    Then the screenshot should be saved
    
  Scenario: Form interaction
    Given a page with a login form
    When I fill "username" with "testuser"
    And I fill "password" with "testpass"
    And I click the submit button
    Then I should be logged in
```

---

### F7: Guidance System

**Crate**: `thulp-guidance`  
**Priority**: Low  
**rs-utcp Integration**: No

#### Description
Embedded documentation and how-to guides accessible via CLI.

#### Commands

```bash
thulp how                    # List all topics
thulp how skills             # How to create skills
thulp how adapters           # How to generate adapters
thulp how mcp                # MCP protocol basics
thulp guidance search <term> # Search documentation
```

#### Content Structure

```
guidance/
├── concepts/
│   ├── mcp.md
│   ├── skills.md
│   └── adapters.md
├── howto/
│   ├── first-skill.md
│   ├── custom-adapter.md
│   └── browser-automation.md
└── reference/
    ├── config.md
    ├── query-syntax.md
    └── skill-format.md
```

---

### F8: Registry System

**Crate**: `thulp-registry`  
**Priority**: Low  
**rs-utcp Integration**: No

#### Description
Multi-tenant artifact sharing for skills and adapters.

#### Capabilities

| Capability | Description |
|------------|-------------|
| Publish | Upload skills/adapters to registry |
| Fetch | Download artifacts by name/version |
| Search | Find artifacts by keyword/tag |
| Versions | Semantic versioning support |
| Auth | Optional token-based authentication |

#### API Surface

```rust
// Connection
Registry::connect(config: RegistryConfig) -> Result<Registry>

// Publishing
Registry::publish(artifact: &Artifact) -> Result<PublishResult>
Registry::unpublish(name: &str, version: &str) -> Result<()>

// Fetching
Registry::fetch(name: &str, version: Option<&str>) -> Result<Artifact>
Registry::search(query: &str) -> Result<Vec<ArtifactSummary>>

// Information
Registry::info(name: &str) -> Result<ArtifactInfo>
Registry::versions(name: &str) -> Result<Vec<Version>>
```

#### CLI Commands

```bash
thulp registry login
thulp registry publish ./skills/my-skill.yaml
thulp registry fetch dirmacs/common-skills
thulp registry search "github"
```

---

## Feature Dependencies

```
thulp-core (foundation)
    │
    ├── thulp-query
    │
    ├── thulp-mcp ←── rs-utcp
    │       │
    │       └── thulp-adapter ←── rs-utcp
    │
    ├── thulp-workspace
    │       │
    │       ├── thulp-skills (uses mcp, query)
    │       │
    │       └── thulp-browser
    │
    ├── thulp-guidance
    │
    └── thulp-registry
            │
            └── thulp-cli (orchestrates all)
```

## Non-Functional Requirements

| Requirement | Target |
|-------------|--------|
| First tool call latency | < 100ms (cached) |
| Memory usage | < 50MB base |
| Binary size | < 20MB |
| Startup time | < 50ms |
| Test coverage | > 80% |
