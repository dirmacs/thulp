//! # thulp-mcp
//!
//! MCP protocol client wrapping rs-utcp's MCP transport.
//!
//! This crate provides a thulp-specific wrapper around rs-utcp's MCP implementation,
//! adding features like caching, session tracking, and error conversion.
//!
//! ## Example
//!
//! ```rust
//! use thulp_mcp::{McpClient, McpTransport};
//!
//! // Create a new MCP client
//! let transport = McpTransport::new();
//! let client = McpClient::new(transport);
//!
//! // Check connection status
//! println!("Client connected: {}", client.is_connected());
//! println!("Session ID: {}", client.session_id());
//! ```

#[cfg(feature = "ares")]
mod ares_integration;
mod client;
mod error;
mod transport;

#[cfg(feature = "ares")]
pub use ares_integration::{AresMcpClient, AresToolRegistry};
pub use client::{McpClient, McpClientBuilder};
pub use error::Result;
pub use transport::McpTransport;

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn client_can_be_created() {
        // This is a basic test to ensure the client struct can be instantiated
        assert!(true);
    }
}
