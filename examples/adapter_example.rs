//! Example demonstrating OpenAPI to Thulp adapter generation
//!
//! This example shows:
//! - Converting OpenAPI specifications to Thulp tool definitions
//! - Extracting authentication configurations
//! - Generating adapter configuration files

use serde_json::json;
use thulp_adapter::AdapterGenerator;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔄 Thulp Adapter Generation Example");
    println!("===================================\n");

    // Example 1: Simple OpenAPI specification
    println!("1. Simple OpenAPI specification");
    let simple_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "User API",
            "version": "1.0.0"
        },
        "paths": {
            "/users": {
                "get": {
                    "operationId": "listUsers",
                    "summary": "List all users",
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "schema": {
                                "type": "integer"
                            },
                            "description": "Maximum number of users to return"
                        }
                    ]
                },
                "post": {
                    "operationId": "createUser",
                    "summary": "Create a new user",
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "name": {"type": "string"},
                                        "email": {"type": "string"}
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/users/{id}": {
                "get": {
                    "operationId": "getUser",
                    "summary": "Get user by ID",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ]
                },
                "put": {
                    "operationId": "updateUser",
                    "summary": "Update user",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "name": {"type": "string"},
                                        "email": {"type": "string"}
                                    }
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "operationId": "deleteUser",
                    "summary": "Delete user",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ]
                }
            }
        }
    });

    let generator = AdapterGenerator::new(simple_spec, Some("user-api".to_string()));
    let tools = generator.generate_tools()?;

    println!("   Generated {} tools:", tools.len());
    for tool in &tools {
        println!(
            "     - {}: {} ({} parameters)",
            tool.name,
            tool.description,
            tool.parameters.len()
        );
    }

    // Example 2: OpenAPI specification with authentication
    println!("\n2. OpenAPI specification with authentication");
    let auth_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Secure API",
            "version": "1.0.0"
        },
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                },
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer"
                }
            }
        },
        "paths": {
            "/protected": {
                "get": {
                    "operationId": "getProtectedData",
                    "summary": "Get protected data",
                    "security": [
                        {
                            "ApiKeyAuth": []
                        }
                    ]
                }
            }
        }
    });

    let auth_generator = AdapterGenerator::new(auth_spec, Some("secure-api".to_string()));
    let auth_configs = auth_generator.extract_auth_config();

    println!(
        "   Found {} authentication configurations:",
        auth_configs.len()
    );
    for auth in &auth_configs {
        println!(
            "     - Type: {}, Name: {:?}, Location: {:?}",
            auth.auth_type, auth.name, auth.location
        );
    }

    // Example 3: Generating adapter configuration
    println!("\n3. Generating adapter configuration");
    let config_yaml = generator.generate_config()?;
    println!("   Generated configuration (YAML format):");
    println!("{}", config_yaml);

    // Example 4: Complex parameter types
    println!("\n4. Complex parameter types");
    let complex_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Complex API",
            "version": "1.0.0"
        },
        "paths": {
            "/search": {
                "get": {
                    "operationId": "searchItems",
                    "summary": "Search for items with filters",
                    "parameters": [
                        {
                            "name": "query",
                            "in": "query",
                            "schema": {
                                "type": "string"
                            },
                            "description": "Search query"
                        },
                        {
                            "name": "categories",
                            "in": "query",
                            "schema": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                }
                            },
                            "description": "Filter by categories"
                        },
                        {
                            "name": "price_range",
                            "in": "query",
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "min": {"type": "number"},
                                    "max": {"type": "number"}
                                }
                            },
                            "description": "Price range filter"
                        },
                        {
                            "name": "active_only",
                            "in": "query",
                            "schema": {
                                "type": "boolean"
                            },
                            "description": "Show only active items"
                        }
                    ]
                }
            }
        }
    });

    let complex_generator = AdapterGenerator::new(complex_spec, Some("complex-api".to_string()));
    let complex_tools = complex_generator.generate_tools()?;

    if !complex_tools.is_empty() {
        let tool = &complex_tools[0];
        println!("   Tool: {} - {}", tool.name, tool.description);
        println!("   Parameters:");
        for param in &tool.parameters {
            println!(
                "     - {}: {} ({})",
                param.name,
                param.param_type.as_str(),
                if param.required {
                    "required"
                } else {
                    "optional"
                }
            );
        }
    }

    println!("\n🎉 Adapter generation example completed!");
    Ok(())
}
