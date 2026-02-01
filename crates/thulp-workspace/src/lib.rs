//! # thulp-workspace
//!
//! Workspace and session management for thulp.
//!
//! This crate provides functionality for managing agent workspaces,
//! including context, state, and session persistence.
//!
//! ## Features
//!
//! - **Workspace Management**: Create, load, and manage workspaces with metadata and context
//! - **Session Management**: Track conversation history, tool calls, and skill executions
//! - **Turn Counting**: Monitor conversation turns with configurable limits
//! - **Persistence**: File-based storage for sessions with in-memory caching
//! - **Filtering**: Query sessions by status, type, tags, and timestamps
//!
//! ## Example
//!
//! ```ignore
//! use thulp_workspace::{Workspace, SessionManager, SessionType, SessionFilter};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a workspace
//!     let workspace = Workspace::new("my-workspace", "My Workspace", PathBuf::from("."));
//!
//!     // Create a session manager
//!     let manager = SessionManager::new(&workspace).await?;
//!
//!     // Create a session
//!     let session = manager.create_session("Chat Session", SessionType::Conversation {
//!         purpose: "User assistance".to_string(),
//!     }).await?;
//!
//!     // Add entries
//!     manager.add_entry(
//!         session.id(),
//!         EntryType::UserMessage,
//!         serde_json::json!({"text": "Hello!"}),
//!     ).await?;
//!
//!     // Query sessions
//!     let active_sessions = manager.find_by_status(SessionStatus::Active).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod filter;
pub mod session;
pub mod session_manager;

pub use filter::SessionFilter;
pub use session::{
    EntryType, LimitAction, LimitCheck, LimitExceeded, Session, SessionConfig, SessionEntry,
    SessionId, SessionMetadata, SessionStatus, SessionType, Timestamp,
};
pub use session_manager::SessionManager;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use std::fs;
use std::path::Path;

/// Result type for workspace operations
pub type Result<T> = std::result::Result<T, WorkspaceError>;

/// Errors that can occur in workspace operations
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Workspace not found: {0}")]
    NotFound(String),
}

/// A workspace for an agent session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace ID
    pub id: String,

    /// Workspace name
    pub name: String,

    /// Root directory path
    pub root: PathBuf,

    /// Workspace metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Context data
    #[serde(default)]
    pub context: HashMap<String, Value>,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(id: impl Into<String>, name: impl Into<String>, root: PathBuf) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            root,
            metadata: HashMap::new(),
            context: HashMap::new(),
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set context data
    pub fn with_context(mut self, key: impl Into<String>, value: Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&Value> {
        self.context.get(key)
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl Workspace {
    /// Save the workspace to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| WorkspaceError::Serialization(e.to_string()))?;
        fs::write(path, json).map_err(WorkspaceError::Io)?;
        Ok(())
    }

    /// Load a workspace from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(path).map_err(WorkspaceError::Io)?;
        let workspace = serde_json::from_str(&json)
            .map_err(|e| WorkspaceError::Serialization(e.to_string()))?;
        Ok(workspace)
    }
}

/// Manager for multiple workspaces
#[derive(Debug, Default)]
pub struct WorkspaceManager {
    workspaces: HashMap<String, Workspace>,
    active_workspace: Option<String>,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create and register a new workspace
    pub fn create(&mut self, workspace: Workspace) {
        self.workspaces.insert(workspace.id.clone(), workspace);
    }

    /// Get a workspace by ID
    pub fn get(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.get(id)
    }

    /// Get a mutable workspace by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Workspace> {
        self.workspaces.get_mut(id)
    }

    /// Set the active workspace
    pub fn set_active(&mut self, id: &str) -> Result<()> {
        if !self.workspaces.contains_key(id) {
            return Err(WorkspaceError::NotFound(id.to_string()));
        }
        self.active_workspace = Some(id.to_string());
        Ok(())
    }

    /// Get the active workspace
    pub fn get_active(&self) -> Option<&Workspace> {
        self.active_workspace
            .as_ref()
            .and_then(|id| self.workspaces.get(id))
    }

    /// Get the active workspace mutably
    pub fn get_active_mut(&mut self) -> Option<&mut Workspace> {
        self.active_workspace
            .as_ref()
            .and_then(|id| self.workspaces.get_mut(id))
    }

    /// List all workspace IDs
    pub fn list(&self) -> Vec<String> {
        self.workspaces.keys().cloned().collect()
    }

    /// Remove a workspace
    pub fn remove(&mut self, id: &str) -> Option<Workspace> {
        if self.active_workspace.as_deref() == Some(id) {
            self.active_workspace = None;
        }
        self.workspaces.remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace::new("test", "Test Workspace", PathBuf::from("/tmp/test"));
        assert_eq!(workspace.id, "test");
        assert_eq!(workspace.name, "Test Workspace");
    }

    #[test]
    fn test_workspace_builder() {
        let workspace = Workspace::new("test", "Test", PathBuf::from("/tmp"))
            .with_metadata("version", "1.0")
            .with_context("key", serde_json::json!({"value": 42}));

        assert_eq!(workspace.get_metadata("version"), Some(&"1.0".to_string()));
        assert!(workspace.get_context("key").is_some());
    }

    #[test]
    fn test_workspace_manager() {
        let mut manager = WorkspaceManager::new();

        let workspace = Workspace::new("test", "Test", PathBuf::from("/tmp"));
        manager.create(workspace);

        assert!(manager.get("test").is_some());
        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_active_workspace() {
        let mut manager = WorkspaceManager::new();

        let workspace = Workspace::new("test", "Test", PathBuf::from("/tmp"));
        manager.create(workspace);

        manager.set_active("test").unwrap();
        assert!(manager.get_active().is_some());
        assert_eq!(manager.get_active().unwrap().id, "test");
    }

    #[test]
    fn test_remove_workspace() {
        let mut manager = WorkspaceManager::new();

        let workspace = Workspace::new("test", "Test", PathBuf::from("/tmp"));
        manager.create(workspace);

        manager.set_active("test").unwrap();
        assert!(manager.remove("test").is_some());
        assert!(manager.get_active().is_none());
    }

    #[test]
    fn test_workspace_save_load() {
        let workspace = Workspace::new("test", "Test Workspace", PathBuf::from("/tmp/test"))
            .with_metadata("version", "1.0")
            .with_context("key", serde_json::json!({"value": 42}));

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save workspace
        workspace.save_to_file(path).unwrap();

        // Load workspace
        let loaded = Workspace::load_from_file(path).unwrap();

        assert_eq!(workspace.id, loaded.id);
        assert_eq!(workspace.name, loaded.name);
        assert_eq!(workspace.metadata, loaded.metadata);
        assert_eq!(workspace.context, loaded.context);
    }
}
