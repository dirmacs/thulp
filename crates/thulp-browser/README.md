# thulp-browser

Browser automation and web scraping utilities for Thulp.

## Overview

This crate provides tools for web page fetching, HTML content extraction, and browser automation. It supports basic HTTP fetching out of the box and optional Chrome DevTools Protocol (CDP) integration for full browser automation.

## Features

- **Web Page Fetching**: Simple async HTTP client for fetching web pages
- **HTML Content Extraction**: Extract text content and page titles from HTML
- **CDP Support**: Optional Chrome DevTools Protocol integration for advanced browser automation
- **Page Metadata**: Access page URL, status code, title, and content
- **Async Design**: Built on tokio and reqwest for efficient async operations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
thulp-browser = "0.2"
```

For CDP browser automation support:

```toml
[dependencies]
thulp-browser = { version = "0.2", features = ["cdp"] }
```

## Usage

### Basic Web Fetching

```rust
use thulp_browser::WebClient;

#[tokio::main]
async fn main() -> Result<(), thulp_browser::BrowserError> {
    let client = WebClient::new();
    let page = client.fetch("https://example.com").await?;

    println!("URL: {}", page.url);
    println!("Status: {}", page.status);
    println!("Title: {:?}", page.title);
    println!("Content length: {} bytes", page.len());

    // Extract text content (HTML tags stripped)
    println!("Text: {}", page.text());

    Ok(())
}
```

### Working with Page Content

```rust
use thulp_browser::{Page, WebClient};

#[tokio::main]
async fn main() -> Result<(), thulp_browser::BrowserError> {
    let client = WebClient::new();
    let page = client.fetch("https://example.com").await?;

    // Check if fetch was successful
    if page.status == 200 {
        // Access raw HTML
        println!("HTML: {}", page.html);

        // Get text without HTML tags
        let text = page.text();

        // Check page title
        if let Some(title) = &page.title {
            println!("Page title: {}", title);
        }
    }

    Ok(())
}
```

### CDP Browser Automation (requires `cdp` feature)

```rust
use thulp_browser::cdp::{Browser, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), thulp_browser::BrowserError> {
    // Configure browser (headless mode)
    let config = BrowserConfig::new().headless(true);

    // Launch browser
    let browser = Browser::launch(config).await?;

    // Create a new page
    let page = browser.new_page().await?;

    // Navigate to URL
    page.navigate("https://example.com").await?;

    // Take a screenshot
    let screenshot = page.screenshot().await?;

    // Evaluate JavaScript
    let result = page.evaluate("document.title").await?;

    Ok(())
}
```

## Page Structure

The `Page` struct contains:

- **url**: The URL of the fetched page
- **html**: Raw HTML content
- **title**: Extracted page title (if found)
- **status**: HTTP status code

## Error Types

The crate provides detailed error types for different failure scenarios:

- `BrowserError::Http`: HTTP request failures
- `BrowserError::Parse`: HTML parsing errors
- `BrowserError::InvalidUrl`: Invalid URL format
- `BrowserError::CdpConnection`: CDP connection failures
- `BrowserError::CdpProtocol`: CDP protocol errors
- `BrowserError::BrowserLaunch`: Browser launch failures
- `BrowserError::Navigation`: Page navigation failures
- `BrowserError::JavaScriptEval`: JavaScript evaluation failures
- `BrowserError::Screenshot`: Screenshot capture failures
- `BrowserError::Timeout`: Operation timeout

## Feature Flags

| Flag | Description |
|------|-------------|
| `cdp` | Enable Chrome DevTools Protocol support |

## Testing

```bash
# Run tests
cargo test -p thulp-browser

# Run tests with CDP feature
cargo test -p thulp-browser --features cdp
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
