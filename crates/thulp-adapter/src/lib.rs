//! # thulp-adapter
//!
//! Adapter generation framework for converting OpenAPI specifications into Thulp tool definitions.
//!
//! This crate provides functionality to:
//! - Parse OpenAPI v2/v3 specifications
//! - Convert API endpoints into Thulp tool definitions
//! - Extract authentication requirements
//! - Generate adapter configuration files
//!
//! ## Example
//!
//! ```rust
//! use thulp_adapter::AdapterGenerator;
//! use serde_json::Value;
//!
//! // Load an OpenAPI specification
//! let spec: Value = serde_json::from_str(r#"{
//!   "openapi": "3.0.0",
//!   "info": {"title": "Test API", "version": "1.0.0"},
//!   "paths": {
//!     "/users": {
//!       "get": {
//!         "operationId": "listUsers",
//!         "summary": "List all users",
//!         "responses": {
//!           "200": {
//!             "description": "A list of users"
//!           }
//!         }
//!       }
//!     }
//!   }
//! }"#).unwrap();
//!
//! // Create an adapter generator
//! let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
//!
//! // Generate tool definitions
//! let tools = generator.generate_tools().unwrap();
//! println!("Generated {} tools", tools.len());
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thulp_core::{Parameter, ParameterType, ToolDefinition};

/// Result type for adapter operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Authentication configuration extracted from OpenAPI spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Authentication type (apiKey, http, oauth2, openIdConnect)
    pub auth_type: String,

    /// Authentication scheme (for http auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,

    /// Parameter name (for apiKey auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Location of the API key (query, header, cookie)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Adapter generator for converting OpenAPI specs to Thulp tools
pub struct AdapterGenerator {
    /// The OpenAPI specification
    openapi_spec: Value,

    /// Provider name
    provider_name: String,
}

impl AdapterGenerator {
    /// Create a new adapter generator from an OpenAPI specification
    pub fn new(openapi_spec: Value, provider_name: Option<String>) -> Self {
        let provider_name = provider_name.unwrap_or_else(|| "unknown-provider".to_string());

        Self {
            openapi_spec,
            provider_name,
        }
    }

    /// Create a new adapter generator from a URL
    pub async fn from_url(spec_url: &str, provider_name: Option<String>) -> Result<Self> {
        use reqwest::Client;

        let provider_name = provider_name.unwrap_or_else(|| "unknown-provider".to_string());
        let client = Client::new();
        let spec: Value = client
            .get(spec_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch OpenAPI spec: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAPI spec: {}", e))?;

        Ok(Self {
            openapi_spec: spec,
            provider_name,
        })
    }

    /// Generate Thulp tool definitions from the OpenAPI specification
    pub fn generate_tools(&self) -> Result<Vec<ToolDefinition>> {
        let mut tools = Vec::new();
        let spec = &self.openapi_spec;

        // Get paths from the specification
        let paths = spec
            .get("paths")
            .and_then(|p| p.as_object())
            .ok_or("No paths found in OpenAPI specification")?;

        // Process each path
        for (path, path_item) in paths {
            let path_item = path_item
                .as_object()
                .ok_or(format!("Invalid path item for path: {}", path))?;

            // Process each HTTP method (get, post, put, delete, etc.)
            for (method, operation) in path_item {
                if !matches!(method.as_str(), "get" | "post" | "put" | "delete" | "patch") {
                    continue;
                }

                if let Some(tool) = self.operation_to_tool(path, method, operation)? {
                    tools.push(tool);
                }
            }
        }

        Ok(tools)
    }

    /// Convert a single OpenAPI operation to a Thulp tool definition
    fn operation_to_tool(
        &self,
        path: &str,
        method: &str,
        operation: &Value,
    ) -> Result<Option<ToolDefinition>> {
        let operation = operation
            .as_object()
            .ok_or(format!("Invalid operation for {} {}", method, path))?;

        // Get operation ID or generate one
        let operation_id = operation
            .get("operationId")
            .and_then(|id| id.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}_{}", method, self.sanitize_path(path)));

        // Get description
        let summary = operation
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        let description = operation
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or(summary);

        // Build parameters
        let mut parameters = Vec::new();

        // Add path parameters
        if let Some(path_params) = self.extract_path_parameters(path) {
            parameters.extend(path_params);
        }

        // Add query parameters from operation
        if let Some(query_params) = operation.get("parameters").and_then(|p| p.as_array()) {
            for param in query_params {
                if let Some(param_def) = self.parameter_to_tool_parameter(param)? {
                    parameters.push(param_def);
                }
            }
        }

        // Add request body as a parameter (for POST, PUT, PATCH)
        if matches!(method, "post" | "put" | "patch") {
            if let Some(body_param) = self.request_body_to_parameter(operation)? {
                parameters.push(body_param);
            }
        }

        let tool = ToolDefinition {
            name: operation_id,
            description: description.to_string(),
            parameters,
        };

        Ok(Some(tool))
    }

    /// Extract path parameters from a path string
    fn extract_path_parameters(&self, path: &str) -> Option<Vec<Parameter>> {
        let mut parameters = Vec::new();
        let param_pattern = regex::Regex::new(r"\{([^}]+)\}").unwrap();

        for capture in param_pattern.captures_iter(path) {
            if let Some(param_name) = capture.get(1) {
                let param = Parameter::builder(param_name.as_str())
                    .param_type(ParameterType::String)
                    .required(true)
                    .description(format!("Path parameter: {}", param_name.as_str()))
                    .build();
                parameters.push(param);
            }
        }

        if parameters.is_empty() {
            None
        } else {
            Some(parameters)
        }
    }

    /// Convert an OpenAPI parameter to a Thulp parameter
    fn parameter_to_tool_parameter(&self, param: &Value) -> Result<Option<Parameter>> {
        let param = param.as_object().ok_or("Invalid parameter definition")?;

        let name = param
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or("Parameter missing name")?;

        let param_type = param
            .get("schema")
            .and_then(|s| s.get("type"))
            .and_then(|t| t.as_str())
            .map(|t| self.parse_parameter_type(t))
            .unwrap_or(ParameterType::String);

        let required = param
            .get("required")
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let description = param
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");

        let param_builder = Parameter::builder(name)
            .param_type(param_type)
            .required(required)
            .description(description);

        Ok(Some(param_builder.build()))
    }

    /// Convert request body to a parameter
    fn request_body_to_parameter(
        &self,
        operation: &serde_json::Map<String, Value>,
    ) -> Result<Option<Parameter>> {
        if let Some(request_body) = operation.get("requestBody") {
            let request_body = request_body.as_object().ok_or("Invalid requestBody")?;

            if let Some(content) = request_body.get("content") {
                let content = content.as_object().ok_or("Invalid requestBody content")?;

                // For simplicity, just use the first content type
                if let Some((media_type, _)) = content.iter().next() {
                    let param = Parameter::builder("data")
                        .param_type(ParameterType::Object)
                        .description(format!("Request body ({} media type)", media_type))
                        .build();

                    return Ok(Some(param));
                }
            }
        }

        Ok(None)
    }

    /// Parse parameter type from OpenAPI type string
    fn parse_parameter_type(&self, openapi_type: &str) -> ParameterType {
        match openapi_type {
            "integer" => ParameterType::Integer,
            "number" => ParameterType::Number,
            "boolean" => ParameterType::Boolean,
            "array" => ParameterType::Array,
            "object" => ParameterType::Object,
            _ => ParameterType::String,
        }
    }

    /// Sanitize a path for use as an operation ID
    fn sanitize_path(&self, path: &str) -> String {
        path.replace("/", "_")
            .replace("-", "_")
            .replace("{", "")
            .replace("}", "")
            .trim_start_matches('_')
            .to_string()
    }

    /// Extract authentication configuration from the OpenAPI specification
    pub fn extract_auth_config(&self) -> Vec<AuthConfig> {
        let mut auth_configs = Vec::new();
        let spec = &self.openapi_spec;

        // Check for components.securitySchemes (OpenAPI 3.x)
        if let Some(components) = spec.get("components") {
            if let Some(security_schemes) = components.get("securitySchemes") {
                if let Some(schemes) = security_schemes.as_object() {
                    for (name, scheme) in schemes {
                        if let Some(auth_config) = self.parse_security_scheme(name, scheme) {
                            auth_configs.push(auth_config);
                        }
                    }
                }
            }
        }

        // Check for security definitions (OpenAPI 2.x)
        if let Some(security_definitions) = spec.get("securityDefinitions") {
            if let Some(definitions) = security_definitions.as_object() {
                for (name, scheme) in definitions {
                    if let Some(auth_config) = self.parse_security_scheme(name, scheme) {
                        auth_configs.push(auth_config);
                    }
                }
            }
        }

        auth_configs
    }

    /// Parse a security scheme into an AuthConfig
    fn parse_security_scheme(&self, _name: &str, scheme: &Value) -> Option<AuthConfig> {
        let scheme = scheme.as_object()?;

        let auth_type = scheme.get("type").and_then(|t| t.as_str())?.to_string();

        let scheme_name = scheme
            .get("scheme")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let param_name = scheme
            .get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.to_string());
        let location = scheme
            .get("in")
            .and_then(|i| i.as_str())
            .map(|i| i.to_string());

        Some(AuthConfig {
            auth_type,
            scheme: scheme_name,
            name: param_name,
            location,
        })
    }

    /// Generate adapter configuration as YAML
    pub fn generate_config(&self) -> Result<String> {
        let tools = self.generate_tools()?;
        let config = AdapterConfig {
            name: self.provider_name.clone(),
            version: "1.0.0".to_string(),
            auth: self.extract_auth_config(),
            tools,
        };

        Ok(serde_yaml::to_string(&config)?)
    }
}

/// Adapter configuration structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdapterConfig {
    /// Provider name
    name: String,

    /// Adapter version
    version: String,

    /// Authentication configuration
    #[serde(default)]
    auth: Vec<AuthConfig>,

    /// Generated tools
    tools: Vec<ToolDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_generator_creation() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "paths": {}
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        assert_eq!(generator.provider_name, "test-api");
    }

    #[test]
    fn test_auth_config_serialization() {
        let auth_config = AuthConfig {
            auth_type: "apiKey".to_string(),
            scheme: None,
            name: Some("X-API-Key".to_string()),
            location: Some("header".to_string()),
        };

        let yaml = serde_yaml::to_string(&auth_config).unwrap();
        assert!(yaml.contains("auth_type: apiKey"));
        assert!(yaml.contains("name: X-API-Key"));
        assert!(yaml.contains("location: header"));
    }

    #[test]
    fn test_generate_tools_simple() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "listUsers",
                        "summary": "List all users",
                        "parameters": [
                            {
                                "name": "limit",
                                "in": "query",
                                "schema": {"type": "integer"},
                                "required": false
                            }
                        ]
                    }
                }
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let tools = generator.generate_tools().unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "listUsers");
        assert!(tools[0].description.contains("List all users"));
        assert_eq!(tools[0].parameters.len(), 1);
        assert_eq!(tools[0].parameters[0].name, "limit");
    }

    #[test]
    fn test_generate_tools_with_path_params() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "paths": {
                "/users/{id}": {
                    "get": {
                        "operationId": "getUser",
                        "summary": "Get user by ID"
                    }
                }
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let tools = generator.generate_tools().unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "getUser");
        assert_eq!(tools[0].parameters.len(), 1);
        assert_eq!(tools[0].parameters[0].name, "id");
        assert!(tools[0].parameters[0].required);
    }

    #[test]
    fn test_generate_tools_multiple_methods() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "listUsers",
                        "summary": "List all users"
                    },
                    "post": {
                        "operationId": "createUser",
                        "summary": "Create a new user"
                    },
                    "put": {
                        "operationId": "updateUser",
                        "summary": "Update a user"
                    },
                    "delete": {
                        "operationId": "deleteUser",
                        "summary": "Delete a user"
                    }
                }
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let tools = generator.generate_tools().unwrap();

        assert_eq!(tools.len(), 4);
        let tool_names: Vec<_> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"listUsers".to_string()));
        assert!(tool_names.contains(&"createUser".to_string()));
        assert!(tool_names.contains(&"updateUser".to_string()));
        assert!(tool_names.contains(&"deleteUser".to_string()));
    }

    #[test]
    fn test_generate_tools_complex_spec() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "paths": {
                "/users/{id}/posts/{postId}": {
                    "get": {
                        "operationId": "getUserPost",
                        "summary": "Get a specific user post",
                        "parameters": [
                            {
                                "name": "filter",
                                "in": "query",
                                "schema": {"type": "string"},
                                "required": false
                            }
                        ]
                    }
                }
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let tools = generator.generate_tools().unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].parameters.len(), 3); // 2 path params + 1 query param

        let param_names: Vec<_> = tools[0].parameters.iter().map(|p| p.name.clone()).collect();
        assert!(param_names.contains(&"id".to_string()));
        assert!(param_names.contains(&"postId".to_string()));
        assert!(param_names.contains(&"filter".to_string()));
    }

    #[test]
    fn test_extract_auth_config_apikey() {
        let spec = serde_json::json!({
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
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let auth_configs = generator.extract_auth_config();

        assert_eq!(auth_configs.len(), 1);
        assert_eq!(auth_configs[0].auth_type, "apiKey");
        assert_eq!(auth_configs[0].name, Some("X-API-Key".to_string()));
        assert_eq!(auth_configs[0].location, Some("header".to_string()));
    }

    #[test]
    fn test_extract_auth_config_http() {
        let spec = serde_json::json!({
            "openapi": "3.0.0",
            "info": {"title": "Test API", "version": "1.0.0"},
            "components": {
                "securitySchemes": {
                    "BasicAuth": {
                        "type": "http",
                        "scheme": "basic"
                    },
                    "BearerAuth": {
                        "type": "http",
                        "scheme": "bearer"
                    }
                }
            }
        });

        let generator = AdapterGenerator::new(spec, Some("test-api".to_string()));
        let auth_configs = generator.extract_auth_config();

        assert_eq!(auth_configs.len(), 2);
        assert!(auth_configs
            .iter()
            .any(|a| a.auth_type == "http" && a.scheme == Some("basic".to_string())));
        assert!(auth_configs
            .iter()
            .any(|a| a.auth_type == "http" && a.scheme == Some("bearer".to_string())));
    }

    #[test]
    fn test_sanitize_path() {
        let generator = AdapterGenerator::new(serde_json::json!({}), Some("test".to_string()));

        assert_eq!(
            generator.sanitize_path("/users/{id}/posts"),
            "users_id_posts"
        );
        assert_eq!(generator.sanitize_path("/api/v1/items"), "api_v1_items");
        assert_eq!(
            generator.sanitize_path("/users/{userId}/profiles/{profileId}"),
            "users_userId_profiles_profileId"
        );
    }

    #[test]
    fn test_parse_parameter_type() {
        let generator = AdapterGenerator::new(serde_json::json!({}), Some("test".to_string()));

        assert_eq!(
            generator.parse_parameter_type("integer"),
            ParameterType::Integer
        );
        assert_eq!(
            generator.parse_parameter_type("number"),
            ParameterType::Number
        );
        assert_eq!(
            generator.parse_parameter_type("boolean"),
            ParameterType::Boolean
        );
        assert_eq!(
            generator.parse_parameter_type("array"),
            ParameterType::Array
        );
        assert_eq!(
            generator.parse_parameter_type("object"),
            ParameterType::Object
        );
        assert_eq!(
            generator.parse_parameter_type("string"),
            ParameterType::String
        );
        assert_eq!(
            generator.parse_parameter_type("unknown"),
            ParameterType::String
        );
    }
}
