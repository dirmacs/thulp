//! Parameter types for tool definitions.

use serde::{Deserialize, Serialize};

/// The type of a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    /// String parameter.
    #[default]
    String,
    /// Integer parameter.
    Integer,
    /// Number (float) parameter.
    Number,
    /// Boolean parameter.
    Boolean,
    /// Array parameter.
    Array,
    /// Object parameter.
    Object,
}

impl ParameterType {
    /// Returns the type name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Object => "object",
        }
    }

    /// Checks if a JSON value matches this parameter type.
    pub fn matches(&self, value: &serde_json::Value) -> bool {
        match (self, value) {
            (Self::String, serde_json::Value::String(_)) => true,
            (Self::Integer, serde_json::Value::Number(n)) => n.is_i64() || n.is_u64(),
            (Self::Number, serde_json::Value::Number(_)) => true,
            (Self::Boolean, serde_json::Value::Bool(_)) => true,
            (Self::Array, serde_json::Value::Array(_)) => true,
            (Self::Object, serde_json::Value::Object(_)) => true,
            _ => false,
        }
    }
}

/// A parameter definition for a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    /// The parameter name.
    pub name: String,

    /// The parameter type.
    #[serde(rename = "type", default)]
    pub param_type: ParameterType,

    /// Whether this parameter is required.
    #[serde(default)]
    pub required: bool,

    /// Description of the parameter.
    #[serde(default)]
    pub description: String,

    /// Default value for the parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Enum of allowed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<serde_json::Value>,
}

impl Parameter {
    /// Create a new parameter with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::default(),
            required: false,
            description: String::new(),
            default: None,
            enum_values: Vec::new(),
        }
    }

    /// Create a builder for a parameter.
    pub fn builder(name: impl Into<String>) -> ParameterBuilder {
        ParameterBuilder::new(name)
    }

    /// Create a required string parameter.
    pub fn required_string(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::String,
            required: true,
            description: String::new(),
            default: None,
            enum_values: Vec::new(),
        }
    }

    /// Create an optional string parameter.
    pub fn optional_string(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: ParameterType::String,
            required: false,
            description: String::new(),
            default: None,
            enum_values: Vec::new(),
        }
    }
}

/// Builder for [`Parameter`].
#[derive(Debug, Default)]
pub struct ParameterBuilder {
    name: String,
    param_type: ParameterType,
    required: bool,
    description: String,
    default: Option<serde_json::Value>,
    enum_values: Vec<serde_json::Value>,
}

impl ParameterBuilder {
    /// Create a new parameter builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the parameter type.
    pub fn param_type(mut self, param_type: ParameterType) -> Self {
        self.param_type = param_type;
        self
    }

    /// Set whether the parameter is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the parameter description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the default value.
    pub fn default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self
    }

    /// Add an enum value.
    pub fn enum_value(mut self, value: serde_json::Value) -> Self {
        self.enum_values.push(value);
        self
    }

    /// Build the parameter.
    pub fn build(self) -> Parameter {
        Parameter {
            name: self.name,
            param_type: self.param_type,
            required: self.required,
            description: self.description,
            default: self.default,
            enum_values: self.enum_values,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parameter_type_as_str() {
        assert_eq!(ParameterType::String.as_str(), "string");
        assert_eq!(ParameterType::Integer.as_str(), "integer");
        assert_eq!(ParameterType::Number.as_str(), "number");
        assert_eq!(ParameterType::Boolean.as_str(), "boolean");
        assert_eq!(ParameterType::Array.as_str(), "array");
        assert_eq!(ParameterType::Object.as_str(), "object");
    }

    #[test]
    fn parameter_type_matches_string() {
        assert!(ParameterType::String.matches(&json!("hello")));
        assert!(!ParameterType::String.matches(&json!(123)));
        assert!(!ParameterType::String.matches(&json!(true)));
    }

    #[test]
    fn parameter_type_matches_integer() {
        assert!(ParameterType::Integer.matches(&json!(123)));
        assert!(ParameterType::Integer.matches(&json!(-456)));
        assert!(!ParameterType::Integer.matches(&json!(1.5)));
        assert!(!ParameterType::Integer.matches(&json!("123")));
    }

    #[test]
    fn parameter_type_matches_number() {
        assert!(ParameterType::Number.matches(&json!(123)));
        assert!(ParameterType::Number.matches(&json!(1.5)));
        assert!(ParameterType::Number.matches(&json!(-3.14)));
        assert!(!ParameterType::Number.matches(&json!("1.5")));
    }

    #[test]
    fn parameter_type_matches_boolean() {
        assert!(ParameterType::Boolean.matches(&json!(true)));
        assert!(ParameterType::Boolean.matches(&json!(false)));
        assert!(!ParameterType::Boolean.matches(&json!("true")));
        assert!(!ParameterType::Boolean.matches(&json!(1)));
    }

    #[test]
    fn parameter_type_matches_array() {
        assert!(ParameterType::Array.matches(&json!([1, 2, 3])));
        assert!(ParameterType::Array.matches(&json!([])));
        assert!(!ParameterType::Array.matches(&json!({"a": 1})));
    }

    #[test]
    fn parameter_type_matches_object() {
        assert!(ParameterType::Object.matches(&json!({"a": 1})));
        assert!(ParameterType::Object.matches(&json!({})));
        assert!(!ParameterType::Object.matches(&json!([1, 2])));
    }

    #[test]
    fn parameter_new() {
        let param = Parameter::new("test");
        assert_eq!(param.name, "test");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(!param.required);
        assert!(param.description.is_empty());
        assert!(param.default.is_none());
    }

    #[test]
    fn parameter_required_string() {
        let param = Parameter::required_string("username");
        assert_eq!(param.name, "username");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(param.required);
    }

    #[test]
    fn parameter_optional_string() {
        let param = Parameter::optional_string("limit");
        assert_eq!(param.name, "limit");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(!param.required);
    }

    #[test]
    fn parameter_builder() {
        let param = Parameter::builder("count")
            .param_type(ParameterType::Integer)
            .required(true)
            .description("Number of items")
            .default(json!(10))
            .build();

        assert_eq!(param.name, "count");
        assert_eq!(param.param_type, ParameterType::Integer);
        assert!(param.required);
        assert_eq!(param.description, "Number of items");
        assert_eq!(param.default, Some(json!(10)));
    }

    #[test]
    fn parameter_builder_with_enum() {
        let param = Parameter::builder("format")
            .param_type(ParameterType::String)
            .enum_value(json!("json"))
            .enum_value(json!("yaml"))
            .enum_value(json!("toml"))
            .build();

        assert_eq!(param.enum_values.len(), 3);
        assert!(param.enum_values.contains(&json!("json")));
    }

    #[test]
    fn parameter_serialization() {
        let param = Parameter::builder("path")
            .param_type(ParameterType::String)
            .required(true)
            .description("File path")
            .build();

        let json = serde_json::to_string(&param).unwrap();
        let parsed: Parameter = serde_json::from_str(&json).unwrap();

        assert_eq!(param, parsed);
    }

    #[test]
    fn parameter_deserialization_with_defaults() {
        let json = r#"{"name": "test"}"#;
        let param: Parameter = serde_json::from_str(json).unwrap();

        assert_eq!(param.name, "test");
        assert_eq!(param.param_type, ParameterType::String);
        assert!(!param.required);
    }

    #[test]
    fn parameter_type_matches_edge_cases() {
        // Integer should match i64 and u64
        let param_type = ParameterType::Integer;
        assert!(param_type.matches(&json!(42)));
        assert!(param_type.matches(&json!(-42)));
        assert!(param_type.matches(&json!(9223372036854775807i64)));
        assert!(!param_type.matches(&json!(3.14)));
        assert!(!param_type.matches(&json!("42")));

        // Number should match any numeric value
        let param_type = ParameterType::Number;
        assert!(param_type.matches(&json!(42)));
        assert!(param_type.matches(&json!(3.14)));
        assert!(param_type.matches(&json!(-42)));
        assert!(!param_type.matches(&json!("42")));

        // Boolean should only match true/false
        let param_type = ParameterType::Boolean;
        assert!(param_type.matches(&json!(true)));
        assert!(param_type.matches(&json!(false)));
        assert!(!param_type.matches(&json!(1)));
        assert!(!param_type.matches(&json!("true")));

        // Array should only match arrays
        let param_type = ParameterType::Array;
        assert!(param_type.matches(&json!([])));
        assert!(param_type.matches(&json!([1, 2, 3])));
        assert!(!param_type.matches(&json!({})));
        assert!(!param_type.matches(&json!("[]")));

        // Object should only match objects
        let param_type = ParameterType::Object;
        assert!(param_type.matches(&json!({})));
        assert!(param_type.matches(&json!({"key": "value"})));
        assert!(!param_type.matches(&json!([])));
        assert!(!param_type.matches(&json!("{}")));
    }

    #[test]
    fn parameter_enum_validation() {
        let param = Parameter::builder("status")
            .param_type(ParameterType::String)
            .enum_value(json!("active"))
            .enum_value(json!("inactive"))
            .enum_value(json!("pending"))
            .build();

        assert_eq!(param.enum_values.len(), 3);
        assert!(param.enum_values.contains(&json!("active")));
        assert!(param.enum_values.contains(&json!("inactive")));
        assert!(param.enum_values.contains(&json!("pending")));
    }

    #[test]
    fn parameter_builder_complex() {
        let param = Parameter::builder("data")
            .param_type(ParameterType::Object)
            .required(true)
            .description("Complex data structure")
            .default(json!({"nested": {"key": "value"}}))
            .enum_value(json!({"type": "default"}))
            .enum_value(json!({"type": "custom"}))
            .build();

        assert_eq!(param.name, "data");
        assert_eq!(param.param_type, ParameterType::Object);
        assert!(param.required);
        assert_eq!(param.description, "Complex data structure");
        assert_eq!(param.default, Some(json!({"nested": {"key": "value"}})));
        assert_eq!(param.enum_values.len(), 2);
    }

    #[test]
    fn parameter_serialization_roundtrip() {
        let original = Parameter::builder("test_param")
            .param_type(ParameterType::Integer)
            .required(true)
            .description("Test description")
            .default(json!(42))
            .build();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Parameter = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.param_type, deserialized.param_type);
        assert_eq!(original.required, deserialized.required);
        assert_eq!(original.description, deserialized.description);
        assert_eq!(original.default, deserialized.default);
    }
}
