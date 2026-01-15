//! MCP Resources support.
//!
//! This module provides support for MCP resources protocol methods:
//! - `resources/list` - List available resources
//! - `resources/read` - Read resource contents
//! - `resources/templates/list` - List resource templates
//! - `resources/subscribe` / `resources/unsubscribe` - Resource subscriptions

use crate::Result;
use std::collections::HashMap;
use std::sync::RwLock;
use thulp_core::{
    Resource, ResourceContents, ResourceListResult, ResourceTemplate, ResourceTemplateListResult,
};

/// MCP Resources client for managing and accessing resources.
pub struct ResourcesClient {
    /// Cached resources
    cache: RwLock<HashMap<String, Resource>>,
    /// Cached templates
    templates_cache: RwLock<Vec<ResourceTemplate>>,
    /// Subscribed resource URIs
    subscriptions: RwLock<Vec<String>>,
}

impl ResourcesClient {
    /// Create a new resources client.
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            templates_cache: RwLock::new(Vec::new()),
            subscriptions: RwLock::new(Vec::new()),
        }
    }

    /// List all available resources.
    ///
    /// In a full implementation, this would call `resources/list` on the MCP server.
    pub async fn list(&self) -> Result<ResourceListResult> {
        let cache = self.cache.read().unwrap();
        Ok(ResourceListResult {
            resources: cache.values().cloned().collect(),
            next_cursor: None,
        })
    }

    /// Read a resource by URI.
    ///
    /// In a full implementation, this would call `resources/read` on the MCP server.
    pub async fn read(&self, uri: &str) -> Result<ResourceContents> {
        // Placeholder - would call MCP server
        Ok(ResourceContents::text(uri, format!("Content of {}", uri)))
    }

    /// List available resource templates.
    ///
    /// In a full implementation, this would call `resources/templates/list`.
    pub async fn list_templates(&self) -> Result<ResourceTemplateListResult> {
        let cache = self.templates_cache.read().unwrap();
        Ok(ResourceTemplateListResult {
            resource_templates: cache.clone(),
            next_cursor: None,
        })
    }

    /// Subscribe to resource changes.
    pub async fn subscribe(&self, uri: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().unwrap();
        if !subs.contains(&uri.to_string()) {
            subs.push(uri.to_string());
        }
        Ok(())
    }

    /// Unsubscribe from resource changes.
    pub async fn unsubscribe(&self, uri: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().unwrap();
        subs.retain(|s| s != uri);
        Ok(())
    }

    /// Get list of subscribed resources.
    pub fn subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().unwrap().clone()
    }

    /// Register a resource (for testing/local use).
    pub fn register(&self, resource: Resource) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(resource.uri.clone(), resource);
    }

    /// Register a template (for testing/local use).
    pub fn register_template(&self, template: ResourceTemplate) {
        let mut cache = self.templates_cache.write().unwrap();
        cache.push(template);
    }

    /// Clear all caches.
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
        self.templates_cache.write().unwrap().clear();
        self.subscriptions.write().unwrap().clear();
    }

    /// Get a resource by URI from cache.
    pub fn get(&self, uri: &str) -> Option<Resource> {
        self.cache.read().unwrap().get(uri).cloned()
    }
}

impl Default for ResourcesClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resources_client_creation() {
        let client = ResourcesClient::new();
        let result = client.list().await.unwrap();
        assert!(result.resources.is_empty());
    }

    #[tokio::test]
    async fn test_register_resource() {
        let client = ResourcesClient::new();
        let resource = Resource::new("file:///test.txt", "test.txt");
        client.register(resource);

        let result = client.list().await.unwrap();
        assert_eq!(result.resources.len(), 1);
    }

    #[tokio::test]
    async fn test_get_resource() {
        let client = ResourcesClient::new();
        client.register(Resource::new("file:///test.txt", "test.txt"));

        let resource = client.get("file:///test.txt");
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().name, "test.txt");
    }

    #[tokio::test]
    async fn test_read_resource() {
        let client = ResourcesClient::new();
        let contents = client.read("file:///test.txt").await.unwrap();
        assert!(contents.text.is_some());
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let client = ResourcesClient::new();

        client.subscribe("file:///test.txt").await.unwrap();
        assert_eq!(client.subscriptions().len(), 1);

        client.unsubscribe("file:///test.txt").await.unwrap();
        assert!(client.subscriptions().is_empty());
    }

    #[tokio::test]
    async fn test_list_templates() {
        let client = ResourcesClient::new();
        client.register_template(ResourceTemplate::new("file:///{path}", "file"));

        let result = client.list_templates().await.unwrap();
        assert_eq!(result.resource_templates.len(), 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let client = ResourcesClient::new();
        client.register(Resource::new("file:///test.txt", "test.txt"));
        client.subscribe("file:///test.txt").await.unwrap();

        client.clear();
        assert!(client.list().await.unwrap().resources.is_empty());
        assert!(client.subscriptions().is_empty());
    }
}
