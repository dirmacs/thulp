# Thulp Architecture

## Overview

Thulp is architected as a Rust workspace with multiple crates, each responsible for a specific domain. The architecture leverages **rs-utcp** as a foundational dependency for MCP protocol support and OpenAPI conversion.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         thulp-cli                                │
│                    (User Interface Layer)                        │
└─────────────────────────────────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│ thulp-skills  │       │thulp-guidance │       │thulp-registry │
│  (Workflows)  │       │    (Docs)     │       │   (Sharing)   │
└───────────────┘       └───────────────┘       └───────────────┘
        │                       │                       │
        ▼                       │                       │
┌───────────────┐               │                       │
│thulp-skill-   │               │                       │
│   files       │               │                       │
│(SKILL.md Parse)               │                       │
└───────────────┘               │                       │
        │                       ▼                       ▼
        └───────────────────────┼───────────────────────┘
                                ▼
┌───────────────────────────────────────────────────────────────┐
│                      thulp-workspace                           │
│         (Project, Session & SessionManager)                    │
└───────────────────────────────────────────────────────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│  thulp-mcp    │       │ thulp-adapter │       │ thulp-browser │
│(MCP Protocol) │       │(API→MCP Conv) │       │  (Playwright) │
└───────────────┘       └───────────────┘       └───────────────┘
        │                       │
        └───────────┬───────────┘
                    ▼
        ┌───────────────────────┐
        │       rs-utcp         │
        │  (Protocol Foundation)│
        └───────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│  thulp-query  │       │  thulp-core   │
│ (jq-compat)   │       │(Types/Traits) │
└───────────────┘       └───────────────┘
```

## Crate Responsibilities

### Foundation Layer

#### `thulp-core`
Core types, traits, and error definitions shared across all crates.

```rust
// Key types
pub struct ToolDefinition { ... }
pub struct ToolCall { ... }
pub struct ToolResult { ... }
pub struct ExecutionContext { ... }

// Key traits
pub trait Tool: Send + Sync { ... }
pub trait Transport: Send + Sync { ... }
pub trait StateStore: Send + Sync { ... }
```

**Dependencies**: `serde`, `thiserror`, `async-trait`

#### `thulp-query`
jq-compatible query engine for JSON transformation.

```rust
pub struct Query { ... }
pub fn parse(expr: &str) -> Result<Query>;
pub fn execute(query: &Query, input: &Value) -> Result<Value>;
```

**Dependencies**: `serde_json`, `nom` (parser combinators)

#### `thulp-skill-files`
SKILL.md file parsing and loading with YAML frontmatter support.

```rust
pub struct SkillFile {
    pub metadata: SkillMetadata,
    pub content: String,
}

pub struct SkillLoader { ... }
pub struct SkillPreprocessor { ... }

impl SkillFile {
    pub fn parse(content: &str) -> Result<Self>;
}

impl SkillLoader {
    pub fn new() -> Self;
    pub fn with_scope(scope: SkillScope) -> Self;
    pub fn load_from_directory(&self, dir: &Path) -> Result<Vec<SkillFile>>;
}

impl SkillPreprocessor {
    pub fn process(&self, content: &str, variables: &HashMap<String, String>) -> String;
}
```

**Skill Scopes**: `Global`, `Workspace`, `Project` (priority: Project > Workspace > Global)

**Dependencies**: `serde`, `serde_yaml`, `thiserror`

### Protocol Layer

#### `thulp-mcp`
MCP protocol client wrapping rs-utcp's MCP transport.

```rust
// Wraps rs_utcp::transports::mcp
pub struct McpClient { ... }
pub struct McpServer { ... }  // Server info
pub struct McpTransport { ... }

impl McpClient {
    pub async fn connect_stdio(command: &str, args: &[&str]) -> Result<Self>;
    pub async fn connect_sse(url: &str) -> Result<Self>;
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>>;
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult>;
}
```

**Dependencies**: `rs-utcp` (features: `mcp`), `tokio`, `thulp-core`

#### `thulp-adapter`
API-to-MCP adapter generation wrapping rs-utcp's OpenAPI converter.

```rust
// Wraps rs_utcp::openapi::OpenApiConverter
pub struct AdapterGenerator { ... }
pub struct GeneratedAdapter { ... }

impl AdapterGenerator {
    pub fn from_openapi(spec: &str) -> Result<Self>;
    pub fn from_graphql(schema: &str) -> Result<Self>;
    pub fn generate(&self) -> Result<GeneratedAdapter>;
}
```

**Dependencies**: `rs-utcp` (features: `http`), `thulp-core`

### Workspace Layer

#### `thulp-workspace`
Project structure and session management.

```rust
pub struct Workspace { ... }
pub struct Session { ... }
pub struct SessionManager { ... }
pub struct SessionConfig { ... }
pub struct Project { ... }

impl Workspace {
    pub fn init(path: &Path) -> Result<Self>;
    pub fn load(path: &Path) -> Result<Self>;
    pub fn create_session(&self) -> Result<Session>;
}

impl Session {
    pub fn new(id: SessionId, config: SessionConfig) -> Self;
    pub fn turn_count(&self) -> usize;  // Count of conversation turns
    pub fn add_entry(&mut self, entry: SessionEntry) -> Result<()>;
    pub fn is_expired(&self) -> bool;
}

impl SessionManager {
    pub fn new(workspace_path: &Path) -> Self;
    pub async fn create(&self, config: SessionConfig) -> Result<Session>;
    pub async fn load(&self, id: &SessionId) -> Result<Option<Session>>;
    pub async fn save(&self, session: &Session) -> Result<()>;
    pub async fn list(&self) -> Result<Vec<SessionId>>;
    pub async fn delete(&self, id: &SessionId) -> Result<()>;
}
```

**SessionConfig Options**:
- `max_turns` - Maximum conversation turns allowed
- `max_entries` - Maximum entries in session
- `max_duration` - Session timeout duration

**Key Files Managed**:
- `.thulp/config.yaml` - Workspace configuration
- `.thulp/sessions/` - Session history (file-based persistence)
- `.thulp/cache/` - Tool discovery cache
- `.thulp/adapters/` - Generated adapters

**Dependencies**: `thulp-core`, `tokio`, `serde_yaml`

### Feature Layer

#### `thulp-skills`
Parameterized workflow definitions and execution.

```rust
pub struct Skill { ... }
pub struct SkillStep { ... }
pub struct SkillParameter { ... }
pub struct SkillResult { ... }
pub struct StepResult { ... }
pub struct ExecutionContext { ... }

// SkillExecutor trait - pluggable execution strategies
#[async_trait]
pub trait SkillExecutor: Send + Sync {
    async fn execute(&self, skill: &Skill, context: &mut ExecutionContext) -> Result<SkillResult, SkillError>;
    async fn execute_step(&self, step: &SkillStep, context: &mut ExecutionContext) -> Result<StepResult, SkillError>;
}

// ExecutionHooks - lifecycle callbacks for observability
pub trait ExecutionHooks: Send + Sync {
    fn before_skill(&self, skill: &Skill, context: &ExecutionContext);
    fn after_skill(&self, skill: &Skill, result: &SkillResult, context: &ExecutionContext);
    fn before_step(&self, step: &SkillStep, step_index: usize, context: &ExecutionContext);
    fn after_step(&self, step: &SkillStep, step_index: usize, result: &StepResult, context: &ExecutionContext);
    fn on_retry(&self, step: &SkillStep, attempt: usize, error: &str, context: &ExecutionContext);
    fn on_error(&self, error: &SkillError, context: &ExecutionContext);
    fn on_timeout(&self, step: &SkillStep, duration_ms: u64, context: &ExecutionContext);
}

// DefaultSkillExecutor - standard implementation with timeout/retry
pub struct DefaultSkillExecutor { ... }

impl DefaultSkillExecutor {
    pub fn new() -> Self;
    pub fn with_hooks<H: ExecutionHooks + 'static>(self, hooks: H) -> Self;
    pub fn with_default_timeout(self, timeout: Duration) -> Self;
    pub fn with_default_retries(self, retries: u32) -> Self;
}

impl Skill {
    pub fn load(path: &Path) -> Result<Self>;
    pub fn validate(&self) -> Result<()>;
    pub async fn execute(&self, params: &Map) -> Result<SkillExecution>;
    pub fn export_shell(&self) -> Result<String>;
}
```

**Built-in Hook Implementations**:
- `NoOpHooks` - No-op implementation (default)
- `TracingHooks` - Logs execution via `tracing` crate
- `CompositeHooks` - Combines multiple hook implementations

**Skill YAML Format**:
```yaml
name: get-user-repos
description: Fetch repositories for a GitHub user
parameters:
  - name: username
    type: string
    required: true
timeout_ms: 30000  # Per-skill timeout
steps:
  - tool: github.list_repos
    args:
      owner: "{{ username }}"
    output: repos
    timeout_ms: 10000  # Per-step timeout
    retries: 2         # Retry on failure
  - query: ".repos | map(.name)"
    output: repo_names
```

**Dependencies**: `thulp-core`, `thulp-mcp`, `thulp-query`, `tera` (templating), `tokio`

#### `thulp-browser`
Browser automation via Playwright/Chrome DevTools Protocol.

```rust
pub struct Browser { ... }
pub struct Page { ... }
pub struct BrowserAction { ... }

impl Browser {
    pub async fn launch() -> Result<Self>;
    pub async fn new_page(&self) -> Result<Page>;
}

impl Page {
    pub async fn navigate(&self, url: &str) -> Result<()>;
    pub async fn click(&self, selector: &str) -> Result<()>;
    pub async fn fill(&self, selector: &str, value: &str) -> Result<()>;
    pub async fn screenshot(&self) -> Result<Vec<u8>>;
}
```

**Dependencies**: `chromiumoxide` or `playwright` bindings, `tokio`

#### `thulp-guidance`
Embedded documentation and how-to system.

```rust
pub struct GuidanceSystem { ... }
pub struct Guide { ... }
pub struct HowTo { ... }

impl GuidanceSystem {
    pub fn search(&self, query: &str) -> Vec<Guide>;
    pub fn get_howto(&self, topic: &str) -> Option<HowTo>;
}
```

**Dependencies**: `thulp-core`, embedded documentation resources

#### `thulp-registry`
Multi-tenant artifact sharing (skills, adapters).

```rust
pub struct Registry { ... }
pub struct Artifact { ... }

impl Registry {
    pub async fn connect(url: &str) -> Result<Self>;
    pub async fn publish(&self, artifact: &Artifact) -> Result<()>;
    pub async fn fetch(&self, name: &str, version: &str) -> Result<Artifact>;
    pub async fn search(&self, query: &str) -> Result<Vec<Artifact>>;
}
```

**Dependencies**: `reqwest`, `thulp-core`, optional auth

### CLI Layer

#### `thulp-cli`
Command-line interface orchestrating all functionality.

```rust
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Skill(SkillArgs),
    Adapter(AdapterArgs),
    Flow(FlowArgs),
    How(HowArgs),
    Registry(RegistryArgs),
}
```

**Dependencies**: `clap`, all thulp-* crates

## rs-utcp Integration Details

### Dependency Configuration

```toml
[dependencies.rs-utcp]
version = "0.3"
default-features = false
features = ["http", "mcp"]
```

### Module Mapping

| Thulp Module | rs-utcp Module | Usage |
|--------------|----------------|-------|
| `thulp-mcp::McpClient` | `rs_utcp::transports::mcp::McpTransport` | MCP protocol communication |
| `thulp-mcp::McpClient::connect_stdio` | `rs_utcp::transports::mcp::StdioMcpTransport` | STDIO process management |
| `thulp-mcp::McpClient::connect_sse` | `rs_utcp::transports::mcp::HttpMcpTransport` | HTTP/SSE transport |
| `thulp-adapter::AdapterGenerator` | `rs_utcp::openapi::OpenApiConverter` | OpenAPI → tools conversion |
| Internal tool search | `rs_utcp::tag::TagSearchStrategy` | Tag-based tool discovery |

### Abstraction Strategy

We wrap rs-utcp types rather than expose them directly:

```rust
// thulp-mcp/src/lib.rs
use rs_utcp::transports::mcp::McpTransport as UtcpMcpTransport;

pub struct McpClient {
    inner: UtcpMcpTransport,
    // Additional thulp-specific state
    session_id: String,
    cache: ToolCache,
}

impl McpClient {
    // Thulp-specific API that delegates to rs-utcp
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult> {
        let utcp_result = self.inner.call(name, args).await?;
        Ok(ToolResult::from_utcp(utcp_result))
    }
}
```

**Benefits**:
- Stable public API regardless of rs-utcp changes
- Add thulp-specific features (caching, session tracking)
- Easier testing with mock implementations

## Data Flow

### Tool Execution Flow

```
User Command
    │
    ▼
┌─────────────┐
│  thulp-cli  │  Parse command, load workspace
└─────────────┘
    │
    ▼
┌─────────────┐
│  workspace  │  Resolve tool, check cache
└─────────────┘
    │
    ▼
┌─────────────┐
│  thulp-mcp  │  Execute via MCP protocol
└─────────────┘
    │
    ▼
┌─────────────┐
│   rs-utcp   │  Protocol implementation
└─────────────┘
    │
    ▼
┌─────────────┐
│ MCP Server  │  External process/service
└─────────────┘
    │
    ▼
┌─────────────┐
│ thulp-query │  Transform response
└─────────────┘
    │
    ▼
Output to User
```

### Skill Execution Flow

```
thulp skill run get-user-repos --username octocat
    │
    ▼
┌─────────────────┐
│ Load skill.yaml │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Validate params │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ For each step:  │
│  - Resolve tool │
│  - Execute      │
│  - Store output │
│  - Apply query  │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Return results  │
└─────────────────┘
```

## Configuration

### Workspace Configuration (`.thulp/config.yaml`)

```yaml
version: 1
workspace:
  name: my-project
  
servers:
  - name: github
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-github"]
    env:
      GITHUB_TOKEN: "${GITHUB_TOKEN}"
      
  - name: filesystem
    transport: stdio
    command: npx
    args: ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed"]

adapters:
  - name: stripe
    source: openapi
    spec: https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.yaml
    
cache:
  ttl: 3600
  max_size: 100MB
```

### Global Configuration (`~/.config/thulp/config.yaml`)

```yaml
registry:
  url: https://registry.dirmacs.org
  # auth is optional
  
defaults:
  output_format: json
  color: auto
```

## Error Handling Strategy

```rust
// thulp-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ThulpError {
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),
    
    #[error("Workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    
    #[error("Skill error: {0}")]
    Skill(#[from] SkillError),
    
    #[error("Query error: {0}")]
    Query(#[from] QueryError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ThulpError>;
```

## Testing Architecture

See [TESTING_STRATEGY.md](./TESTING_STRATEGY.md) for detailed testing approach.

### Test Categories

1. **Unit Tests**: Per-crate, mocked dependencies
2. **Integration Tests**: Cross-crate, real file system
3. **E2E Tests**: Full CLI execution, real MCP servers
4. **BDD Tests**: Cucumber/Gherkin for user-facing features

### Test Infrastructure

```
tests/
├── fixtures/           # Shared test data
│   ├── openapi/       # Sample OpenAPI specs
│   ├── skills/        # Sample skill definitions
│   └── mcp/           # Mock MCP server responses
├── integration/       # Cross-crate tests
└── e2e/              # End-to-end CLI tests
```

## Security Considerations

1. **Credential Management**: Environment variables, not in config files
2. **Subprocess Isolation**: MCP servers run as separate processes
3. **Path Validation**: Adapters can only access declared paths
4. **Registry Auth**: Optional, token-based authentication
