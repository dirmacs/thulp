//! Session filtering for queries.
//!
//! This module provides the `SessionFilter` enum for filtering sessions
//! when listing or querying.

use crate::session::{Session, SessionStatus, SessionType, Timestamp};

/// Filter for querying sessions.
///
/// Filters can be combined using `SessionFilter::And` for complex queries.
///
/// # Example
///
/// ```ignore
/// use thulp_workspace::{SessionFilter, SessionStatus};
///
/// // Find active conversation sessions
/// let filter = SessionFilter::And(vec![
///     SessionFilter::ByStatus(SessionStatus::Active),
///     SessionFilter::ByTypeName("conversation".to_string()),
/// ]);
/// ```
#[derive(Debug, Clone)]
pub enum SessionFilter {
    /// Match sessions with the given status.
    ByStatus(SessionStatus),

    /// Match sessions by type name (e.g., "conversation", "teacher_demo").
    ByTypeName(String),

    /// Match sessions that have a specific tag.
    HasTag(String),

    /// Match sessions created after the given timestamp.
    CreatedAfter(Timestamp),

    /// Match sessions created before the given timestamp.
    CreatedBefore(Timestamp),

    /// Match sessions updated after the given timestamp.
    UpdatedAfter(Timestamp),

    /// Match sessions updated before the given timestamp.
    UpdatedBefore(Timestamp),

    /// Match sessions where the name contains the given text.
    NameContains(String),

    /// Match sessions that have a parent session.
    HasParent,

    /// Match sessions with a specific parent session ID.
    WithParent(crate::session::SessionId),

    /// Match sessions that are root sessions (no parent).
    IsRoot,

    /// Combine multiple filters with AND logic.
    And(Vec<SessionFilter>),

    /// Combine multiple filters with OR logic.
    Or(Vec<SessionFilter>),

    /// Negate a filter.
    Not(Box<SessionFilter>),

    /// Match all sessions.
    All,
}

impl SessionFilter {
    /// Check if a session matches this filter.
    pub fn matches(&self, session: &Session) -> bool {
        match self {
            SessionFilter::ByStatus(status) => session.status() == *status,

            SessionFilter::ByTypeName(type_name) => {
                let session_type_name = session_type_name(&session.metadata.session_type);
                session_type_name.eq_ignore_ascii_case(type_name)
            }

            SessionFilter::HasTag(tag) => session.metadata.tags.iter().any(|t| t == tag),

            SessionFilter::CreatedAfter(timestamp) => {
                session.metadata.created_at.as_millis() > timestamp.as_millis()
            }

            SessionFilter::CreatedBefore(timestamp) => {
                session.metadata.created_at.as_millis() < timestamp.as_millis()
            }

            SessionFilter::UpdatedAfter(timestamp) => {
                session.metadata.updated_at.as_millis() > timestamp.as_millis()
            }

            SessionFilter::UpdatedBefore(timestamp) => {
                session.metadata.updated_at.as_millis() < timestamp.as_millis()
            }

            SessionFilter::NameContains(text) => session
                .metadata
                .name
                .to_lowercase()
                .contains(&text.to_lowercase()),

            SessionFilter::HasParent => session.metadata.parent_session.is_some(),

            SessionFilter::WithParent(parent_id) => {
                session.metadata.parent_session.as_ref() == Some(parent_id)
            }

            SessionFilter::IsRoot => session.metadata.parent_session.is_none(),

            SessionFilter::And(filters) => filters.iter().all(|f| f.matches(session)),

            SessionFilter::Or(filters) => filters.iter().any(|f| f.matches(session)),

            SessionFilter::Not(filter) => !filter.matches(session),

            SessionFilter::All => true,
        }
    }

    /// Create an AND filter from two filters.
    pub fn and(self, other: SessionFilter) -> SessionFilter {
        match self {
            SessionFilter::And(mut filters) => {
                filters.push(other);
                SessionFilter::And(filters)
            }
            _ => SessionFilter::And(vec![self, other]),
        }
    }

    /// Create an OR filter from two filters.
    pub fn or(self, other: SessionFilter) -> SessionFilter {
        match self {
            SessionFilter::Or(mut filters) => {
                filters.push(other);
                SessionFilter::Or(filters)
            }
            _ => SessionFilter::Or(vec![self, other]),
        }
    }

    /// Negate this filter.
    pub fn negate(self) -> SessionFilter {
        SessionFilter::Not(Box::new(self))
    }

    /// Create a filter for active sessions.
    pub fn active() -> Self {
        SessionFilter::ByStatus(SessionStatus::Active)
    }

    /// Create a filter for completed sessions.
    pub fn completed() -> Self {
        SessionFilter::ByStatus(SessionStatus::Completed)
    }

    /// Create a filter for failed sessions.
    pub fn failed() -> Self {
        SessionFilter::ByStatus(SessionStatus::Failed)
    }

    /// Create a filter for conversation sessions.
    pub fn conversations() -> Self {
        SessionFilter::ByTypeName("conversation".to_string())
    }

    /// Create a filter for teacher demo sessions.
    pub fn teacher_demos() -> Self {
        SessionFilter::ByTypeName("teacher_demo".to_string())
    }

    /// Create a filter for evaluation sessions.
    pub fn evaluations() -> Self {
        SessionFilter::ByTypeName("evaluation".to_string())
    }

    /// Create a filter for refinement sessions.
    pub fn refinements() -> Self {
        SessionFilter::ByTypeName("refinement".to_string())
    }

    /// Create a filter for agent sessions.
    pub fn agent_sessions() -> Self {
        SessionFilter::ByTypeName("agent".to_string())
    }
}

/// Get the type name for a session type.
fn session_type_name(session_type: &SessionType) -> &'static str {
    match session_type {
        SessionType::TeacherDemo { .. } => "teacher_demo",
        SessionType::Evaluation { .. } => "evaluation",
        SessionType::Refinement { .. } => "refinement",
        SessionType::Conversation { .. } => "conversation",
        SessionType::Agent { .. } => "agent",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Session, SessionId, SessionType};

    fn create_test_session(name: &str) -> Session {
        Session::new(
            name,
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        )
    }

    #[test]
    fn test_filter_by_status() {
        let mut session = create_test_session("Test");

        let filter = SessionFilter::ByStatus(SessionStatus::Active);
        assert!(filter.matches(&session));

        session.complete();
        assert!(!filter.matches(&session));

        let completed_filter = SessionFilter::ByStatus(SessionStatus::Completed);
        assert!(completed_filter.matches(&session));
    }

    #[test]
    fn test_filter_by_type_name() {
        let conversation = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        let evaluation = Session::new(
            "Test",
            SessionType::Evaluation {
                skill_name: "test".to_string(),
                test_cases: 10,
            },
        );

        let conv_filter = SessionFilter::ByTypeName("conversation".to_string());
        assert!(conv_filter.matches(&conversation));
        assert!(!conv_filter.matches(&evaluation));

        let eval_filter = SessionFilter::ByTypeName("evaluation".to_string());
        assert!(eval_filter.matches(&evaluation));
        assert!(!eval_filter.matches(&conversation));
    }

    #[test]
    fn test_filter_has_tag() {
        let mut session = create_test_session("Test");
        session.metadata.tags.push("important".to_string());

        let filter = SessionFilter::HasTag("important".to_string());
        assert!(filter.matches(&session));

        let no_tag_filter = SessionFilter::HasTag("other".to_string());
        assert!(!no_tag_filter.matches(&session));
    }

    #[test]
    fn test_filter_name_contains() {
        let session = create_test_session("My Test Session");

        let filter = SessionFilter::NameContains("test".to_string());
        assert!(filter.matches(&session));

        let no_match = SessionFilter::NameContains("other".to_string());
        assert!(!no_match.matches(&session));
    }

    #[test]
    fn test_filter_and() {
        let mut session = create_test_session("Test");
        session.metadata.tags.push("important".to_string());

        let filter = SessionFilter::active().and(SessionFilter::HasTag("important".to_string()));

        assert!(filter.matches(&session));

        session.complete();
        assert!(!filter.matches(&session));
    }

    #[test]
    fn test_filter_or() {
        let mut session = create_test_session("Test");

        let filter = SessionFilter::completed().or(SessionFilter::active());

        assert!(filter.matches(&session)); // Active

        session.complete();
        assert!(filter.matches(&session)); // Completed

        session.fail();
        assert!(!filter.matches(&session)); // Failed - neither active nor completed
    }

    #[test]
    fn test_filter_not() {
        let session = create_test_session("Test");

        let filter = SessionFilter::completed().negate();
        assert!(filter.matches(&session)); // Not completed (active)

        let mut completed = create_test_session("Test");
        completed.complete();
        assert!(!filter.matches(&completed)); // Is completed
    }

    #[test]
    fn test_filter_parent() {
        let mut session = create_test_session("Test");

        let has_parent = SessionFilter::HasParent;
        let is_root = SessionFilter::IsRoot;

        assert!(!has_parent.matches(&session));
        assert!(is_root.matches(&session));

        session.metadata.parent_session = Some(SessionId::new());
        assert!(has_parent.matches(&session));
        assert!(!is_root.matches(&session));
    }

    #[test]
    fn test_filter_all() {
        let session = create_test_session("Test");
        assert!(SessionFilter::All.matches(&session));
    }

    #[test]
    fn test_convenience_constructors() {
        let session = Session::new(
            "Test",
            SessionType::Conversation {
                purpose: "Test".to_string(),
            },
        );

        assert!(SessionFilter::conversations().matches(&session));
        assert!(!SessionFilter::evaluations().matches(&session));
        assert!(SessionFilter::active().matches(&session));
    }
}
