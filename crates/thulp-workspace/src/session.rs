//! Session types and management for thulp.
//!
//! This module provides session tracking for conversation history,
//! evaluation runs, and skill execution sessions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Unique session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    /// Create a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse a session ID from a string.
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the UUID as a string.
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Timestamp in milliseconds since Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Create a timestamp for the current time.
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Self(duration.as_millis() as u64)
    }

    /// Create a timestamp from milliseconds.
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Get the timestamp as milliseconds.
    pub fn as_millis(&self) -> u64 {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

/// Type of session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionType {
    /// Teacher demonstration session (for distillation).
    TeacherDemo {
        /// Task being demonstrated.
        task: String,
        /// Model used for demonstration.
        model: String,
    },

    /// Skill evaluation run.
    Evaluation {
        /// Name of the skill being evaluated.
        skill_name: String,
        /// Number of test cases.
        test_cases: usize,
    },

    /// Skill refinement session.
    Refinement {
        /// Name of the skill being refined.
        skill_name: String,
        /// Iteration number.
        iteration: usize,
    },

    /// Generic conversation session.
    Conversation {
        /// Purpose of the conversation.
        purpose: String,
    },

    /// Agent interaction session.
    Agent {
        /// Agent name or identifier.
        agent_name: String,
    },
}

/// Status of a session.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is currently active.
    #[default]
    Active,
    /// Session completed successfully.
    Completed,
    /// Session failed with an error.
    Failed,
    /// Session was cancelled.
    Cancelled,
    /// Session is paused.
    Paused,
}

/// Session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Unique session identifier.
    pub id: SessionId,
    /// Human-readable session name.
    pub name: String,
    /// Type of session.
    pub session_type: SessionType,
    /// When the session was created.
    pub created_at: Timestamp,
    /// When the session was last updated.
    pub updated_at: Timestamp,
    /// Current session status.
    pub status: SessionStatus,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Parent session ID (for linked sessions).
    #[serde(default)]
    pub parent_session: Option<SessionId>,
}

impl SessionMetadata {
    /// Create new session metadata.
    pub fn new(name: impl Into<String>, session_type: SessionType) -> Self {
        let now = Timestamp::now();
        Self {
            id: SessionId::new(),
            name: name.into(),
            session_type,
            created_at: now,
            updated_at: now,
            status: SessionStatus::Active,
            tags: Vec::new(),
            parent_session: None,
        }
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set parent session.
    pub fn with_parent(mut self, parent: SessionId) -> Self {
        self.parent_session = Some(parent);
        self
    }
}

/// Type of entry in a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EntryType {
    /// User message.
    UserMessage,

    /// Assistant/AI response.
    AssistantMessage,

    /// System message.
    SystemMessage,

    /// Tool invocation.
    ToolCall {
        /// Name of the tool called.
        tool_name: String,
        /// Whether the call succeeded.
        success: bool,
    },

    /// Skill execution.
    SkillExecution {
        /// Name of the skill.
        skill_name: String,
        /// Whether execution succeeded.
        success: bool,
    },

    /// Evaluation result.
    EvaluationResult {
        /// Overall score (0.0 - 1.0).
        score: f64,
        /// Detailed metrics.
        metrics: HashMap<String, f64>,
    },

    /// System event (logging, state changes, etc.).
    SystemEvent {
        /// Event type/name.
        event: String,
    },
}

/// A single entry in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Unique entry identifier.
    pub id: Uuid,
    /// When the entry was created.
    pub timestamp: Timestamp,
    /// Type of entry.
    pub entry_type: EntryType,
    /// Entry content/data.
    pub content: Value,
}

impl SessionEntry {
    /// Create a new session entry.
    pub fn new(entry_type: EntryType, content: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Timestamp::now(),
            entry_type,
            content,
        }
    }

    /// Create a user message entry.
    pub fn user_message(content: impl Into<String>) -> Self {
        Self::new(
            EntryType::UserMessage,
            serde_json::json!({ "text": content.into() }),
        )
    }

    /// Create an assistant message entry.
    pub fn assistant_message(content: impl Into<String>) -> Self {
        Self::new(
            EntryType::AssistantMessage,
            serde_json::json!({ "text": content.into() }),
        )
    }

    /// Create a tool call entry.
    pub fn tool_call(tool_name: impl Into<String>, success: bool, result: Value) -> Self {
        Self::new(
            EntryType::ToolCall {
                tool_name: tool_name.into(),
                success,
            },
            result,
        )
    }

    /// Create a skill execution entry.
    pub fn skill_execution(skill_name: impl Into<String>, success: bool, result: Value) -> Self {
        Self::new(
            EntryType::SkillExecution {
                skill_name: skill_name.into(),
                success,
            },
            result,
        )
    }
}

/// Complete session data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session metadata.
    pub metadata: SessionMetadata,
    /// Session entries (conversation history, tool calls, etc.).
    pub entries: Vec<SessionEntry>,
    /// Session context data (key-value store).
    #[serde(default)]
    pub context: HashMap<String, Value>,
}

impl Session {
    /// Create a new session.
    pub fn new(name: impl Into<String>, session_type: SessionType) -> Self {
        Self {
            metadata: SessionMetadata::new(name, session_type),
            entries: Vec::new(),
            context: HashMap::new(),
        }
    }

    /// Get the session ID.
    pub fn id(&self) -> &SessionId {
        &self.metadata.id
    }

    /// Get the session name.
    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    /// Get the session status.
    pub fn status(&self) -> SessionStatus {
        self.metadata.status
    }

    /// Add an entry to the session.
    pub fn add_entry(&mut self, entry: SessionEntry) {
        self.entries.push(entry);
        self.metadata.updated_at = Timestamp::now();
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_entry(SessionEntry::user_message(content));
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_entry(SessionEntry::assistant_message(content));
    }

    /// Set context value.
    pub fn set_context(&mut self, key: impl Into<String>, value: Value) {
        self.context.insert(key.into(), value);
        self.metadata.updated_at = Timestamp::now();
    }

    /// Get context value.
    pub fn get_context(&self, key: &str) -> Option<&Value> {
        self.context.get(key)
    }

    /// Update session status.
    pub fn set_status(&mut self, status: SessionStatus) {
        self.metadata.status = status;
        self.metadata.updated_at = Timestamp::now();
    }

    /// Mark session as completed.
    pub fn complete(&mut self) {
        self.set_status(SessionStatus::Completed);
    }

    /// Mark session as failed.
    pub fn fail(&mut self) {
        self.set_status(SessionStatus::Failed);
    }

    /// Count the number of turns in the session.
    ///
    /// A turn is defined as a user message followed by an assistant message.
    /// This counts the number of complete turns.
    pub fn turn_count(&self) -> u32 {
        let mut turns = 0;
        let mut awaiting_response = false;

        for entry in &self.entries {
            match &entry.entry_type {
                EntryType::UserMessage => {
                    awaiting_response = true;
                }
                EntryType::AssistantMessage => {
                    if awaiting_response {
                        turns += 1;
                        awaiting_response = false;
                    }
                }
                _ => {}
            }
        }

        turns
    }

    /// Count user messages in the session.
    pub fn user_message_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.entry_type, EntryType::UserMessage))
            .count()
    }

    /// Count assistant messages in the session.
    pub fn assistant_message_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e.entry_type, EntryType::AssistantMessage))
            .count()
    }

    /// Get the duration of the session.
    pub fn duration(&self) -> Duration {
        let start = self.metadata.created_at.as_millis();
        let end = self.metadata.updated_at.as_millis();
        Duration::from_millis(end.saturating_sub(start))
    }
}

/// Configuration for session limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum number of turns allowed (None = unlimited).
    pub max_turns: Option<u32>,
    /// Maximum number of entries allowed (None = unlimited).
    pub max_entries: Option<usize>,
    /// Maximum session duration (None = unlimited).
    pub max_duration: Option<Duration>,
    /// Action to take when limit is reached.
    pub limit_action: LimitAction,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_turns: None,
            max_entries: None,
            max_duration: None,
            limit_action: LimitAction::Error,
        }
    }
}

impl SessionConfig {
    /// Create a new session config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum turns.
    pub fn with_max_turns(mut self, max: u32) -> Self {
        self.max_turns = Some(max);
        self
    }

    /// Set maximum entries.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }

    /// Set maximum duration.
    pub fn with_max_duration(mut self, max: Duration) -> Self {
        self.max_duration = Some(max);
        self
    }

    /// Set limit action.
    pub fn with_limit_action(mut self, action: LimitAction) -> Self {
        self.limit_action = action;
        self
    }
}

/// Action to take when a session limit is reached.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LimitAction {
    /// Return an error.
    #[default]
    Error,
    /// End the session gracefully.
    EndSession,
    /// Ignore and continue (for soft limits).
    Ignore,
}

/// Check result for session limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitCheck {
    /// Within limits.
    Ok,
    /// At the limit but not exceeded.
    AtLimit,
    /// Limit exceeded.
    Exceeded(LimitExceeded),
}

/// Which limit was exceeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitExceeded {
    /// Turn limit exceeded.
    Turns { current: u32, max: u32 },
    /// Entry limit exceeded.
    Entries { current: usize, max: usize },
    /// Duration limit exceeded.
    Duration { current: Duration, max: Duration },
}

impl Session {
    /// Check if the session is within limits.
    pub fn check_limits(&self, config: &SessionConfig) -> LimitCheck {
        // Check turn limit
        if let Some(max_turns) = config.max_turns {
            let current = self.turn_count();
            if current > max_turns {
                return LimitCheck::Exceeded(LimitExceeded::Turns {
                    current,
                    max: max_turns,
                });
            }
            if current == max_turns {
                return LimitCheck::AtLimit;
            }
        }

        // Check entry limit
        if let Some(max_entries) = config.max_entries {
            let current = self.entries.len();
            if current > max_entries {
                return LimitCheck::Exceeded(LimitExceeded::Entries {
                    current,
                    max: max_entries,
                });
            }
            if current == max_entries {
                return LimitCheck::AtLimit;
            }
        }

        // Check duration limit
        if let Some(max_duration) = config.max_duration {
            let current = self.duration();
            if current > max_duration {
                return LimitCheck::Exceeded(LimitExceeded::Duration {
                    current,
                    max: max_duration,
                });
            }
        }

        LimitCheck::Ok
    }

    /// Check if the session is at its turn limit.
    pub fn is_at_turn_limit(&self, config: &SessionConfig) -> bool {
        if let Some(max_turns) = config.max_turns {
            self.turn_count() >= max_turns
        } else {
            false
        }
    }

    /// Get remaining turns before limit.
    pub fn remaining_turns(&self, config: &SessionConfig) -> Option<u32> {
        config
            .max_turns
            .map(|max| max.saturating_sub(self.turn_count()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id() {
        let id = SessionId::new();
        let id_str = id.as_str();
        let parsed = SessionId::from_string(&id_str).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_timestamp() {
        let ts1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = Timestamp::now();
        assert!(ts2.as_millis() > ts1.as_millis());
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "Test Session",
            SessionType::Conversation {
                purpose: "Testing".to_string(),
            },
        );

        assert_eq!(session.name(), "Test Session");
        assert_eq!(session.status(), SessionStatus::Active);
        assert_eq!(session.entries.len(), 0);
    }

    #[test]
    fn test_session_entries() {
        let mut session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        session.add_user_message("Hello");
        session.add_assistant_message("Hi there!");
        session.add_user_message("How are you?");
        session.add_assistant_message("I'm doing well!");

        assert_eq!(session.entries.len(), 4);
        assert_eq!(session.user_message_count(), 2);
        assert_eq!(session.assistant_message_count(), 2);
    }

    #[test]
    fn test_turn_counting() {
        let mut session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        assert_eq!(session.turn_count(), 0);

        session.add_user_message("Hello");
        assert_eq!(session.turn_count(), 0); // No response yet

        session.add_assistant_message("Hi!");
        assert_eq!(session.turn_count(), 1); // First complete turn

        session.add_user_message("How are you?");
        session.add_assistant_message("Good!");
        assert_eq!(session.turn_count(), 2); // Second complete turn
    }

    #[test]
    fn test_session_limits() {
        let mut session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        let config = SessionConfig::new().with_max_turns(2);

        assert_eq!(session.check_limits(&config), LimitCheck::Ok);
        assert_eq!(session.remaining_turns(&config), Some(2));
        assert!(!session.is_at_turn_limit(&config));

        // First turn
        session.add_user_message("Hello");
        session.add_assistant_message("Hi!");
        assert_eq!(session.remaining_turns(&config), Some(1));

        // Second turn (at limit)
        session.add_user_message("How are you?");
        session.add_assistant_message("Good!");
        assert_eq!(session.check_limits(&config), LimitCheck::AtLimit);
        assert!(session.is_at_turn_limit(&config));
        assert_eq!(session.remaining_turns(&config), Some(0));

        // Third turn (exceeded)
        session.add_user_message("What's up?");
        session.add_assistant_message("Nothing much!");
        assert!(matches!(
            session.check_limits(&config),
            LimitCheck::Exceeded(LimitExceeded::Turns { current: 3, max: 2 })
        ));
    }

    #[test]
    fn test_session_context() {
        let mut session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        session.set_context("key1", serde_json::json!("value1"));
        session.set_context("key2", serde_json::json!(42));

        assert_eq!(
            session.get_context("key1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(session.get_context("key2"), Some(&serde_json::json!(42)));
        assert_eq!(session.get_context("key3"), None);
    }

    #[test]
    fn test_session_status() {
        let mut session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        assert_eq!(session.status(), SessionStatus::Active);

        session.complete();
        assert_eq!(session.status(), SessionStatus::Completed);

        session.fail();
        assert_eq!(session.status(), SessionStatus::Failed);
    }

    #[test]
    fn test_session_serialization() {
        let mut session = Session::new(
            "Test",
            SessionType::TeacherDemo {
                task: "Demo task".to_string(),
                model: "gpt-4".to_string(),
            },
        );

        session.add_user_message("Hello");
        session.add_assistant_message("Hi!");
        session.set_context("key", serde_json::json!("value"));

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name(), session.name());
        assert_eq!(deserialized.entries.len(), 2);
        assert_eq!(
            deserialized.get_context("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_session_entry_types() {
        let tool_entry =
            SessionEntry::tool_call("my_tool", true, serde_json::json!({"result": "ok"}));
        assert!(matches!(
            tool_entry.entry_type,
            EntryType::ToolCall { tool_name, success } if tool_name == "my_tool" && success
        ));

        let skill_entry = SessionEntry::skill_execution("my_skill", false, serde_json::json!({}));
        assert!(matches!(
            skill_entry.entry_type,
            EntryType::SkillExecution { skill_name, success } if skill_name == "my_skill" && !success
        ));
    }
}
