# thulp-skills

Skill composition and execution for thulp.

## Overview

This crate provides a system for defining and executing complex tool workflows called "skills." Skills are predefined sequences of tool calls that accomplish higher-level tasks by orchestrating multiple tools in a coordinated manner.

## Features

- Define multi-step tool workflows
- Parameterized skill definitions
- Context passing between steps
- Error handling with continue-on-error options
- Skill registry for organization
- Execution with any Thulp transport
- JSON serialization/deserialization

## Usage

```rust
use thulp_skills::{Skill, SkillStep, SkillRegistry};
use serde_json::json;

// Create a skill
let skill = Skill::new("search_and_summarize", "Search and summarize results")
    .with_input("query")
    .with_step(SkillStep {
        name: "search".to_string(),
        tool: "web_search".to_string(),
        arguments: json!({"query": "{{query}}"}),
        continue_on_error: false,
    })
    .with_step(SkillStep {
        name: "summarize".to_string(),
        tool: "summarize".to_string(),
        arguments: json!({"text": "{{search.results}}"}),
        continue_on_error: false,
    });

// Register skills
let mut registry = SkillRegistry::new();
registry.register(skill);

// Execute skills with a transport
// let result = skill.execute(&transport, &input_args).await?;
```

## Skill Structure

A skill consists of:

- **Name**: Unique identifier
- **Description**: Human-readable description
- **Inputs**: Required input parameters
- **Steps**: Ordered sequence of tool executions

Each step contains:

- **Name**: Step identifier
- **Tool**: Tool to execute
- **Arguments**: Parameters for the tool (supports templating)
- **ContinueOnError**: Whether to continue if this step fails

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.