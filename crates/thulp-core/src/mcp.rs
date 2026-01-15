//! MCP Resources and Prompts types.
//!
//! Types for MCP protocol resources and prompts capabilities.

use serde::{Deserialize, Serialize};
/// MCP Resource definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// Unique URI identifier for the resource (RFC3986)
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Human-readable display title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Resource annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

impl Resource {
    /// Create a new resource with required fields.
    pub fn new(uri: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            title: None,
            description: None,
            mime_type: None,
            size: None,
            annotations: None,
        }
    }

    /// Create a builder for a resource.
    pub fn builder(uri: impl Into<String>, name: impl Into<String>) -> ResourceBuilder {
        ResourceBuilder::new(uri, name)
    }
}

/// Builder for Resource.
#[derive(Debug)]
pub struct ResourceBuilder {
    resource: Resource,
}

impl ResourceBuilder {
    /// Create a new resource builder.
    pub fn new(uri: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            resource: Resource::new(uri, name),
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.resource.title = Some(title.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.resource.description = Some(description.into());
        self
    }

    /// Set the MIME type.
    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.resource.mime_type = Some(mime_type.into());
        self
    }

    /// Set the size.
    pub fn size(mut self, size: u64) -> Self {
        self.resource.size = Some(size);
        self
    }

    /// Set the annotations.
    pub fn annotations(mut self, annotations: ResourceAnnotations) -> Self {
        self.resource.annotations = Some(annotations);
        self
    }

    /// Build the resource.
    pub fn build(self) -> Resource {
        self.resource
    }
}

/// Resource annotations for additional metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourceAnnotations {
    /// Target audience ("user" or "assistant")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    /// Priority from 0.0 to 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    /// Last modified timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// Resource contents after reading.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceContents {
    /// The resource URI
    pub uri: String,
    /// MIME type of the content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content (for text resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded blob (for binary resources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

impl ResourceContents {
    /// Create text resource contents.
    pub fn text(uri: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: Some("text/plain".to_string()),
            text: Some(content.into()),
            blob: None,
        }
    }

    /// Create blob resource contents.
    pub fn blob(
        uri: impl Into<String>,
        data: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
            text: None,
            blob: Some(data.into()),
        }
    }
}

/// Resource template with URI template pattern.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceTemplate {
    /// RFC6570 URI template
    pub uri_template: String,
    /// Template name
    pub name: String,
    /// Human-readable title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default MIME type for resources from this template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ResourceTemplate {
    /// Create a new resource template.
    pub fn new(uri_template: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri_template: uri_template.into(),
            name: name.into(),
            title: None,
            description: None,
            mime_type: None,
        }
    }
}

/// MCP Prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// Unique prompt name
    pub name: String,
    /// Human-readable title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Prompt arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

impl Prompt {
    /// Create a new prompt.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: None,
            description: None,
            arguments: None,
        }
    }

    /// Create a builder for a prompt.
    pub fn builder(name: impl Into<String>) -> PromptBuilder {
        PromptBuilder::new(name)
    }
}

/// Builder for Prompt.
#[derive(Debug)]
pub struct PromptBuilder {
    prompt: Prompt,
}

impl PromptBuilder {
    /// Create a new prompt builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            prompt: Prompt::new(name),
        }
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.prompt.title = Some(title.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.prompt.description = Some(description.into());
        self
    }

    /// Add an argument.
    pub fn argument(mut self, arg: PromptArgument) -> Self {
        self.prompt.arguments.get_or_insert_with(Vec::new).push(arg);
        self
    }

    /// Build the prompt.
    pub fn build(self) -> Prompt {
        self.prompt
    }
}

/// Prompt argument definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// Argument name
    pub name: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the argument is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

impl PromptArgument {
    /// Create a new prompt argument.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: None,
        }
    }

    /// Create a required argument.
    pub fn required(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            required: Some(true),
        }
    }

    /// Create an optional argument.
    pub fn optional(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            required: Some(false),
        }
    }
}

/// Message returned when getting a prompt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// Role: "user" or "assistant"
    pub role: String,
    /// Message content
    pub content: PromptContent,
}

impl PromptMessage {
    /// Create a user message with text content.
    pub fn user_text(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: PromptContent::Text {
                r#type: "text".to_string(),
                text: text.into(),
            },
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant_text(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: PromptContent::Text {
                r#type: "text".to_string(),
                text: text.into(),
            },
        }
    }
}

/// Prompt content variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PromptContent {
    /// Text content
    Text { r#type: String, text: String },
    /// Image content
    Image {
        r#type: String,
        data: String,
        mime_type: String,
    },
    /// Audio content
    Audio {
        r#type: String,
        data: String,
        mime_type: String,
    },
    /// Embedded resource
    Resource {
        r#type: String,
        resource: EmbeddedResource,
    },
}

/// Embedded resource in prompt content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedResource {
    /// Resource URI
    pub uri: String,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded blob
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Result of getting a prompt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptResult {
    /// Description of the rendered prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Messages in the prompt
    pub messages: Vec<PromptMessage>,
}

impl GetPromptResult {
    /// Create a new prompt result.
    pub fn new(messages: Vec<PromptMessage>) -> Self {
        Self {
            description: None,
            messages,
        }
    }

    /// Create a prompt result with description.
    pub fn with_description(description: impl Into<String>, messages: Vec<PromptMessage>) -> Self {
        Self {
            description: Some(description.into()),
            messages,
        }
    }
}

/// Result of listing resources.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceListResult {
    /// List of resources
    pub resources: Vec<Resource>,
    /// Pagination cursor for next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Result of listing prompts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptListResult {
    /// List of prompts
    pub prompts: Vec<Prompt>,
    /// Pagination cursor for next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Result of listing resource templates.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceTemplateListResult {
    /// List of templates
    pub resource_templates: Vec<ResourceTemplate>,
    /// Pagination cursor for next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new("file:///path/to/file.txt", "file.txt");
        assert_eq!(resource.uri, "file:///path/to/file.txt");
        assert_eq!(resource.name, "file.txt");
    }

    #[test]
    fn test_resource_builder() {
        let resource = Resource::builder("file:///test.md", "test.md")
            .title("Test File")
            .description("A test markdown file")
            .mime_type("text/markdown")
            .size(1024)
            .build();

        assert_eq!(resource.title, Some("Test File".to_string()));
        assert_eq!(resource.size, Some(1024));
    }

    #[test]
    fn test_resource_contents_text() {
        let contents = ResourceContents::text("file:///test.txt", "Hello, World!");
        assert!(contents.text.is_some());
        assert!(contents.blob.is_none());
    }

    #[test]
    fn test_resource_contents_blob() {
        let contents = ResourceContents::blob("file:///image.png", "base64data", "image/png");
        assert!(contents.blob.is_some());
        assert!(contents.text.is_none());
    }

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new("code_review");
        assert_eq!(prompt.name, "code_review");
    }

    #[test]
    fn test_prompt_builder() {
        let prompt = Prompt::builder("code_review")
            .title("Code Review")
            .description("Review code for best practices")
            .argument(PromptArgument::required("code", "Code to review"))
            .argument(PromptArgument::optional("language", "Programming language"))
            .build();

        assert_eq!(prompt.title, Some("Code Review".to_string()));
        assert_eq!(prompt.arguments.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_prompt_message() {
        let user_msg = PromptMessage::user_text("Please review this code");
        assert_eq!(user_msg.role, "user");

        let asst_msg = PromptMessage::assistant_text("I'll review the code");
        assert_eq!(asst_msg.role, "assistant");
    }

    #[test]
    fn test_get_prompt_result() {
        let result = GetPromptResult::with_description(
            "Code review prompt",
            vec![PromptMessage::user_text("Review this")],
        );
        assert_eq!(result.description, Some("Code review prompt".to_string()));
        assert_eq!(result.messages.len(), 1);
    }

    #[test]
    fn test_resource_serialization() {
        let resource = Resource::builder("file:///test.txt", "test.txt")
            .mime_type("text/plain")
            .build();

        let json = serde_json::to_string(&resource).unwrap();
        let parsed: Resource = serde_json::from_str(&json).unwrap();
        assert_eq!(resource, parsed);
    }

    #[test]
    fn test_prompt_serialization() {
        let prompt = Prompt::builder("test")
            .description("Test prompt")
            .argument(PromptArgument::required("input", "Input text"))
            .build();

        let json = serde_json::to_string(&prompt).unwrap();
        let parsed: Prompt = serde_json::from_str(&json).unwrap();
        assert_eq!(prompt, parsed);
    }
}
