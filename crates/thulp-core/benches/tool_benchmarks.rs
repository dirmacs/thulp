use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::json;
use thulp_core::{Parameter, ParameterType, ToolCall, ToolDefinition};

fn benchmark_tool_creation(c: &mut Criterion) {
    c.bench_function("tool_definition_new", |b| {
        b.iter(|| ToolDefinition::new(black_box("test_tool")))
    });

    c.bench_function("tool_definition_builder", |b| {
        b.iter(|| {
            ToolDefinition::builder(black_box("test_tool"))
                .description("A test tool")
                .parameter(Parameter::required_string("param1"))
                .parameter(Parameter::optional_string("param2"))
                .build()
        })
    });
}

fn benchmark_tool_validation(c: &mut Criterion) {
    let tool = ToolDefinition::builder("test_tool")
        .parameter(Parameter::required_string("name"))
        .parameter(
            Parameter::builder("age")
                .param_type(ParameterType::Integer)
                .required(true)
                .build(),
        )
        .parameter(
            Parameter::builder("active")
                .param_type(ParameterType::Boolean)
                .required(false)
                .build(),
        )
        .build();

    let valid_args = json!({
        "name": "test",
        "age": 42,
        "active": true
    });

    let invalid_args = json!({
        "name": "test"
        // Missing required 'age'
    });

    c.bench_function("validate_args_valid", |b| {
        b.iter(|| tool.validate_args(black_box(&valid_args)))
    });

    c.bench_function("validate_args_invalid", |b| {
        b.iter(|| {
            let _ = tool.validate_args(black_box(&invalid_args));
        })
    });
}

fn benchmark_parameter_matching(c: &mut Criterion) {
    let param_types = vec![
        ("string", ParameterType::String, json!("test")),
        ("integer", ParameterType::Integer, json!(42)),
        ("number", ParameterType::Number, json!(3.14)),
        ("boolean", ParameterType::Boolean, json!(true)),
        ("array", ParameterType::Array, json!([1, 2, 3])),
        ("object", ParameterType::Object, json!({"key": "value"})),
    ];

    for (name, param_type, value) in param_types {
        c.bench_with_input(
            BenchmarkId::new("parameter_type_matches", name),
            &(param_type, value),
            |b, (pt, val)| b.iter(|| pt.matches(black_box(val))),
        );
    }
}

fn benchmark_tool_call_builder(c: &mut Criterion) {
    c.bench_function("tool_call_builder_simple", |b| {
        b.iter(|| {
            ToolCall::builder(black_box("test_tool"))
                .arg_str("name", "value")
                .arg_int("count", 42)
                .build()
        })
    });

    c.bench_function("tool_call_builder_complex", |b| {
        b.iter(|| {
            ToolCall::builder(black_box("test_tool"))
                .arg_str("name", "value")
                .arg_int("count", 42)
                .arg_bool("active", true)
                .arg("data", json!({"nested": {"key": "value"}}))
                .arg("items", json!([1, 2, 3, 4, 5]))
                .build()
        })
    });
}

fn benchmark_parse_mcp_schema(c: &mut Criterion) {
    let simple_schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string", "description": "The name"},
            "age": {"type": "integer", "description": "The age"}
        },
        "required": ["name"]
    });

    let complex_schema = json!({
        "type": "object",
        "properties": {
            "string_field": {"type": "string"},
            "int_field": {"type": "integer"},
            "bool_field": {"type": "boolean"},
            "array_field": {"type": "array"},
            "object_field": {"type": "object"},
            "number_field": {"type": "number"},
            "field1": {"type": "string", "description": "Field 1"},
            "field2": {"type": "string", "description": "Field 2"},
            "field3": {"type": "string", "description": "Field 3"},
            "field4": {"type": "string", "description": "Field 4"},
        },
        "required": ["string_field", "int_field"]
    });

    c.bench_function("parse_mcp_schema_simple", |b| {
        b.iter(|| ToolDefinition::parse_mcp_input_schema(black_box(&simple_schema)))
    });

    c.bench_function("parse_mcp_schema_complex", |b| {
        b.iter(|| ToolDefinition::parse_mcp_input_schema(black_box(&complex_schema)))
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let tool = ToolDefinition::builder("test_tool")
        .description("A test tool")
        .parameter(Parameter::required_string("name"))
        .parameter(
            Parameter::builder("age")
                .param_type(ParameterType::Integer)
                .required(true)
                .build(),
        )
        .build();

    c.bench_function("serialize_tool_definition", |b| {
        b.iter(|| serde_json::to_string(black_box(&tool)))
    });

    let tool_json = serde_json::to_string(&tool).unwrap();

    c.bench_function("deserialize_tool_definition", |b| {
        b.iter(|| serde_json::from_str::<ToolDefinition>(black_box(&tool_json)))
    });
}

criterion_group!(
    benches,
    benchmark_tool_creation,
    benchmark_tool_validation,
    benchmark_parameter_matching,
    benchmark_tool_call_builder,
    benchmark_parse_mcp_schema,
    benchmark_serialization
);
criterion_main!(benches);
