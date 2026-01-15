//! MCP Prompts support.
//!
//! This module provides support for MCP prompts protocol methods:
//! - `prompts/list` - List available prompts
//! - `prompts/get` - Get a rendered prompt with arguments

use crate::Result;
use std::collections::HashMap;
use std::sync::RwLock;
use thulp_core::{GetPromptResult, Prompt, PromptListResult, PromptMessage};

/// Type alias for prompt renderer function.
type PromptRenderer = Box<dyn Fn(&HashMap<String, String>) -> GetPromptResult + Send + Sync>;

/// MCP Prompts client for managing and rendering prompts.
pub struct PromptsClient {
    /// Cached prompts
    cache: RwLock<HashMap<String, Prompt>>,
    /// Prompt renderers (name -> renderer function)
    renderers: RwLock<HashMap<String, PromptRenderer>>,
}

impl PromptsClient {
    /// Create a new prompts client.
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            renderers: RwLock::new(HashMap::new()),
        }
    }

    /// List all available prompts.
    ///
    /// In a full implementation, this would call `prompts/list` on the MCP server.
    pub async fn list(&self) -> Result<PromptListResult> {
        let cache = self.cache.read().unwrap();
        Ok(PromptListResult {
            prompts: cache.values().cloned().collect(),
            next_cursor: None,
        })
    }

    /// Get a rendered prompt with arguments.
    ///
    /// In a full implementation, this would call `prompts/get` on the MCP server.
    pub async fn get(&self, name: &str, arguments: HashMap<String, String>) -> Result<GetPromptResult> {
        let renderers = self.renderers.read().unwrap();
        
        if let Some(renderer) = renderers.get(name) {
            Ok(renderer(&arguments))
        } else {
            // Default: return a simple message
            let prompt = self.cache.read().unwrap().get(name).cloned();
            let description = prompt.and_then(|p| p.description);
            
            Ok(GetPromptResult {
                description,
                messages: vec![PromptMessage::user_text(format!(
                    "Prompt '{}' with args: {:?}",
                    name, arguments
                ))],
            })
        }
    }

    /// Register a prompt definition.
    pub fn register(&self, prompt: Prompt) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(prompt.name.clone(), prompt);
    }

    /// Register a prompt with a custom renderer.
    pub fn register_with_renderer<F>(&self, prompt: Prompt, renderer: F)
    where
        F: Fn(&HashMap<String, String>) -> GetPromptResult + Send + Sync + 'static,
    {
        let name = prompt.name.clone();
        self.register(prompt);
        let mut renderers = self.renderers.write().unwrap();
        renderers.insert(name, Box::new(renderer));
    }

    /// Get a prompt definition by name.
    pub fn get_definition(&self, name: &str) -> Option<Prompt> {
        self.cache.read().unwrap().get(name).cloned()
    }

    /// Clear all cached prompts.
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
        self.renderers.write().unwrap().clear();
    }
}

impl Default for PromptsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use thulp_core::PromptArgument;

    #[tokio::test]
    async fn test_prompts_client_creation() {
        let client = PromptsClient::new();
        let result = client.list().await.unwrap();
        assert!(result.prompts.is_empty());
    }

    #[tokio::test]
    async fn test_register_prompt() {
        let client = PromptsClient::new();
        let prompt = Prompt::new("test_prompt");
        client.register(prompt);

        let result = client.list().await.unwrap();
        assert_eq!(result.prompts.len(), 1);
    }

    #[tokio::test]
    async fn test_get_prompt() {
        let client = PromptsClient::new();
        let prompt = Prompt::builder("greeting")
            .description("A greeting prompt")
            .argument(PromptArgument::required("name", "Person to greet"))
            .build();
        
        client.register(prompt);

        let args = HashMap::from([("name".to_string(), "Alice".to_string())]);
        let result = client.get("greeting", args).await.unwrap();
        
        assert_eq!(result.description, Some("A greeting prompt".to_string()));
    }

    #[tokio::test]
    async fn test_get_with_renderer() {
        let client = PromptsClient::new();
        let prompt = Prompt::builder("greeting")
            .description("A greeting prompt")
            .argument(PromptArgument::required("name", "Person to greet"))
            .build();

        client.register_with_renderer(prompt, |args| {
            let name = args.get("name").map(|s| s.as_str()).unwrap_or("World");
            GetPromptResult::new(vec![
                PromptMessage::user_text(format!("Hello, {}!", name)),
            ])
        });

        let args = HashMap::from([("name".to_string(), "Alice".to_string())]);
        let result = client.get("greeting", args).await.unwrap();
        
        assert_eq!(result.messages.len(), 1);
    }

    #[tokio::test]
    async fn test_get_definition() {
        let client = PromptsClient::new();
        let prompt = Prompt::builder("test")
            .title("Test Prompt")
            .build();
        
        client.register(prompt);

        let def = client.get_definition("test");
        assert!(def.is_some());
        assert_eq!(def.unwrap().title, Some("Test Prompt".to_string()));
    }

    #[tokio::test]
    async fn test_clear() {
        let client = PromptsClient::new();
        client.register(Prompt::new("test"));
        
        client.clear();
        
        assert!(client.list().await.unwrap().prompts.is_empty());
    }
}
