use clap::ValueEnum;
use serde::Serialize;

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text output
    #[default]
    Text,
    /// JSON output for programmatic consumption
    Json,
    /// Compact JSON (no pretty-printing)
    JsonCompact,
}

/// Output helper for formatted output
pub struct Output {
    pub format: OutputFormat,
}

impl Output {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn print_text(&self, text: &str) {
        if matches!(self.format, OutputFormat::Text) {
            println!("{}", text);
        }
    }

    pub fn print_json<T: Serialize>(&self, data: &T) {
        match self.format {
            OutputFormat::Text => {}
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data).unwrap());
            }
            OutputFormat::JsonCompact => {
                println!("{}", serde_json::to_string(data).unwrap());
            }
        }
    }

    pub fn is_json(&self) -> bool {
        matches!(self.format, OutputFormat::Json | OutputFormat::JsonCompact)
    }
}
