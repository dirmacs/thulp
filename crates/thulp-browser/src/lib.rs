//! # thulp-browser
//!
//! Web browser automation and scraping utilities for thulp.
//!
//! This crate provides tools for:
//! - Web page fetching and parsing
//! - HTML content extraction
//! - Basic web scraping operations

use serde::{Deserialize, Serialize};

/// Result type for browser operations
pub type Result<T> = std::result::Result<T, BrowserError>;

/// Errors that can occur in browser operations
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

/// Web page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// The URL of the page
    pub url: String,

    /// The HTML content
    pub html: String,

    /// The page title (if found)
    pub title: Option<String>,

    /// HTTP status code
    pub status: u16,
}

impl Page {
    /// Create a new page
    pub fn new(url: String, html: String, status: u16) -> Self {
        let title = extract_title(&html);
        Self {
            url,
            html,
            title,
            status,
        }
    }

    /// Extract text content from the HTML
    pub fn text(&self) -> String {
        // Simple text extraction - in a real implementation would use html5ever or similar
        strip_html_tags(&self.html)
    }

    /// Get the content length
    pub fn len(&self) -> usize {
        self.html.len()
    }

    /// Check if the page is empty
    pub fn is_empty(&self) -> bool {
        self.html.is_empty()
    }
}

/// Simple web client for fetching pages
pub struct WebClient {
    /// HTTP client
    client: reqwest::Client,
}

impl WebClient {
    /// Create a new web client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch a web page
    pub async fn fetch(&self, url: &str) -> Result<Page> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| BrowserError::Http(e.to_string()))?;

        let status = response.status().as_u16();
        let html = response
            .text()
            .await
            .map_err(|e| BrowserError::Http(e.to_string()))?;

        Ok(Page::new(url.to_string(), html, status))
    }
}

impl Default for WebClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract title from HTML content
fn extract_title(html: &str) -> Option<String> {
    // Simple regex-based title extraction
    let title_start = html.find("<title>")?;
    let title_end = html[title_start..].find("</title>")?;
    let title = &html[title_start + 7..title_start + title_end];
    Some(title.trim().to_string())
}

/// Strip HTML tags from content
fn strip_html_tags(html: &str) -> String {
    // Simple tag stripping - in production would use proper HTML parser
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new(
            "https://example.com".to_string(),
            "<html><title>Test</title><body>Content</body></html>".to_string(),
            200,
        );

        assert_eq!(page.url, "https://example.com");
        assert_eq!(page.status, 200);
        assert_eq!(page.title, Some("Test".to_string()));
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>Test Title</title></head></html>";
        assert_eq!(extract_title(html), Some("Test Title".to_string()));
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "<html><body><p>Hello <b>World</b></p></body></html>";
        let text = strip_html_tags(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<"));
    }

    #[test]
    fn test_web_client_creation() {
        let _client = WebClient::new();
        assert!(true); // Just verify it can be created
    }
}
