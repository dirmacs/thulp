# thulp-adapter

Adapter for converting external tool definitions to Thulp format.

## Overview

This crate provides functionality to convert OpenAPI v2/v3 specifications into Thulp tool definitions. It enables automatic generation of tool adapters from existing API specifications, making it easy to integrate external services with AI agents.

## Features

- Parse OpenAPI v2 and v3 specifications (JSON and YAML)
- Convert API endpoints into Thulp tool definitions
- Extract authentication requirements
- Generate adapter configuration files
- Support for path, query, and body parameters

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-adapter = "0.2"
```

## Usage

### Basic Conversion

```rust
use thulp_adapter::AdapterGenerator;
use serde_json::Value;

// Load an OpenAPI specification
let spec: Value = serde_json::from_str(r#"{
  "openapi": "3.0.0",
  "info": {"title": "Test API", "version": "1.0.0"},
  "paths": {
    "/users": {
      "get": {
        "operationId": "listUsers",
        "summary": "List all users",
        "responses": {
          "200": {
            "description": "A list of users"
          }
        }
      }
    }
  }
}"#).unwrap();

// Create an adapter generator
let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));

// Generate tool definitions
let tools = generator.generate_tools().unwrap();
println!("Generated {} tools", tools.len());
```

### Loading from URL

```rust
use thulp_adapter::AdapterGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generator = AdapterGenerator::from_url(
        "https://api.example.com/openapi.json",
        Some("example-api".to_string())
    ).await?;

    let tools = generator.generate_tools()?;
    Ok(())
}
```

### Extract Authentication Configuration

```rust
use thulp_adapter::AdapterGenerator;
use serde_json::json;

let spec = json!({
    "openapi": "3.0.0",
    "info": {"title": "Test API", "version": "1.0.0"},
    "components": {
        "securitySchemes": {
            "ApiKeyAuth": {
                "type": "apiKey",
                "in": "header",
                "name": "X-API-Key"
            }
        }
    },
    "paths": {}
});

let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
let auth_configs = generator.extract_auth_config();
```

### Generate Configuration File

```rust
use thulp_adapter::AdapterGenerator;
use serde_json::json;

let spec = json!({
    "openapi": "3.0.0",
    "info": {"title": "API", "version": "1.0.0"},
    "paths": {
        "/items": {
            "get": {
                "operationId": "listItems",
                "summary": "List items"
            }
        }
    }
});

let generator = AdapterGenerator::new(spec, Some("items-api".to_string()));
let yaml_config = generator.generate_config()?;
```

## Parameter Type Mapping

The adapter automatically maps OpenAPI types to Thulp parameter types:

| OpenAPI Type | Thulp ParameterType |
|--------------|---------------------|
| `string`     | `ParameterType::String` |
| `integer`    | `ParameterType::Integer` |
| `number`     | `ParameterType::Number` |
| `boolean`    | `ParameterType::Boolean` |
| `array`      | `ParameterType::Array` |
| `object`     | `ParameterType::Object` |

## CLI Usage

Use the Thulp CLI to convert OpenAPI specifications:

```bash
# Convert OpenAPI spec to tool definitions
thulp convert openapi spec.yaml --output tools.yaml

# Show conversion examples
thulp convert examples
```

## Testing

```bash
cargo test -p thulp-adapter
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
