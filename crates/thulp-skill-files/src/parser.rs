//! SKILL.md file parser.
//!
//! Parses SKILL.md files with optional YAML frontmatter and markdown content.

use crate::error::{Result, SkillFileError};
use crate::frontmatter::SkillFrontmatter;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const FRONTMATTER_DELIMITER: &str = "---";

/// A supporting file in the skill directory.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportingFile {
    /// File name.
    pub name: String,
    /// Full path to the file.
    pub path: PathBuf,
    /// Classification of the file.
    pub file_type: SupportingFileType,
}

/// Type classification for supporting files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupportingFileType {
    /// Template files (.md, .txt in templates/).
    Template,
    /// Example files (in examples/).
    Example,
    /// Script files (in scripts/ or .sh/.py/.js).
    Script,
    /// Reference documentation (.md files).
    Reference,
    /// Other file types.
    Other,
}

/// Parsed skill file with frontmatter and content.
#[derive(Debug, Clone)]
pub struct SkillFile {
    /// Parsed YAML frontmatter.
    pub frontmatter: SkillFrontmatter,

    /// Markdown content (instructions).
    pub content: String,

    /// Path to the SKILL.md file.
    pub path: PathBuf,

    /// Directory containing the skill.
    pub directory: PathBuf,

    /// Supporting files discovered in the directory.
    pub supporting_files: Vec<SupportingFile>,
}

impl SkillFile {
    /// Parse a SKILL.md file from the given path.
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        Self::parse_content(&content, path)
    }

    /// Parse SKILL.md content with path context.
    pub fn parse_content(content: &str, path: &Path) -> Result<Self> {
        let (frontmatter, body) = Self::split_frontmatter(content)?;

        let directory = path
            .parent()
            .ok_or_else(|| SkillFileError::InvalidPath("No parent directory".into()))?
            .to_path_buf();

        let supporting_files = Self::discover_supporting_files(&directory)?;

        Ok(Self {
            frontmatter,
            content: body,
            path: path.to_path_buf(),
            directory,
            supporting_files,
        })
    }

    /// Parse SKILL.md content without path context (for testing).
    pub fn parse_content_only(content: &str) -> Result<(SkillFrontmatter, String)> {
        Self::split_frontmatter(content)
    }

    /// Split content into frontmatter and body.
    fn split_frontmatter(content: &str) -> Result<(SkillFrontmatter, String)> {
        let trimmed = content.trim_start();

        if !trimmed.starts_with(FRONTMATTER_DELIMITER) {
            // No frontmatter, entire content is body
            return Ok((SkillFrontmatter::default(), content.to_string()));
        }

        // Skip the opening delimiter and find the closing one
        let rest = &trimmed[FRONTMATTER_DELIMITER.len()..];

        // Skip any newline after opening delimiter
        let rest = rest.trim_start_matches('\n').trim_start_matches('\r');

        // Find the closing delimiter
        let end_pos = rest
            .find(FRONTMATTER_DELIMITER)
            .ok_or_else(|| SkillFileError::Parse("Missing closing frontmatter delimiter".into()))?;

        let yaml_content = rest[..end_pos].trim();
        let body = rest[end_pos + FRONTMATTER_DELIMITER.len()..]
            .trim_start_matches('\n')
            .trim_start_matches('\r');

        // Parse YAML, allowing empty frontmatter
        let frontmatter: SkillFrontmatter = if yaml_content.is_empty() {
            SkillFrontmatter::default()
        } else {
            serde_yaml::from_str(yaml_content)?
        };

        Ok((frontmatter, body.to_string()))
    }

    /// Discover supporting files in the skill directory.
    fn discover_supporting_files(directory: &Path) -> Result<Vec<SupportingFile>> {
        let mut files = Vec::new();

        if !directory.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(directory)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.file_name() != Some(std::ffi::OsStr::new("SKILL.md")) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let file_type = Self::classify_supporting_file(path, directory);

                files.push(SupportingFile {
                    name,
                    path: path.to_path_buf(),
                    file_type,
                });
            }
        }

        Ok(files)
    }

    /// Classify a supporting file by its location and extension.
    fn classify_supporting_file(path: &Path, base: &Path) -> SupportingFileType {
        let relative = path.strip_prefix(base).unwrap_or(path);
        let components: Vec<_> = relative.components().collect();

        // Check if file is in a special subdirectory
        if components.len() > 1 {
            let first_dir = components[0].as_os_str().to_str().unwrap_or("");
            match first_dir {
                "examples" => return SupportingFileType::Example,
                "scripts" => return SupportingFileType::Script,
                "templates" => return SupportingFileType::Template,
                _ => {}
            }
        }

        // Classify by extension
        match path.extension().and_then(|e| e.to_str()) {
            Some("md") => SupportingFileType::Reference,
            Some("txt") => SupportingFileType::Template,
            Some("sh") | Some("py") | Some("js") | Some("ts") => SupportingFileType::Script,
            _ => SupportingFileType::Other,
        }
    }

    /// Get effective name (from frontmatter or directory name).
    pub fn effective_name(&self) -> String {
        self.frontmatter.name.clone().unwrap_or_else(|| {
            self.directory
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string()
        })
    }

    /// Get effective description (from frontmatter or first paragraph).
    pub fn effective_description(&self) -> String {
        self.frontmatter.description.clone().unwrap_or_else(|| {
            // Extract first paragraph as description
            self.content
                .split("\n\n")
                .next()
                .unwrap_or("")
                .lines()
                .filter(|l| !l.starts_with('#'))
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
                .to_string()
        })
    }

    /// Check if a tool is allowed for this skill.
    ///
    /// Supports wildcard patterns with `*` which matches any characters.
    /// Examples:
    /// - `Bash` matches exactly `Bash`
    /// - `Bash*` matches `Bash`, `Bash(python:test.py)`, etc.
    /// - `Bash(python:*)` matches `Bash(python:foo)`, `Bash(python:bar)`, etc.
    pub fn is_tool_allowed(&self, tool_name: &str) -> bool {
        match &self.frontmatter.allowed_tools {
            Some(allowed) => {
                allowed.iter().any(|pattern| {
                    if pattern.contains('*') {
                        // Convert glob pattern to regex
                        // Escape regex special chars except *, then replace * with .*
                        let regex_pattern = regex::escape(pattern).replace(r"\*", ".*");
                        regex::Regex::new(&format!("^{}$", regex_pattern))
                            .map(|re| re.is_match(tool_name))
                            .unwrap_or(false)
                    } else {
                        tool_name == pattern
                    }
                })
            }
            None => true, // No restrictions
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# My Skill\n\nThis is the skill content.";
        let (fm, body) = SkillFile::parse_content_only(content).unwrap();
        assert!(fm.name.is_none());
        assert!(body.contains("My Skill"));
    }

    #[test]
    fn test_parse_with_frontmatter() {
        let content = r#"---
name: test-skill
description: A test skill
---
# Instructions

Do something useful.
"#;
        let (fm, body) = SkillFile::parse_content_only(content).unwrap();
        assert_eq!(fm.name, Some("test-skill".to_string()));
        assert_eq!(fm.description, Some("A test skill".to_string()));
        assert!(body.contains("Instructions"));
        assert!(body.contains("Do something useful"));
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = r#"---
---
# Just content here
"#;
        let (fm, body) = SkillFile::parse_content_only(content).unwrap();
        assert!(fm.name.is_none());
        assert!(body.contains("Just content here"));
    }

    #[test]
    fn test_missing_closing_delimiter() {
        let content = r#"---
name: broken
This has no closing delimiter
"#;
        let result = SkillFile::parse_content_only(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_allowed_exact_match() {
        let content = r#"---
allowed-tools:
  - Read
  - Write
---
Content
"#;
        let (fm, _) = SkillFile::parse_content_only(content).unwrap();
        let skill = SkillFile {
            frontmatter: fm,
            content: String::new(),
            path: PathBuf::new(),
            directory: PathBuf::new(),
            supporting_files: Vec::new(),
        };

        assert!(skill.is_tool_allowed("Read"));
        assert!(skill.is_tool_allowed("Write"));
        assert!(!skill.is_tool_allowed("Bash"));
    }

    #[test]
    fn test_tool_allowed_wildcard() {
        let content = r#"---
allowed-tools:
  - "Bash(python:*)"
  - Read
---
Content
"#;
        let (fm, _) = SkillFile::parse_content_only(content).unwrap();
        let skill = SkillFile {
            frontmatter: fm,
            content: String::new(),
            path: PathBuf::new(),
            directory: PathBuf::new(),
            supporting_files: Vec::new(),
        };

        // Pattern "Bash(python:*)" matches "Bash(python:...)" - the * matches the inner content
        assert!(skill.is_tool_allowed("Bash(python:test.py)"));
        assert!(skill.is_tool_allowed("Bash(python:run.py)"));
        assert!(skill.is_tool_allowed("Bash(python:)")); // Empty is ok too
        assert!(!skill.is_tool_allowed("Bash(node:test.js)"));
        assert!(!skill.is_tool_allowed("Bash")); // No parens, doesn't match
        assert!(skill.is_tool_allowed("Read"));
    }

    #[test]
    fn test_tool_allowed_no_restrictions() {
        let content = r#"---
name: unrestricted
---
Content
"#;
        let (fm, _) = SkillFile::parse_content_only(content).unwrap();
        let skill = SkillFile {
            frontmatter: fm,
            content: String::new(),
            path: PathBuf::new(),
            directory: PathBuf::new(),
            supporting_files: Vec::new(),
        };

        assert!(skill.is_tool_allowed("Anything"));
        assert!(skill.is_tool_allowed("Read"));
        assert!(skill.is_tool_allowed("Bash"));
    }
}
