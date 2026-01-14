# Thulp CLI API Design

## Command Structure

```
thulp <command> [subcommand] [options] [arguments]
```

## Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--help` | `-h` | Show help information |
| `--version` | `-V` | Show version |
| `--config` | `-c` | Path to config file |
| `--workspace` | `-w` | Path to workspace |
| `--output` | `-o` | Output format: json, table, plain |
| `--quiet` | `-q` | Suppress non-essential output |
| `--verbose` | `-v` | Increase verbosity (can repeat: -vv) |
| `--color` | | Color output: auto, always, never |

---

## Commands

### `thulp init`

Initialize a new thulp workspace.

```bash
thulp init [path] [options]
```

**Arguments**:
- `path` - Directory to initialize (default: current directory)

**Options**:
| Option | Description |
|--------|-------------|
| `--name` | Workspace name |
| `--template` | Use a template: minimal, standard, full |

**Examples**:
```bash
thulp init
thulp init ./my-project --name "My API Project"
thulp init --template standard
```

**Output**:
```
Initialized thulp workspace in /path/to/project
Created:
  .thulp/config.yaml
  .thulp/sessions/
  .thulp/cache/
  .thulp/adapters/
  .thulp/skills/
```

---

### `thulp run`

Execute a tool call directly.

```bash
thulp run <server>.<tool> [arguments] [options]
```

**Arguments**:
- `server.tool` - Fully qualified tool name
- `arguments` - Tool arguments as key=value pairs or JSON

**Options**:
| Option | Description |
|--------|-------------|
| `--json` | Pass arguments as JSON string |
| `--file` | Read arguments from file |
| `--query` | jq query to transform output |
| `--raw` | Output raw result (no formatting) |

**Examples**:
```bash
# Simple call
thulp run github.list_repos owner=octocat

# With JSON arguments
thulp run github.create_issue --json '{"owner":"octocat","repo":"Hello-World","title":"Bug"}'

# With query transformation
thulp run github.list_repos owner=octocat --query ".[].name"

# From file
thulp run stripe.create_customer --file customer.json
```

**Output** (default):
```json
{
  "result": [...],
  "duration_ms": 234,
  "tokens_used": 0
}
```

---

### `thulp skill`

Manage and execute skills.

#### `thulp skill run`

Execute a skill with parameters.

```bash
thulp skill run <skill-name> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--param`, `-p` | Parameter value (repeatable): -p name=value |
| `--params-file` | Load parameters from YAML/JSON file |
| `--dry-run` | Show execution plan without running |
| `--trace` | Show detailed execution trace |

**Examples**:
```bash
thulp skill run get-user-activity -p username=octocat -p limit=5
thulp skill run deploy-service --params-file params.yaml
thulp skill run backup-database --dry-run
```

#### `thulp skill list`

List available skills.

```bash
thulp skill list [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--local` | Show only local skills |
| `--registry` | Show only registry skills |
| `--tag` | Filter by tag |

#### `thulp skill show`

Show skill details.

```bash
thulp skill show <skill-name>
```

#### `thulp skill create`

Create a new skill from template.

```bash
thulp skill create <name> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--template` | Skill template to use |
| `--output` | Output file path |

#### `thulp skill validate`

Validate a skill definition.

```bash
thulp skill validate <path>
```

---

### `thulp adapter`

Manage API adapters.

#### `thulp adapter generate`

Generate adapter from API specification.

```bash
thulp adapter generate <source> [options]
```

**Arguments**:
- `source` - URL or path to API specification

**Options**:
| Option | Description |
|--------|-------------|
| `--type` | Spec type: openapi, graphql (auto-detected) |
| `--name` | Adapter name |
| `--output` | Output file path |
| `--base-url` | Override base URL |
| `--auth` | Auth type: bearer, api-key, basic |
| `--filter` | Filter operations by tag or pattern |

**Examples**:
```bash
# From URL
thulp adapter generate https://api.example.com/openapi.json --name example

# From local file
thulp adapter generate ./specs/stripe.yaml --name stripe

# With filtering
thulp adapter generate ./api.yaml --filter "tag:users" --name users-api
```

#### `thulp adapter list`

List configured adapters.

```bash
thulp adapter list
```

#### `thulp adapter show`

Show adapter details and available tools.

```bash
thulp adapter show <name>
```

#### `thulp adapter test`

Test adapter connectivity.

```bash
thulp adapter test <name>
```

---

### `thulp server`

Manage MCP servers.

#### `thulp server add`

Add an MCP server configuration.

```bash
thulp server add <name> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--transport` | Transport type: stdio, sse |
| `--command` | Command to run (stdio) |
| `--args` | Command arguments |
| `--url` | Server URL (sse) |
| `--env` | Environment variable: KEY=value |

**Examples**:
```bash
# STDIO server
thulp server add github --transport stdio \
  --command npx \
  --args "-y" --args "@modelcontextprotocol/server-github" \
  --env "GITHUB_TOKEN=\${GITHUB_TOKEN}"

# SSE server
thulp server add remote-api --transport sse \
  --url "https://api.example.com/mcp"
```

#### `thulp server list`

List configured servers.

```bash
thulp server list
```

#### `thulp server remove`

Remove a server configuration.

```bash
thulp server remove <name>
```

#### `thulp server test`

Test server connectivity.

```bash
thulp server test <name>
```

#### `thulp server tools`

List tools from a server.

```bash
thulp server tools <name>
```

---

### `thulp flow`

Manage execution flows.

#### `thulp flow export`

Export a skill or session to a script.

```bash
thulp flow export <source> [options]
```

**Arguments**:
- `source` - Skill name or session ID

**Options**:
| Option | Description |
|--------|-------------|
| `--format` | Output format: bash, powershell, json |
| `--output` | Output file path |

**Examples**:
```bash
thulp flow export get-user-repos --format bash --output get-repos.sh
thulp flow export session:abc123 --format bash
```

#### `thulp flow replay`

Replay a recorded flow.

```bash
thulp flow replay <flow-file> [options]
```

---

### `thulp query`

Execute jq queries on data.

```bash
thulp query <expression> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--input`, `-i` | Input file (default: stdin) |
| `--slurp`, `-s` | Read entire input into array |
| `--raw-output`, `-r` | Output raw strings |
| `--compact`, `-c` | Compact output |

**Examples**:
```bash
echo '{"name":"Alice"}' | thulp query '.name'
thulp query '.users[].email' -i data.json
cat repos.json | thulp query 'map(select(.stars > 100))'
```

---

### `thulp how`

Access embedded guidance.

```bash
thulp how [topic]
```

**Topics**:
- `skills` - Creating and using skills
- `adapters` - Generating API adapters
- `mcp` - MCP protocol basics
- `query` - Query language reference
- `config` - Configuration options
- `flow` - Flow export and replay

**Examples**:
```bash
thulp how              # List all topics
thulp how skills       # Show skills guide
thulp how query        # Show query syntax
```

---

### `thulp guidance`

Search documentation.

```bash
thulp guidance search <query>
```

**Examples**:
```bash
thulp guidance search "authentication"
thulp guidance search "export shell"
```

---

### `thulp registry`

Interact with artifact registry.

#### `thulp registry login`

Authenticate with registry.

```bash
thulp registry login [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--token` | Use token directly |
| `--url` | Registry URL |

#### `thulp registry publish`

Publish a skill or adapter.

```bash
thulp registry publish <path> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--public` | Make publicly available |
| `--tag` | Add tags |

#### `thulp registry fetch`

Fetch an artifact.

```bash
thulp registry fetch <name> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--version` | Specific version |
| `--output` | Output directory |

#### `thulp registry search`

Search for artifacts.

```bash
thulp registry search <query> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--type` | Filter by type: skill, adapter |
| `--limit` | Maximum results |

---

### `thulp config`

Manage configuration.

#### `thulp config show`

Show current configuration.

```bash
thulp config show [key]
```

#### `thulp config set`

Set a configuration value.

```bash
thulp config set <key> <value> [options]
```

**Options**:
| Option | Description |
|--------|-------------|
| `--global` | Set in global config |

#### `thulp config get`

Get a configuration value.

```bash
thulp config get <key>
```

---

### `thulp completion`

Generate shell completions.

```bash
thulp completion <shell>
```

**Shells**: bash, zsh, fish, powershell

**Examples**:
```bash
# Bash
thulp completion bash > ~/.local/share/bash-completion/completions/thulp

# Zsh
thulp completion zsh > ~/.zfunc/_thulp

# Fish
thulp completion fish > ~/.config/fish/completions/thulp.fish

# PowerShell
thulp completion powershell >> $PROFILE
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Connection error |
| 5 | Execution error |
| 6 | Validation error |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `THULP_CONFIG` | Path to global config file |
| `THULP_WORKSPACE` | Default workspace path |
| `THULP_OUTPUT` | Default output format |
| `THULP_COLOR` | Color output setting |
| `THULP_REGISTRY_URL` | Registry URL |
| `THULP_REGISTRY_TOKEN` | Registry auth token |

---

## Output Formats

### JSON (default)
```json
{
  "status": "success",
  "data": { ... },
  "metadata": {
    "duration_ms": 234,
    "server": "github"
  }
}
```

### Table
```
NAME         STARS    LANGUAGE
─────────────────────────────
repo-one     1234     Rust
repo-two     567      TypeScript
```

### Plain
```
repo-one
repo-two
repo-three
```

---

## Interactive Features

### Progress Indicators
Long-running operations show progress:
```
Connecting to github... done
Fetching tools... done (23 tools)
Executing get_user... done (234ms)
```

### Confirmation Prompts
Destructive operations prompt for confirmation:
```
This will delete the adapter 'stripe'. Continue? [y/N]
```

Use `--yes` or `-y` to skip confirmations.

---

## Common Workflows

### First-Time Setup
```bash
# Initialize workspace
thulp init

# Add GitHub server
thulp server add github --transport stdio \
  --command npx \
  --args "-y" --args "@modelcontextprotocol/server-github"

# Test connection
thulp server test github

# List available tools
thulp server tools github
```

### Create and Run Skill
```bash
# Create skill from template
thulp skill create get-user-info --template github-user

# Edit skill file
$EDITOR .thulp/skills/get-user-info.yaml

# Validate
thulp skill validate .thulp/skills/get-user-info.yaml

# Run
thulp skill run get-user-info -p username=octocat
```

### Generate API Adapter
```bash
# Generate from OpenAPI
thulp adapter generate https://api.stripe.com/openapi.json --name stripe

# List tools
thulp adapter show stripe

# Test a call
thulp run stripe.customers_list limit=10
```

### Export to Shell Script
```bash
# Export skill to bash
thulp flow export get-user-info --format bash --output get-user.sh

# Make executable and run
chmod +x get-user.sh
./get-user.sh octocat
```
