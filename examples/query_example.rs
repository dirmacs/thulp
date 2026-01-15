//! Query DSL example
//!
//! Run: `cargo run --example query`

use thulp_core::{Parameter, ParameterType, ToolDefinition};
use thulp_query::{parse_query, Query, QueryBuilder, QueryCriteria};

fn main() {
    println!("=== Thulp Query DSL Example ===\n");

    // Create sample tools
    let tools = vec![
        ToolDefinition::builder("get_weather")
            .description("Get current weather for a location")
            .parameter(Parameter::required_string("location"))
            .build(),
        ToolDefinition::builder("search_files")
            .description("Search files in a directory")
            .parameter(Parameter::required_string("pattern"))
            .parameter(
                Parameter::builder("recursive")
                    .param_type(ParameterType::Boolean)
                    .description("Recursive search")
                    .build(),
            )
            .build(),
        ToolDefinition::builder("send_email")
            .description("Send an email")
            .parameter(Parameter::required_string("to"))
            .parameter(Parameter::required_string("subject"))
            .build(),
        ToolDefinition::builder("read_file")
            .description("Read file contents")
            .parameter(Parameter::required_string("path"))
            .build(),
    ];

    // Example 1: Using parse_query with natural language-like DSL
    println!("--- Parsing DSL queries ---\n");

    let queries = vec![
        "name:weather",          // Name contains "weather"
        "name:*file*",           // Name contains "file" (wildcard)
        "has:path",              // Has parameter named "path"
        "min:2",                 // At least 2 parameters
        "desc:file",             // Description contains "file"
        "name:search and min:1", // Combined with AND
    ];

    for query_str in queries {
        println!("Query: '{}'", query_str);
        match parse_query(query_str) {
            Ok(criteria) => {
                let matches: Vec<_> = tools
                    .iter()
                    .filter(|t| criteria.matches(t))
                    .map(|t| &t.name)
                    .collect();
                println!("  Matches: {:?}\n", matches);
            }
            Err(e) => {
                println!("  Parse error: {:?}\n", e);
            }
        }
    }

    // Example 2: Using QueryBuilder for programmatic queries
    println!("--- Using QueryBuilder ---\n");

    let query = QueryBuilder::new().name("file").min_parameters(1).build();

    let results = query.execute(&tools);
    println!("Query: name contains 'file' AND min 1 parameter");
    println!(
        "Results: {:?}\n",
        results.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Example 3: Using QueryCriteria directly
    println!("--- Using QueryCriteria ---\n");

    let criteria = QueryCriteria::Or(vec![
        QueryCriteria::Name("weather".to_string()),
        QueryCriteria::Name("email".to_string()),
    ]);

    let query = Query::new(criteria);
    let results = query.execute(&tools);
    println!("Query: name contains 'weather' OR 'email'");
    println!(
        "Results: {:?}\n",
        results.iter().map(|t| &t.name).collect::<Vec<_>>()
    );

    // Example 4: Complex nested query
    println!("--- Complex Nested Query ---\n");

    let criteria = QueryCriteria::And(vec![
        QueryCriteria::Or(vec![
            QueryCriteria::Name("file".to_string()),
            QueryCriteria::HasParameter("path".to_string()),
        ]),
        QueryCriteria::Not(Box::new(QueryCriteria::MinParameters(2))),
    ]);

    let query = Query::new(criteria);
    let results = query.execute(&tools);
    println!("Query: (name contains 'file' OR has 'path' param) AND NOT min 2 params");
    println!(
        "Results: {:?}",
        results.iter().map(|t| &t.name).collect::<Vec<_>>()
    );
}
