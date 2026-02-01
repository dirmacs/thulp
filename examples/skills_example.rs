//! Skills workflow example
//!
//! Run: `cargo run --example skills`

use serde_json::json;
use thulp_skills::{Skill, SkillRegistry, SkillStep};

fn main() {
    println!("=== Thulp Skills Example ===\n");

    // Create a skill that chains multiple tool calls
    let skill = Skill::new(
        "search_and_summarize",
        "Search for information and create a summary",
    )
    .with_input("query")
    .with_input("max_results")
    .with_step(SkillStep {
        name: "search".to_string(),
        tool: "web_search".to_string(),
        arguments: json!({
            "query": "{{query}}",
            "limit": "{{max_results}}"
        }),
        continue_on_error: false,
        timeout_secs: None,
        max_retries: None,
    })
    .with_step(SkillStep {
        name: "summarize".to_string(),
        tool: "text_summarizer".to_string(),
        arguments: json!({
            "text": "{{search.results}}",
            "format": "bullet_points"
        }),
        continue_on_error: false,
        timeout_secs: None,
        max_retries: None,
    })
    .with_step(SkillStep {
        name: "notify".to_string(),
        tool: "send_notification".to_string(),
        arguments: json!({
            "message": "Search complete: {{summarize.summary}}",
            "channel": "results"
        }),
        continue_on_error: true, // Continue even if notification fails
        timeout_secs: None,
        max_retries: None,
    });

    println!("Skill: {} - {}", skill.name, skill.description);
    println!("\nInputs: {:?}", skill.inputs);
    println!("\nSteps:");
    for (i, step) in skill.steps.iter().enumerate() {
        println!("  {}. {} -> {}", i + 1, step.name, step.tool);
        println!("     Args: {}", step.arguments);
        if step.continue_on_error {
            println!("     (continues on error)");
        }
    }

    // Create a registry and register skills
    println!("\n--- Skill Registry ---\n");

    let mut registry = SkillRegistry::new();

    // Create and register multiple skills
    let fetch_skill = Skill::new("fetch_and_parse", "Fetch a URL and parse its content")
        .with_input("url")
        .with_step(SkillStep {
            name: "fetch".to_string(),
            tool: "http_get".to_string(),
            arguments: json!({"url": "{{url}}"}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        })
        .with_step(SkillStep {
            name: "parse".to_string(),
            tool: "html_parser".to_string(),
            arguments: json!({"html": "{{fetch.body}}"}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

    let analyze_skill = Skill::new("analyze_code", "Analyze source code quality")
        .with_input("file_path")
        .with_step(SkillStep {
            name: "read".to_string(),
            tool: "read_file".to_string(),
            arguments: json!({"path": "{{file_path}}"}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        })
        .with_step(SkillStep {
            name: "analyze".to_string(),
            tool: "code_analyzer".to_string(),
            arguments: json!({"code": "{{read.content}}"}),
            continue_on_error: false,
            timeout_secs: None,
            max_retries: None,
        });

    registry.register(skill);
    registry.register(fetch_skill);
    registry.register(analyze_skill);

    println!("Registered skills: {:?}", registry.list());

    // Look up a skill
    if let Some(skill) = registry.get("fetch_and_parse") {
        println!("\nFound skill: {}", skill.name);
        println!("  Description: {}", skill.description);
        println!("  Steps: {}", skill.steps.len());
    }

    // Unregister a skill
    registry.unregister("analyze_code");
    println!("\nAfter unregister: {:?}", registry.list());
}
