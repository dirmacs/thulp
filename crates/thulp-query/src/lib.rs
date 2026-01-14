//! # thulp-query
//!
//! Query engine for searching and filtering tools.
//!
//! This crate provides a DSL for querying tool definitions by various criteria.

use serde::{Deserialize, Serialize};
use thulp_core::ToolDefinition;

/// Result type for query operations
pub type Result<T> = std::result::Result<T, QueryError>;

/// Errors that can occur in query operations
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid query: {0}")]
    Invalid(String),
}

/// Query criteria for filtering tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryCriteria {
    /// Match tools by name (supports wildcards)
    Name(String),

    /// Match tools by description keyword
    Description(String),

    /// Match tools with specific parameter
    HasParameter(String),

    /// Match tools with at least N parameters
    MinParameters(usize),

    /// Match tools with at most N parameters
    MaxParameters(usize),

    /// Combine criteria with AND
    And(Vec<QueryCriteria>),

    /// Combine criteria with OR
    Or(Vec<QueryCriteria>),

    /// Negate a criteria
    Not(Box<QueryCriteria>),
}

impl QueryCriteria {
    /// Check if a tool matches this criteria
    pub fn matches(&self, tool: &ToolDefinition) -> bool {
        match self {
            QueryCriteria::Name(pattern) => {
                if pattern.contains('*') {
                    let regex = pattern.replace('*', ".*");
                    regex::Regex::new(&regex)
                        .map(|re| re.is_match(&tool.name))
                        .unwrap_or(false)
                } else {
                    tool.name.contains(pattern)
                }
            }
            QueryCriteria::Description(keyword) => tool
                .description
                .to_lowercase()
                .contains(&keyword.to_lowercase()),
            QueryCriteria::HasParameter(param_name) => {
                tool.parameters.iter().any(|p| p.name == *param_name)
            }
            QueryCriteria::MinParameters(min) => tool.parameters.len() >= *min,
            QueryCriteria::MaxParameters(max) => tool.parameters.len() <= *max,
            QueryCriteria::And(criteria) => criteria.iter().all(|c| c.matches(tool)),
            QueryCriteria::Or(criteria) => criteria.iter().any(|c| c.matches(tool)),
            QueryCriteria::Not(criteria) => !criteria.matches(tool),
        }
    }
}

/// Query builder for constructing queries
#[derive(Debug, Default)]
pub struct QueryBuilder {
    criteria: Vec<QueryCriteria>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Match tools by name
    pub fn name(mut self, pattern: impl Into<String>) -> Self {
        self.criteria.push(QueryCriteria::Name(pattern.into()));
        self
    }

    /// Match tools by description keyword
    pub fn description(mut self, keyword: impl Into<String>) -> Self {
        self.criteria
            .push(QueryCriteria::Description(keyword.into()));
        self
    }

    /// Match tools with specific parameter
    pub fn has_parameter(mut self, param_name: impl Into<String>) -> Self {
        self.criteria
            .push(QueryCriteria::HasParameter(param_name.into()));
        self
    }

    /// Match tools with at least N parameters
    pub fn min_parameters(mut self, min: usize) -> Self {
        self.criteria.push(QueryCriteria::MinParameters(min));
        self
    }

    /// Match tools with at most N parameters
    pub fn max_parameters(mut self, max: usize) -> Self {
        self.criteria.push(QueryCriteria::MaxParameters(max));
        self
    }

    /// Build the query
    pub fn build(self) -> Query {
        Query {
            criteria: if self.criteria.len() == 1 {
                self.criteria.into_iter().next().unwrap()
            } else {
                QueryCriteria::And(self.criteria)
            },
        }
    }
}

/// A query for filtering tools
#[derive(Debug, Clone)]
pub struct Query {
    criteria: QueryCriteria,
}

impl Query {
    /// Create a new query from criteria
    pub fn new(criteria: QueryCriteria) -> Self {
        Self { criteria }
    }

    /// Execute the query on a collection of tools
    pub fn execute(&self, tools: &[ToolDefinition]) -> Vec<ToolDefinition> {
        tools
            .iter()
            .filter(|tool| self.criteria.matches(tool))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thulp_core::Parameter;

    fn create_test_tool(name: &str, description: &str, param_count: usize) -> ToolDefinition {
        let mut builder = ToolDefinition::builder(name).description(description);

        for i in 0..param_count {
            builder = builder.parameter(Parameter::required_string(format!("param{}", i)));
        }

        builder.build()
    }

    #[test]
    fn test_query_name() {
        let tool = create_test_tool("file_read", "Read a file", 1);
        let criteria = QueryCriteria::Name("file".to_string());
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_name_wildcard() {
        let tool = create_test_tool("file_read", "Read a file", 1);
        let criteria = QueryCriteria::Name("file_*".to_string());
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_description() {
        let tool = create_test_tool("file_read", "Read a file from disk", 1);
        let criteria = QueryCriteria::Description("disk".to_string());
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_has_parameter() {
        let tool = ToolDefinition::builder("test")
            .parameter(Parameter::required_string("path"))
            .build();

        let criteria = QueryCriteria::HasParameter("path".to_string());
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_min_parameters() {
        let tool = create_test_tool("test", "Test", 3);
        let criteria = QueryCriteria::MinParameters(2);
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_max_parameters() {
        let tool = create_test_tool("test", "Test", 2);
        let criteria = QueryCriteria::MaxParameters(3);
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_and() {
        let tool = create_test_tool("file_read", "Read a file", 2);
        let criteria = QueryCriteria::And(vec![
            QueryCriteria::Name("file".to_string()),
            QueryCriteria::MinParameters(2),
        ]);
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_or() {
        let tool = create_test_tool("file_read", "Read a file", 1);
        let criteria = QueryCriteria::Or(vec![
            QueryCriteria::Name("network".to_string()),
            QueryCriteria::Name("file".to_string()),
        ]);
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_not() {
        let tool = create_test_tool("file_read", "Read a file", 1);
        let criteria = QueryCriteria::Not(Box::new(QueryCriteria::Name("network".to_string())));
        assert!(criteria.matches(&tool));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new().name("file").min_parameters(1).build();

        let tools = vec![
            create_test_tool("file_read", "Read", 2),
            create_test_tool("network_get", "Get", 1),
        ];

        let results = query.execute(&tools);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "file_read");
    }

    #[test]
    fn test_query_execute() {
        let query = Query::new(QueryCriteria::MinParameters(2));

        let tools = vec![
            create_test_tool("tool1", "Test 1", 1),
            create_test_tool("tool2", "Test 2", 2),
            create_test_tool("tool3", "Test 3", 3),
        ];

        let results = query.execute(&tools);
        assert_eq!(results.len(), 2);
    }
}
