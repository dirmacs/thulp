# thulp-skill-files

SKILL.md file parsing and loading for the Thulp execution context platform.

## Features

- **SKILL.md Parsing**: Parse skill files with YAML frontmatter and markdown content
- **Preprocessor**: Handle `$ARGUMENTS`, `!`command``, `{{variable}}`, and `${ENV_VAR}` substitutions
- **Skill Loader**: Discover skills from multiple directories with scope-based priority
- **Tool Restrictions**: Support for `allowed-tools` to sandbox skill execution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-skill-files = { path = "../thulp-skill-files" }
```

## Quick Start

### Parsing a Skill File

```rust
use thulp_skill_files::SkillFile;

let skill = SkillFile::parse("path/to/SKILL.md")?;
println!("Name: {}", skill.effective_name());
println!("Description: {}", skill.effective_description());
```

### Loading Skills from Directories

```rust
use thulp_skill_files::{SkillLoader, SkillLoaderConfig};

let config = SkillLoaderConfig::default();
let loader = SkillLoader::new(config);
let skills = loader.load_all()?;

for skill in &skills {
    println!("{} ({})", skill.qualified_name(), skill.scope);
}
```

### Preprocessing Skill Content

```rust
use thulp_skill_files::SkillPreprocessor;
use std::collections::HashMap;

let pp = SkillPreprocessor::new();
let mut context = HashMap::new();
context.insert("project".to_string(), serde_json::json!("myapp"));

let content = "Process $ARGUMENTS for {{project}}";
let processed = pp.preprocess(content, "file.txt", &context)?;
// Result: "Process file.txt for myapp"
```

## SKILL.md Format

```markdown
---
name: my-skill
description: Does something useful
allowed-tools:
  - Read
  - Write
  - Bash
disable-model-invocation: false
user-invocable: true
context: inline
requires-approval: false
tags:
  - utility
  - files
---
# Instructions

When invoked with $ARGUMENTS, do the following:

1. Read the specified file
2. Process the contents
3. Write the results
```

## Frontmatter Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | directory name | Display name |
| `description` | string | first paragraph | What the skill does |
| `argument-hint` | string | - | Hint for arguments |
| `disable-model-invocation` | bool | false | Prevent automatic invocation |
| `user-invocable` | bool | true | Allow user invocation |
| `allowed-tools` | list | all | Restrict tool usage |
| `model` | string | - | Model to use |
| `context` | inline/fork | inline | Execution context |
| `agent` | string | - | Subagent type for fork |
| `requires-approval` | bool | false | Require user approval |
| `tags` | list | [] | Categorization tags |
| `version` | string | - | Skill version |
| `author` | string | - | Skill author |

## Scope Priority

Skills are loaded from multiple directories with priority resolution:

1. **Enterprise** (highest) - Organization-wide skills
2. **Personal** - User's personal skills (~/.claude/skills/)
3. **Project** (lowest) - Project-specific skills (./.claude/skills/)
4. **Plugin** - Namespaced, no conflicts

When multiple skills have the same name, higher scope wins.

## Preprocessor Substitutions

| Syntax | Description |
|--------|-------------|
| `$ARGUMENTS` | Replaced with invocation arguments |
| `!`command`` | Replaced with shell command output |
| `{{variable}}` | Replaced with context value |
| `{{a.b.c}}` | Replaced with nested context value |
| `${ENV_VAR}` | Replaced with environment variable |

## License

MIT OR Apache-2.0
