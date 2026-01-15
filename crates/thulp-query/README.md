# thulp-query

Query engine for searching and filtering tools in the Thulp framework.

## Overview

This crate provides a DSL for querying tool definitions by various criteria such as name, description, parameters, and more. It's particularly useful for building intelligent tool discovery systems that can match user intent to available tools.

## Features

- Query tools by name with wildcard support
- Filter by description keywords
- Match tools with specific parameters
- Filter by parameter count (min/max)
- Combine criteria with AND/OR logic
- Natural language query parsing
- Efficient execution against tool collections

## Usage

```rust
use thulp_query::{QueryBuilder, QueryCriteria};
use thulp_core::ToolDefinition;

// Create a query using the builder pattern
let query = QueryBuilder::new()
    .name("file_*")
    .description("read")
    .min_parameters(1)
    .build();

// Execute against a collection of tools
let tools: Vec<ToolDefinition> = vec![/* ... */];
let results = query.execute(&tools);
```

### Natural Language Queries

The crate also supports parsing natural language queries:

```rust
use thulp_query::parse_query;

// Parse a natural language query
let criteria = parse_query("name:search and has:query").unwrap();
let query = Query::new(criteria);
```

## Query Criteria

- `Name(String)` - Match tools by name (supports wildcards with `*`)
- `Description(String)` - Match tools by description keyword
- `HasParameter(String)` - Match tools with specific parameter
- `MinParameters(usize)` - Match tools with at least N parameters
- `MaxParameters(usize)` - Match tools with at most N parameters
- `And(Vec<QueryCriteria>)` - Combine criteria with AND
- `Or(Vec<QueryCriteria>)` - Combine criteria with OR
- `Not(Box<QueryCriteria>)` - Negate a criteria

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.