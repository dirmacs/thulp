//! # thulp-core
//!
//! Core types, traits, and error definitions for the thulp execution context platform.
//!
//! This crate provides the foundational abstractions for defining, validating, and executing
//! tools in AI agent environments. It includes type-safe parameter definitions, tool metadata,
//! and extensible traits for implementing custom tool providers and transports.
//!
//! ## Core Types
//!
//! - [`ToolDefinition`]: Describes an available tool with its parameters and metadata
//! - [`ToolCall`]: Represents a request to execute a specific tool with arguments
//! - [`ToolResult`]: The result of a tool execution (success or failure)
//! - [`Parameter`]: Defines a tool parameter with type information and validation rules
//! - [`ParameterType`]: Strongly-typed parameter types (String, Integer, Number, Boolean, Array, Object)
//!
//! ## Traits
//!
//! - [`Tool`]: Trait for implementing executable tools
//! - [`Transport`]: Trait for implementing tool transport layers (e.g., MCP, HTTP, gRPC)
//!
//! ## Features
//!
//! - **Type Safety**: Compile-time and runtime validation of tool parameters
//! - **Builder Pattern**: Ergonomic API for constructing tool definitions
//! - **JSON Serialization**: Full serde support for all types
//! - **MCP Integration**: Parse MCP JSON Schema to Thulp parameter definitions
//! - **Async Support**: Built on tokio for efficient async execution
//!
//! ## Quick Start
//!
//! ### Defining a Tool
//!
//! ```rust
//! use thulp_core::{ToolDefinition, Parameter, ParameterType};
//!
//! let tool = ToolDefinition::builder("read_file")
//!     .description("Read contents of a file")
//!     .parameter(
//!         Parameter::builder("path")
//!             .param_type(ParameterType::String)
//!             .required(true)
//!             .description("Path to the file to read")
//!             .build()
//!     )
//!     .parameter(
//!         Parameter::builder("encoding")
//!             .param_type(ParameterType::String)
//!             .default(serde_json::Value::String("utf-8".to_string()))
//!             .description("File encoding")
//!             .build()
//!     )
//!     .build();
//!
//! assert_eq!(tool.name, "read_file");
//! assert_eq!(tool.parameters.len(), 2);
//! ```
//!
//! ### Validating Tool Arguments
//!
//! ```rust
//! use thulp_core::{ToolDefinition, Parameter, ParameterType};
//! use serde_json::json;
//!
//! let tool = ToolDefinition::builder("add")
//!     .description("Add two numbers")
//!     .parameter(
//!         Parameter::builder("a")
//!             .param_type(ParameterType::Number)
//!             .required(true)
//!             .build()
//!     )
//!     .parameter(
//!         Parameter::builder("b")
//!             .param_type(ParameterType::Number)
//!             .required(true)
//!             .build()
//!     )
//!     .build();
//!
//! // Valid arguments
//! let args = json!({"a": 5.0, "b": 3.0});
//! assert!(tool.validate_args(&args).is_ok());
//!
//! // Invalid - missing required parameter
//! let args = json!({"a": 5.0});
//! assert!(tool.validate_args(&args).is_err());
//!
//! // Invalid - wrong type
//! let args = json!({"a": "not a number", "b": 3.0});
//! assert!(tool.validate_args(&args).is_err());
//! ```
//!
//! ### Creating Tool Calls
//!
//! ```rust
//! use thulp_core::ToolCall;
//! use serde_json::json;
//!
//! let call = ToolCall::builder("search")
//!     .arg("query", json!("rust programming"))
//!     .arg("max_results", json!(10))
//!     .build();
//!
//! assert_eq!(call.tool, "search");
//! ```
//!
//! ### Parsing MCP JSON Schema
//!
//! ```rust
//! use thulp_core::ToolDefinition;
//! use serde_json::json;
//!
//! let schema = json!({
//!     "type": "object",
//!     "properties": {
//!         "name": {
//!             "type": "string",
//!             "description": "User name"
//!         },
//!         "age": {
//!             "type": "integer",
//!             "description": "User age"
//!         }
//!     },
//!     "required": ["name"]
//! });
//!
//! let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
//! assert_eq!(params.len(), 2);
//! assert!(params.iter().find(|p| p.name == "name").unwrap().required);
//! assert!(!params.iter().find(|p| p.name == "age").unwrap().required);
//! ```
//!
//! ## Error Handling
//!
//! All fallible operations return [`Result<T, Error>`](Result), where [`Error`] provides
//! detailed error information:
//!
//! ```rust
//! use thulp_core::{ToolDefinition, Error};
//! use serde_json::json;
//!
//! let tool = ToolDefinition::builder("test")
//!     .parameter(
//!         thulp_core::Parameter::builder("required_param")
//!             .param_type(thulp_core::ParameterType::String)
//!             .required(true)
//!             .build()
//!     )
//!     .build();
//!
//! match tool.validate_args(&json!({})) {
//!     Ok(_) => println!("Valid!"),
//!     Err(Error::MissingParameter(name)) => {
//!         eprintln!("Missing required parameter: {}", name);
//!     }
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```

mod error;
mod parameter;
mod tool;
mod traits;

pub use error::{Error, Result};
pub use parameter::{Parameter, ParameterBuilder, ParameterType};
pub use tool::{ToolCall, ToolCallBuilder, ToolDefinition, ToolDefinitionBuilder, ToolResult};
pub use traits::{Tool, Transport};
