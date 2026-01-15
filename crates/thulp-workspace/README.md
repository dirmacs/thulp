# thulp-workspace

Workspace and session management for thulp execution contexts.

## Overview

This crate provides functionality for managing agent workspaces, including context, state, and session persistence. Workspaces allow AI agents to maintain state between tool executions and organize their working environment.

## Features

- Create and manage multiple workspaces
- Persistent storage of workspace state
- Context data management
- Metadata storage
- Active workspace tracking
- JSON serialization/deserialization

## Usage

```rust
use thulp_workspace::{Workspace, WorkspaceManager};
use std::path::PathBuf;

// Create a new workspace
let workspace = Workspace::new("my_project", "My Project", PathBuf::from("/path/to/project"))
    .with_metadata("version", "1.0")
    .with_context("api_key", serde_json::json!("secret123"));

// Save to file
workspace.save_to_file("workspace.json").unwrap();

// Load from file
let loaded = Workspace::load_from_file("workspace.json").unwrap();

// Manage workspaces
let mut manager = WorkspaceManager::new();
manager.create(workspace);
manager.set_active("my_project").unwrap();
```

## Workspace Structure

A workspace contains:

- **ID**: Unique identifier for the workspace
- **Name**: Human-readable name
- **Root**: Root directory path
- **Metadata**: Key-value pairs for workspace metadata
- **Context**: Structured data for the execution context

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.