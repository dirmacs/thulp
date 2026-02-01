//! Session manager for persisting and managing sessions.
//!
//! This module provides the `SessionManager` for creating, loading,
//! saving, and querying sessions with file-based persistence.

use crate::filter::SessionFilter;
use crate::session::{
    EntryType, Session, SessionEntry, SessionId, SessionMetadata, SessionStatus, SessionType,
    Timestamp,
};
use crate::{Result, Workspace, WorkspaceError};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Manager for session persistence and lifecycle.
///
/// The `SessionManager` provides file-based persistence for sessions,
/// storing them in `{workspace}/.thulp/sessions/` as JSON files.
///
/// # Example
///
/// ```ignore
/// use thulp_workspace::{Workspace, SessionManager, SessionType};
///
/// let workspace = Workspace::new("my-workspace", "My Workspace", PathBuf::from("."));
/// let manager = SessionManager::new(&workspace).await?;
///
/// let session = manager.create_session("My Session", SessionType::Conversation {
///     purpose: "Testing".to_string(),
/// }).await?;
///
/// manager.add_entry(&session.id(), EntryType::UserMessage, json!({"text": "Hello"})).await?;
/// ```
pub struct SessionManager {
    /// Directory where sessions are stored.
    sessions_dir: PathBuf,
    /// In-memory cache of active sessions.
    active_sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl SessionManager {
    /// Create a new session manager for the given workspace.
    ///
    /// This will create the sessions directory if it doesn't exist.
    pub async fn new(workspace: &Workspace) -> Result<Self> {
        let sessions_dir = workspace.root.join(".thulp").join("sessions");
        fs::create_dir_all(&sessions_dir).await?;

        debug!(?sessions_dir, "Initialized session manager");

        Ok(Self {
            sessions_dir,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new session manager with a custom sessions directory.
    pub async fn with_sessions_dir(sessions_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&sessions_dir).await?;

        Ok(Self {
            sessions_dir,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get the path to a session file.
    fn session_path(&self, id: &SessionId) -> PathBuf {
        self.sessions_dir.join(format!("{}.json", id))
    }

    /// Create a new session.
    ///
    /// The session is automatically persisted to disk and cached in memory.
    pub async fn create_session(
        &self,
        name: impl Into<String>,
        session_type: SessionType,
    ) -> Result<Session> {
        let session = Session::new(name, session_type);
        let id = session.id().clone();

        // Persist to disk
        self.save_session_internal(&session).await?;

        // Cache in memory
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(id.clone(), session.clone());
        }

        info!(session_id = %id, "Created new session");
        Ok(session)
    }

    /// Load a session from disk.
    ///
    /// If the session is already cached in memory, returns the cached version.
    pub async fn load_session(&self, id: &SessionId) -> Result<Session> {
        // Check cache first
        {
            let sessions = self.active_sessions.read().await;
            if let Some(session) = sessions.get(id) {
                return Ok(session.clone());
            }
        }

        // Load from disk
        let path = self.session_path(id);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::NotFound(format!("Session {} not found", id))
            } else {
                WorkspaceError::Io(e)
            }
        })?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| WorkspaceError::Serialization(e.to_string()))?;

        // Cache for future access
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(id.clone(), session.clone());
        }

        debug!(session_id = %id, "Loaded session from disk");
        Ok(session)
    }

    /// Save a session to disk.
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        self.save_session_internal(session).await?;

        // Update cache
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session.id().clone(), session.clone());
        }

        Ok(())
    }

    /// Internal save without updating cache (to avoid deadlocks).
    async fn save_session_internal(&self, session: &Session) -> Result<()> {
        let path = self.session_path(session.id());
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| WorkspaceError::Serialization(e.to_string()))?;

        fs::write(&path, content).await?;
        debug!(session_id = %session.id(), "Saved session to disk");
        Ok(())
    }

    /// Add an entry to a session.
    ///
    /// The session is automatically saved after adding the entry.
    pub async fn add_entry(
        &self,
        session_id: &SessionId,
        entry_type: EntryType,
        content: Value,
    ) -> Result<SessionEntry> {
        let entry = SessionEntry::new(entry_type, content);

        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.add_entry(entry.clone());
                // Save to disk
                self.save_session_internal(session).await?;
            } else {
                // Load from disk, add entry, save
                drop(sessions); // Release lock before loading
                let mut session = self.load_session(session_id).await?;
                session.add_entry(entry.clone());
                self.save_session(&session).await?;
            }
        }

        debug!(session_id = %session_id, entry_id = %entry.id, "Added entry to session");
        Ok(entry)
    }

    /// Complete a session.
    ///
    /// Marks the session as completed and saves it.
    pub async fn complete_session(&self, session_id: &SessionId) -> Result<()> {
        self.update_status(session_id, SessionStatus::Completed)
            .await
    }

    /// Fail a session.
    ///
    /// Marks the session as failed and saves it.
    pub async fn fail_session(&self, session_id: &SessionId) -> Result<()> {
        self.update_status(session_id, SessionStatus::Failed).await
    }

    /// Cancel a session.
    ///
    /// Marks the session as cancelled and saves it.
    pub async fn cancel_session(&self, session_id: &SessionId) -> Result<()> {
        self.update_status(session_id, SessionStatus::Cancelled)
            .await
    }

    /// Pause a session.
    ///
    /// Marks the session as paused and saves it.
    pub async fn pause_session(&self, session_id: &SessionId) -> Result<()> {
        self.update_status(session_id, SessionStatus::Paused).await
    }

    /// Resume a paused session.
    ///
    /// Marks the session as active and saves it.
    pub async fn resume_session(&self, session_id: &SessionId) -> Result<()> {
        self.update_status(session_id, SessionStatus::Active).await
    }

    /// Update session status.
    async fn update_status(&self, session_id: &SessionId, status: SessionStatus) -> Result<()> {
        let mut session = self.load_session(session_id).await?;
        session.set_status(status);
        self.save_session(&session).await?;

        info!(session_id = %session_id, ?status, "Updated session status");
        Ok(())
    }

    /// List all sessions, optionally filtered.
    pub async fn list_sessions(
        &self,
        filter: Option<&SessionFilter>,
    ) -> Result<Vec<SessionMetadata>> {
        let mut metadata_list = Vec::new();

        let mut entries = fs::read_dir(&self.sessions_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            match fs::read_to_string(&path).await {
                Ok(content) => {
                    match serde_json::from_str::<Session>(&content) {
                        Ok(session) => {
                            let metadata = session.metadata.clone();

                            // Apply filter if provided
                            if let Some(filter) = filter {
                                if filter.matches(&session) {
                                    metadata_list.push(metadata);
                                }
                            } else {
                                metadata_list.push(metadata);
                            }
                        }
                        Err(e) => {
                            warn!(path = ?path, error = %e, "Failed to parse session file");
                        }
                    }
                }
                Err(e) => {
                    warn!(path = ?path, error = %e, "Failed to read session file");
                }
            }
        }

        // Sort by updated_at descending (most recent first)
        metadata_list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(metadata_list)
    }

    /// Delete a session.
    ///
    /// Removes the session from disk and cache.
    pub async fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        // Remove from cache
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(session_id);
        }

        // Remove from disk
        let path = self.session_path(session_id);
        if path.exists() {
            fs::remove_file(&path).await?;
            info!(session_id = %session_id, "Deleted session");
        }

        Ok(())
    }

    /// Check if a session exists.
    pub async fn session_exists(&self, session_id: &SessionId) -> bool {
        // Check cache
        {
            let sessions = self.active_sessions.read().await;
            if sessions.contains_key(session_id) {
                return true;
            }
        }

        // Check disk
        self.session_path(session_id).exists()
    }

    /// Get a session without loading it into cache.
    ///
    /// Useful for one-off reads where caching isn't beneficial.
    pub async fn peek_session(&self, session_id: &SessionId) -> Result<Session> {
        let path = self.session_path(session_id);
        let content = fs::read_to_string(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::NotFound(format!("Session {} not found", session_id))
            } else {
                WorkspaceError::Io(e)
            }
        })?;

        serde_json::from_str(&content).map_err(|e| WorkspaceError::Serialization(e.to_string()))
    }

    /// Evict a session from the in-memory cache.
    ///
    /// The session remains on disk but is removed from memory.
    pub async fn evict_from_cache(&self, session_id: &SessionId) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
    }

    /// Clear all sessions from the in-memory cache.
    pub async fn clear_cache(&self) {
        let mut sessions = self.active_sessions.write().await;
        sessions.clear();
    }

    /// Get the number of cached sessions.
    pub async fn cached_session_count(&self) -> usize {
        let sessions = self.active_sessions.read().await;
        sessions.len()
    }

    /// Get all active sessions (in-memory only).
    pub async fn active_sessions(&self) -> Vec<Session> {
        let sessions = self.active_sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Find sessions by tag.
    pub async fn find_by_tag(&self, tag: &str) -> Result<Vec<SessionMetadata>> {
        self.list_sessions(Some(&SessionFilter::HasTag(tag.to_string())))
            .await
    }

    /// Find sessions by type.
    pub async fn find_by_type(&self, session_type_name: &str) -> Result<Vec<SessionMetadata>> {
        self.list_sessions(Some(&SessionFilter::ByTypeName(
            session_type_name.to_string(),
        )))
        .await
    }

    /// Find sessions by status.
    pub async fn find_by_status(&self, status: SessionStatus) -> Result<Vec<SessionMetadata>> {
        self.list_sessions(Some(&SessionFilter::ByStatus(status)))
            .await
    }

    /// Find sessions created after a timestamp.
    pub async fn find_created_after(&self, timestamp: Timestamp) -> Result<Vec<SessionMetadata>> {
        self.list_sessions(Some(&SessionFilter::CreatedAfter(timestamp)))
            .await
    }

    /// Find sessions updated after a timestamp.
    pub async fn find_updated_after(&self, timestamp: Timestamp) -> Result<Vec<SessionMetadata>> {
        self.list_sessions(Some(&SessionFilter::UpdatedAfter(timestamp)))
            .await
    }

    /// Get session count on disk.
    pub async fn session_count(&self) -> Result<usize> {
        let mut count = 0;
        let mut entries = fs::read_dir(&self.sessions_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }
        Ok(count)
    }
}

impl std::fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("sessions_dir", &self.sessions_dir)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_manager() -> (SessionManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let sessions_dir = temp_dir.path().join("sessions");
        let manager = SessionManager::with_sessions_dir(sessions_dir)
            .await
            .unwrap();
        (manager, temp_dir)
    }

    #[tokio::test]
    async fn test_create_session() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(session.name(), "Test Session");
        assert_eq!(session.status(), SessionStatus::Active);
    }

    #[tokio::test]
    async fn test_load_session() {
        let (manager, _temp) = create_test_manager().await;

        let created = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        // Clear cache to force disk load
        manager.clear_cache().await;

        let loaded = manager.load_session(created.id()).await.unwrap();
        assert_eq!(loaded.name(), created.name());
        assert_eq!(loaded.id(), created.id());
    }

    #[tokio::test]
    async fn test_add_entry() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        let entry = manager
            .add_entry(
                session.id(),
                EntryType::UserMessage,
                serde_json::json!({"text": "Hello"}),
            )
            .await
            .unwrap();

        assert!(matches!(entry.entry_type, EntryType::UserMessage));

        // Verify entry was persisted
        manager.clear_cache().await;
        let loaded = manager.load_session(session.id()).await.unwrap();
        assert_eq!(loaded.entries.len(), 1);
    }

    #[tokio::test]
    async fn test_complete_session() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        manager.complete_session(session.id()).await.unwrap();

        let loaded = manager.load_session(session.id()).await.unwrap();
        assert_eq!(loaded.status(), SessionStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let (manager, _temp) = create_test_manager().await;

        // Create multiple sessions
        manager
            .create_session(
                "Session 1",
                SessionType::Conversation {
                    purpose: "Test 1".to_string(),
                },
            )
            .await
            .unwrap();

        manager
            .create_session(
                "Session 2",
                SessionType::Conversation {
                    purpose: "Test 2".to_string(),
                },
            )
            .await
            .unwrap();

        let sessions = manager.list_sessions(None).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        manager.delete_session(session.id()).await.unwrap();

        assert!(!manager.session_exists(session.id()).await);
    }

    #[tokio::test]
    async fn test_session_exists() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        assert!(manager.session_exists(session.id()).await);

        let fake_id = SessionId::new();
        assert!(!manager.session_exists(&fake_id).await);
    }

    #[tokio::test]
    async fn test_filter_by_status() {
        let (manager, _temp) = create_test_manager().await;

        let session1 = manager
            .create_session(
                "Active Session",
                SessionType::Conversation {
                    purpose: "Test".to_string(),
                },
            )
            .await
            .unwrap();

        let session2 = manager
            .create_session(
                "Completed Session",
                SessionType::Conversation {
                    purpose: "Test".to_string(),
                },
            )
            .await
            .unwrap();

        manager.complete_session(session2.id()).await.unwrap();

        let active = manager.find_by_status(SessionStatus::Active).await.unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, session1.metadata.id);

        let completed = manager
            .find_by_status(SessionStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, session2.metadata.id);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let (manager, _temp) = create_test_manager().await;

        let session = manager
            .create_session(
                "Test Session",
                SessionType::Conversation {
                    purpose: "Testing".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(manager.cached_session_count().await, 1);

        manager.evict_from_cache(session.id()).await;
        assert_eq!(manager.cached_session_count().await, 0);

        // Session should still exist on disk
        assert!(manager.session_exists(session.id()).await);
    }

    #[tokio::test]
    async fn test_session_count() {
        let (manager, _temp) = create_test_manager().await;

        assert_eq!(manager.session_count().await.unwrap(), 0);

        manager
            .create_session(
                "Session 1",
                SessionType::Conversation {
                    purpose: "Test".to_string(),
                },
            )
            .await
            .unwrap();

        manager
            .create_session(
                "Session 2",
                SessionType::Conversation {
                    purpose: "Test".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(manager.session_count().await.unwrap(), 2);
    }
}
