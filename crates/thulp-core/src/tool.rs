//! Tool types for thulp.

use crate::{Error, Parameter, ParameterType, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Definition of an available tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool name (unique identifier).
    pub name: String,

    /// Human-readable description of what the tool does.
    #[serde(default)]
    pub description: String,

    /// Parameters accepted by the tool.
    #[serde(default)]
    pub parameters: Vec<Parameter>,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters: Vec::new(),
        }
    }

    /// Create a builder for a tool definition.
    pub fn builder(name: impl Into<String>) -> ToolDefinitionBuilder {
        ToolDefinitionBuilder::new(name)
    }

    /// Get a parameter by name.
    pub fn get_parameter(&self, name: &str) -> Option<&Parameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    /// Get all required parameters.
    pub fn required_parameters(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter().filter(|p| p.required)
    }

    /// Validate arguments against this tool's parameters.
    pub fn validate_args(&self, args: &Value) -> Result<()> {
        let empty_map = serde_json::Map::new();
        let args_obj = args.as_object().unwrap_or(&empty_map);

        // Check required parameters
        for param in self.required_parameters() {
            if !args_obj.contains_key(&param.name) {
                // Check if there's a default
                if param.default.is_none() {
                    return Err(Error::MissingParameter(param.name.clone()));
                }
            }
        }

        // Check parameter types
        for (key, value) in args_obj {
            if let Some(param) = self.get_parameter(key) {
                if !param.param_type.matches(value) {
                    return Err(Error::InvalidParameterType {
                        name: key.clone(),
                        expected: param.param_type.as_str().to_string(),
                        actual: json_type_name(value).to_string(),
                    });
                }

                // Check enum values if defined
                if !param.enum_values.is_empty() && !param.enum_values.contains(value) {
                    return Err(Error::InvalidConfig(format!(
                        "parameter '{}' must be one of: {:?}",
                        key, param.enum_values
                    )));
                }
            }
        }

        Ok(())
    }

    /// Parse MCP inputSchema into Parameters
    pub fn parse_mcp_input_schema(schema: &serde_json::Value) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if let Some(properties) = schema.get("properties") {
            if let Some(props_obj) = properties.as_object() {
                for (name, prop) in props_obj {
                    let param_type = if let Some(type_val) = prop.get("type") {
                        match type_val.as_str() {
                            Some("string") => ParameterType::String,
                            Some("integer") => ParameterType::Integer,
                            Some("number") => ParameterType::Number,
                            Some("boolean") => ParameterType::Boolean,
                            Some("array") => ParameterType::Array,
                            Some("object") => ParameterType::Object,
                            _ => ParameterType::String,
                        }
                    } else {
                        ParameterType::String
                    };

                    let description = prop
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let required = if let Some(req_array) = schema.get("required") {
                        if let Some(arr) = req_array.as_array() {
                            arr.iter().any(|v| v.as_str() == Some(name))
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    params.push(Parameter {
                        name: name.to_string(),
                        param_type,
                        description,
                        required,
                        default: None,
                        enum_values: vec![],
                    });
                }
            }
        }

        Ok(params)
    }
}

/// Get the JSON type name for a value.
fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Builder for [`ToolDefinition`].
#[derive(Debug, Default)]
pub struct ToolDefinitionBuilder {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
}

impl ToolDefinitionBuilder {
    /// Create a new tool definition builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the tool description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a parameter.
    pub fn parameter(mut self, parameter: Parameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Add multiple parameters.
    pub fn parameters(mut self, parameters: impl IntoIterator<Item = Parameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    /// Build the tool definition.
    pub fn build(self) -> ToolDefinition {
        ToolDefinition {
            name: self.name,
            description: self.description,
            parameters: self.parameters,
        }
    }
}

/// A request to execute a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// The name of the tool to call.
    pub tool: String,

    /// Arguments to pass to the tool.
    #[serde(default)]
    pub arguments: Value,
}

impl ToolCall {
    /// Create a new tool call.
    pub fn new(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            arguments: Value::Object(serde_json::Map::new()),
        }
    }

    /// Create a tool call with arguments.
    pub fn with_args(tool: impl Into<String>, arguments: Value) -> Self {
        Self {
            tool: tool.into(),
            arguments,
        }
    }

    /// Create a builder for a tool call.
    pub fn builder(tool: impl Into<String>) -> ToolCallBuilder {
        ToolCallBuilder::new(tool)
    }
}

/// Builder for [`ToolCall`].
#[derive(Debug, Default)]
pub struct ToolCallBuilder {
    tool: String,
    arguments: serde_json::Map<String, Value>,
}

impl ToolCallBuilder {
    /// Create a new tool call builder.
    pub fn new(tool: impl Into<String>) -> Self {
        Self {
            tool: tool.into(),
            arguments: serde_json::Map::new(),
        }
    }

    /// Add an argument.
    pub fn arg(mut self, name: impl Into<String>, value: Value) -> Self {
        self.arguments.insert(name.into(), value);
        self
    }

    /// Add a string argument.
    pub fn arg_str(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.arguments
            .insert(name.into(), Value::String(value.into()));
        self
    }

    /// Add an integer argument.
    pub fn arg_int(mut self, name: impl Into<String>, value: i64) -> Self {
        self.arguments
            .insert(name.into(), Value::Number(value.into()));
        self
    }

    /// Add a boolean argument.
    pub fn arg_bool(mut self, name: impl Into<String>, value: bool) -> Self {
        self.arguments.insert(name.into(), Value::Bool(value));
        self
    }

    /// Build the tool call.
    pub fn build(self) -> ToolCall {
        ToolCall {
            tool: self.tool,
            arguments: Value::Object(self.arguments),
        }
    }
}

/// The result of a tool execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the execution was successful.
    pub success: bool,

    /// The result data (if successful).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    /// Error message (if failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    /// Execution duration in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

impl ToolResult {
    /// Create a successful result.
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            duration_ms: None,
        }
    }

    /// Create a failed result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            duration_ms: None,
        }
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Check if the result is successful.
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the data, returning an error if the result failed.
    pub fn into_data(self) -> Result<Value> {
        if self.success {
            self.data
                .ok_or_else(|| Error::ExecutionFailed("no data returned".to_string()))
        } else {
            Err(Error::ExecutionFailed(
                self.error.unwrap_or_else(|| "unknown error".to_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParameterType;
    use serde_json::json;

    #[test]
    fn tool_definition_new() {
        let tool = ToolDefinition::new("test_tool");
        assert_eq!(tool.name, "test_tool");
        assert!(tool.description.is_empty());
        assert!(tool.parameters.is_empty());
    }

    #[test]
    fn tool_definition_builder() {
        let tool = ToolDefinition::builder("read_file")
            .description("Read a file")
            .parameter(Parameter::required_string("path"))
            .build();

        assert_eq!(tool.name, "read_file");
        assert_eq!(tool.description, "Read a file");
        assert_eq!(tool.parameters.len(), 1);
    }

    #[test]
    fn tool_definition_get_parameter() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("path"))
            .parameter(Parameter::optional_string("encoding"))
            .build();

        assert!(tool.get_parameter("path").is_some());
        assert!(tool.get_parameter("encoding").is_some());
        assert!(tool.get_parameter("nonexistent").is_none());
    }

    #[test]
    fn tool_definition_required_parameters() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("required1"))
            .parameter(Parameter::optional_string("optional1"))
            .parameter(Parameter::required_string("required2"))
            .build();

        let required: Vec<_> = tool.required_parameters().collect();
        assert_eq!(required.len(), 2);
        assert!(required.iter().any(|p| p.name == "required1"));
        assert!(required.iter().any(|p| p.name == "required2"));
    }

    #[test]
    fn tool_definition_validate_args_success() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("name"))
            .parameter(
                Parameter::builder("count")
                    .param_type(ParameterType::Integer)
                    .build(),
            )
            .build();

        let args = json!({"name": "test", "count": 5});
        assert!(tool.validate_args(&args).is_ok());
    }

    #[test]
    fn tool_definition_validate_args_missing_required() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("name"))
            .build();

        let args = json!({});
        let result = tool.validate_args(&args);
        assert!(matches!(result, Err(Error::MissingParameter(_))));
    }

    #[test]
    fn tool_definition_validate_args_with_default() {
        let tool = ToolDefinition::builder("test")
            .parameter(
                Parameter::builder("count")
                    .required(true)
                    .default(json!(10))
                    .build(),
            )
            .build();

        let args = json!({});
        assert!(tool.validate_args(&args).is_ok());
    }

    #[test]
    fn tool_definition_validate_args_wrong_type() {
        let tool = ToolDefinition::builder("test")
            .parameter(
                Parameter::builder("count")
                    .param_type(ParameterType::Integer)
                    .build(),
            )
            .build();

        let args = json!({"count": "not a number"});
        let result = tool.validate_args(&args);
        assert!(matches!(result, Err(Error::InvalidParameterType { .. })));
    }

    #[test]
    fn tool_definition_validate_args_enum() {
        let tool = ToolDefinition::builder("test")
            .parameter(
                Parameter::builder("format")
                    .enum_value(json!("json"))
                    .enum_value(json!("yaml"))
                    .build(),
            )
            .build();

        assert!(tool.validate_args(&json!({"format": "json"})).is_ok());
        assert!(tool.validate_args(&json!({"format": "yaml"})).is_ok());
        assert!(tool.validate_args(&json!({"format": "toml"})).is_err());
    }

    #[test]
    fn tool_call_new() {
        let call = ToolCall::new("test_tool");
        assert_eq!(call.tool, "test_tool");
        assert!(call.arguments.is_object());
    }

    #[test]
    fn tool_call_with_args() {
        let args = json!({"path": "/tmp/test.txt"});
        let call = ToolCall::with_args("read_file", args.clone());
        assert_eq!(call.tool, "read_file");
        assert_eq!(call.arguments, args);
    }

    #[test]
    fn tool_call_builder() {
        let call = ToolCall::builder("github.list_repos")
            .arg_str("owner", "octocat")
            .arg_int("per_page", 10)
            .arg_bool("include_forks", false)
            .build();

        assert_eq!(call.tool, "github.list_repos");
        assert_eq!(call.arguments["owner"], "octocat");
        assert_eq!(call.arguments["per_page"], 10);
        assert_eq!(call.arguments["include_forks"], false);
    }

    #[test]
    fn tool_result_success() {
        let result = ToolResult::success(json!({"status": "ok"}));
        assert!(result.is_success());
        assert!(result.data.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn tool_result_failure() {
        let result = ToolResult::failure("Something went wrong");
        assert!(!result.is_success());
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn tool_result_with_duration() {
        let result = ToolResult::success(json!(null)).with_duration(250);
        assert_eq!(result.duration_ms, Some(250));
    }

    #[test]
    fn tool_result_into_data_success() {
        let result = ToolResult::success(json!({"value": 42}));
        let data = result.into_data().unwrap();
        assert_eq!(data["value"], 42);
    }

    #[test]
    fn tool_result_into_data_failure() {
        let result = ToolResult::failure("error");
        let err = result.into_data().unwrap_err();
        assert!(matches!(err, Error::ExecutionFailed(_)));
    }

    #[test]
    fn tool_definition_serialization() {
        let tool = ToolDefinition::builder("test")
            .description("A test tool")
            .parameter(Parameter::required_string("name"))
            .build();

        let json = serde_json::to_string(&tool).unwrap();
        let parsed: ToolDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(tool, parsed);
    }

    #[test]
    fn tool_definition_validate_args_edge_cases() {
        let tool = ToolDefinition::builder("test")
            .parameter(
                Parameter::builder("array_param")
                    .param_type(ParameterType::Array)
                    .required(true)
                    .build(),
            )
            .parameter(
                Parameter::builder("object_param")
                    .param_type(ParameterType::Object)
                    .required(false)
                    .default(json!({}))
                    .build(),
            )
            .build();

        // Valid array and object
        assert!(tool
            .validate_args(&json!({"array_param": [1, 2, 3], "object_param": {"key": "value"}}))
            .is_ok());

        // Valid with default for optional object
        assert!(tool.validate_args(&json!({"array_param": []})).is_ok());

        // Invalid array type (should be array, not object)
        assert!(tool.validate_args(&json!({"array_param": {}})).is_err());

        // Invalid object type (should be object, not string)
        assert!(tool
            .validate_args(&json!({"array_param": [], "object_param": "not an object"}))
            .is_err());
    }

    #[test]
    fn tool_definition_validate_args_with_all_types() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("str_param"))
            .parameter(
                Parameter::builder("int_param")
                    .param_type(ParameterType::Integer)
                    .required(true)
                    .build(),
            )
            .parameter(
                Parameter::builder("num_param")
                    .param_type(ParameterType::Number)
                    .required(true)
                    .build(),
            )
            .parameter(
                Parameter::builder("bool_param")
                    .param_type(ParameterType::Boolean)
                    .required(true)
                    .build(),
            )
            .parameter(
                Parameter::builder("arr_param")
                    .param_type(ParameterType::Array)
                    .required(true)
                    .build(),
            )
            .parameter(
                Parameter::builder("obj_param")
                    .param_type(ParameterType::Object)
                    .required(true)
                    .build(),
            )
            .build();

        let args = json!({
            "str_param": "test",
            "int_param": 42,
            "num_param": 3.14,
            "bool_param": true,
            "arr_param": [1, 2, 3],
            "obj_param": {"key": "value"}
        });

        assert!(tool.validate_args(&args).is_ok());

        // Test each wrong type individually
        assert!(tool.validate_args(&json!({"str_param": 42, "int_param": 42, "num_param": 3.14, "bool_param": true, "arr_param": [], "obj_param": {}})).is_err());
        assert!(tool.validate_args(&json!({"str_param": "test", "int_param": "not int", "num_param": 3.14, "bool_param": true, "arr_param": [], "obj_param": {}})).is_err());
    }

    #[test]
    fn tool_definition_validate_args_empty_required() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("param1"))
            .parameter(Parameter::required_string("param2"))
            .parameter(Parameter::required_string("param3"))
            .build();

        // All missing
        assert!(tool.validate_args(&json!({})).is_err());

        // Partial missing
        assert!(tool.validate_args(&json!({"param1": "value"})).is_err());

        // All present
        assert!(tool
            .validate_args(&json!({"param1": "v1", "param2": "v2", "param3": "v3"}))
            .is_ok());
    }

    #[test]
    fn tool_call_serialization() {
        let call = ToolCall::builder("test").arg_str("name", "value").build();

        let json = serde_json::to_string(&call).unwrap();
        let parsed: ToolCall = serde_json::from_str(&json).unwrap();

        assert_eq!(call, parsed);
    }

    #[test]
    fn tool_result_serialization() {
        let result = ToolResult::success(json!({"data": [1, 2, 3]})).with_duration(100);

        let json = serde_json::to_string(&result).unwrap();
        let parsed: ToolResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result, parsed);
    }

    #[test]
    fn parse_mcp_input_schema_basic() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name"
                },
                "age": {
                    "type": "integer",
                    "description": "The age"
                }
            },
            "required": ["name"]
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 2);

        let name_param = params.iter().find(|p| p.name == "name").unwrap();
        assert_eq!(name_param.param_type, ParameterType::String);
        assert_eq!(name_param.description, "The name");
        assert!(name_param.required);

        let age_param = params.iter().find(|p| p.name == "age").unwrap();
        assert_eq!(age_param.param_type, ParameterType::Integer);
        assert_eq!(age_param.description, "The age");
        assert!(!age_param.required);
    }

    #[test]
    fn parse_mcp_input_schema_all_types() {
        let schema = json!({
            "type": "object",
            "properties": {
                "str": {"type": "string"},
                "num": {"type": "number"},
                "int": {"type": "integer"},
                "bool": {"type": "boolean"},
                "arr": {"type": "array"},
                "obj": {"type": "object"}
            }
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 6);

        assert_eq!(
            params.iter().find(|p| p.name == "str").unwrap().param_type,
            ParameterType::String
        );
        assert_eq!(
            params.iter().find(|p| p.name == "num").unwrap().param_type,
            ParameterType::Number
        );
        assert_eq!(
            params.iter().find(|p| p.name == "int").unwrap().param_type,
            ParameterType::Integer
        );
        assert_eq!(
            params.iter().find(|p| p.name == "bool").unwrap().param_type,
            ParameterType::Boolean
        );
        assert_eq!(
            params.iter().find(|p| p.name == "arr").unwrap().param_type,
            ParameterType::Array
        );
        assert_eq!(
            params.iter().find(|p| p.name == "obj").unwrap().param_type,
            ParameterType::Object
        );
    }

    #[test]
    fn parse_mcp_input_schema_empty() {
        let schema = json!({});
        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_mcp_input_schema_no_properties() {
        let schema = json!({"type": "object"});
        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn parse_mcp_input_schema_unknown_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "unknown": {"type": "unknown_type"}
            }
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 1);
        // Unknown types default to String
        assert_eq!(params[0].param_type, ParameterType::String);
    }

    #[test]
    fn parse_mcp_input_schema_missing_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "field": {"description": "A field without type"}
            }
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 1);
        // Missing type defaults to String
        assert_eq!(params[0].param_type, ParameterType::String);
    }

    #[test]
    fn parse_mcp_input_schema_all_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "field1": {"type": "string"},
                "field2": {"type": "string"},
                "field3": {"type": "string"}
            },
            "required": ["field1", "field2", "field3"]
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 3);
        assert!(params.iter().all(|p| p.required));
    }

    #[test]
    fn parse_mcp_input_schema_no_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "field1": {"type": "string"},
                "field2": {"type": "string"}
            }
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 2);
        assert!(params.iter().all(|p| !p.required));
    }

    #[test]
    fn parse_mcp_input_schema_no_description() {
        let schema = json!({
            "type": "object",
            "properties": {
                "field": {"type": "string"}
            }
        });

        let params = ToolDefinition::parse_mcp_input_schema(&schema).unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].description, "");
    }
}
