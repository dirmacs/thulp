# thulp-guidance

LLM guidance and prompt template system for Thulp.

## Overview

This crate provides utilities for creating, managing, and rendering prompt templates for AI agent interactions. It supports variable substitution, default values, and a template registry for organizing prompts.

## Features

- **Prompt Templates**: Define templates with `{{variable}}` placeholders
- **Variable Substitution**: Replace placeholders with runtime values
- **Default Values**: Set fallback values for template variables
- **Template Registry**: Organize and manage multiple templates
- **JSON Serialization**: Full serde support for templates

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-guidance = "0.2"
```

## Usage

### Creating a Prompt Template

```rust
use thulp_guidance::PromptTemplate;

let template = PromptTemplate::new(
    "greeting",
    "Hello {{name}}! Welcome to {{place}}."
);
```

### Rendering with Variables

```rust
use thulp_guidance::PromptTemplate;
use std::collections::HashMap;

let template = PromptTemplate::new("greeting", "Hello {{name}}!");

let mut vars = HashMap::new();
vars.insert("name".to_string(), "World".to_string());

let result = template.render(&vars).unwrap();
assert_eq!(result, "Hello World!");
```

### Using Default Values

```rust
use thulp_guidance::PromptTemplate;
use std::collections::HashMap;

let template = PromptTemplate::new("greeting", "Hello {{name}}!")
    .with_default("name", "Guest");

// Renders with default value
let result = template.render(&HashMap::new()).unwrap();
assert_eq!(result, "Hello Guest!");

// Override default with provided value
let mut vars = HashMap::new();
vars.insert("name".to_string(), "Alice".to_string());
let result = template.render(&vars).unwrap();
assert_eq!(result, "Hello Alice!");
```

### Template Registry

```rust
use thulp_guidance::{PromptTemplate, TemplateRegistry};
use std::collections::HashMap;

let mut registry = TemplateRegistry::new();

// Register templates
registry.register(PromptTemplate::new("greeting", "Hello {{name}}!"));
registry.register(PromptTemplate::new("farewell", "Goodbye {{name}}!"));

// List all templates
let names = registry.list();
println!("Available templates: {:?}", names);

// Render by name
let mut vars = HashMap::new();
vars.insert("name".to_string(), "World".to_string());

let greeting = registry.render("greeting", &vars).unwrap();
assert_eq!(greeting, "Hello World!");
```

### Complex Templates

```rust
use thulp_guidance::PromptTemplate;
use std::collections::HashMap;

let template = PromptTemplate::new(
    "code_review",
    r#"Please review the following {{language}} code:

```{{language}}
{{code}}
```

Focus on: {{focus_areas}}

Provide feedback on code quality, potential bugs, and improvements."#
)
.with_default("focus_areas", "readability and performance");

let mut vars = HashMap::new();
vars.insert("language".to_string(), "rust".to_string());
vars.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());

let prompt = template.render(&vars).unwrap();
```

## Error Handling

The crate provides specific error types:

- `GuidanceError::Template`: Template rendering errors
- `GuidanceError::VariableNotFound`: Missing template or variable
- `GuidanceError::InvalidFormat`: Format validation errors

```rust
use thulp_guidance::{PromptTemplate, GuidanceError};
use std::collections::HashMap;

let template = PromptTemplate::new("test", "Hello {{name}}!");

// Missing variable returns error
let result = template.render(&HashMap::new());
assert!(result.is_err());
```

## Testing

```bash
cargo test -p thulp-guidance
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
