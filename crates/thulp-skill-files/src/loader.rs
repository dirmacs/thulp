//! Skill loader and directory scanner.
//!
//! Discovers and loads skills from configured directories with
//! scope-based priority resolution.

use crate::error::Result;
use crate::parser::SkillFile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scope levels for skill priority.
///
/// Higher scope values take precedence over lower ones when
/// multiple skills have the same name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkillScope {
    /// Lowest priority: project-level skills.
    Project = 0,
    /// Medium priority: user's personal skills.
    Personal = 1,
    /// Highest priority: enterprise/organization skills.
    Enterprise = 2,
    /// Plugin skills (namespaced, don't conflict).
    Plugin = 3,
}

impl std::fmt::Display for SkillScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillScope::Project => write!(f, "project"),
            SkillScope::Personal => write!(f, "personal"),
            SkillScope::Enterprise => write!(f, "enterprise"),
            SkillScope::Plugin => write!(f, "plugin"),
        }
    }
}

/// Configuration for skill loading.
#[derive(Debug, Clone)]
pub struct SkillLoaderConfig {
    /// Project skills directory (e.g., ./.claude/skills/).
    pub project_dir: Option<PathBuf>,
    /// Personal skills directory (e.g., ~/.claude/skills/).
    pub personal_dir: Option<PathBuf>,
    /// Enterprise skills directory.
    pub enterprise_dir: Option<PathBuf>,
    /// Plugin directories.
    pub plugin_dirs: Vec<PathBuf>,
    /// Maximum directory depth to scan.
    pub max_depth: usize,
}

impl Default for SkillLoaderConfig {
    fn default() -> Self {
        Self {
            project_dir: Some(PathBuf::from(".claude/skills")),
            personal_dir: dirs::home_dir().map(|h| h.join(".claude/skills")),
            enterprise_dir: None,
            plugin_dirs: Vec::new(),
            max_depth: 3,
        }
    }
}

impl SkillLoaderConfig {
    /// Create a config with only project directory.
    pub fn project_only(path: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: Some(path.into()),
            personal_dir: None,
            enterprise_dir: None,
            plugin_dirs: Vec::new(),
            max_depth: 3,
        }
    }

    /// Create a config for testing with a single directory.
    pub fn single(path: impl Into<PathBuf>) -> Self {
        Self::project_only(path)
    }
}

/// Loaded skill with scope information.
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    /// The parsed skill file.
    pub file: SkillFile,
    /// Scope level of the skill.
    pub scope: SkillScope,
    /// For plugins, the plugin name prefix.
    pub namespace: Option<String>,
}

impl LoadedSkill {
    /// Get the fully qualified name (with namespace if applicable).
    pub fn qualified_name(&self) -> String {
        match &self.namespace {
            Some(ns) => format!("{}:{}", ns, self.file.effective_name()),
            None => self.file.effective_name(),
        }
    }

    /// Get the effective description.
    pub fn effective_description(&self) -> String {
        self.file.effective_description()
    }

    /// Check if this skill can be invoked by the model.
    pub fn is_model_invocable(&self) -> bool {
        !self.file.frontmatter.disable_model_invocation
    }

    /// Check if this skill can be invoked by the user.
    pub fn is_user_invocable(&self) -> bool {
        self.file.frontmatter.user_invocable
    }
}

/// Skill loader that discovers and loads skills from configured directories.
pub struct SkillLoader {
    config: SkillLoaderConfig,
}

impl SkillLoader {
    /// Create a new skill loader with the given configuration.
    pub fn new(config: SkillLoaderConfig) -> Self {
        Self { config }
    }

    /// Create a skill loader with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SkillLoaderConfig::default())
    }

    /// Load all skills from configured directories.
    pub fn load_all(&self) -> Result<Vec<LoadedSkill>> {
        let mut skills = Vec::new();

        // Load in priority order (lowest first, so higher priority overwrites)
        if let Some(ref dir) = self.config.project_dir {
            skills.extend(self.load_from_directory(dir, SkillScope::Project, None)?);
        }

        if let Some(ref dir) = self.config.personal_dir {
            skills.extend(self.load_from_directory(dir, SkillScope::Personal, None)?);
        }

        if let Some(ref dir) = self.config.enterprise_dir {
            skills.extend(self.load_from_directory(dir, SkillScope::Enterprise, None)?);
        }

        // Load plugins with namespace
        for plugin_dir in &self.config.plugin_dirs {
            if let Some(plugin_name) = plugin_dir.file_name().and_then(|n| n.to_str()) {
                let skills_dir = plugin_dir.join("skills");
                if skills_dir.exists() {
                    skills.extend(self.load_from_directory(
                        &skills_dir,
                        SkillScope::Plugin,
                        Some(plugin_name.to_string()),
                    )?);
                }
            }
        }

        Ok(skills)
    }

    /// Load skills from a single directory.
    fn load_from_directory(
        &self,
        dir: &Path,
        scope: SkillScope,
        namespace: Option<String>,
    ) -> Result<Vec<LoadedSkill>> {
        let mut skills = Vec::new();

        if !dir.exists() {
            return Ok(skills);
        }

        for entry in WalkDir::new(dir)
            .max_depth(self.config.max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("SKILL.md")) {
                match SkillFile::parse(path) {
                    Ok(file) => {
                        skills.push(LoadedSkill {
                            file,
                            scope,
                            namespace: namespace.clone(),
                        });
                    }
                    Err(e) => {
                        // Log warning but continue loading other skills
                        eprintln!("Warning: Failed to load skill at {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(skills)
    }

    /// Resolve skills by priority (higher scope wins for same name).
    pub fn resolve_priority(skills: Vec<LoadedSkill>) -> HashMap<String, LoadedSkill> {
        let mut resolved: HashMap<String, LoadedSkill> = HashMap::new();

        // Sort by scope (lowest first)
        let mut sorted = skills;
        sorted.sort_by_key(|s| s.scope);

        for skill in sorted {
            let name = skill.qualified_name();
            // Higher scope always wins
            if let Some(existing) = resolved.get(&name) {
                if skill.scope >= existing.scope {
                    resolved.insert(name, skill);
                }
            } else {
                resolved.insert(name, skill);
            }
        }

        resolved
    }

    /// Find a skill by name.
    pub fn find_skill<'a>(skills: &'a [LoadedSkill], name: &str) -> Option<&'a LoadedSkill> {
        skills
            .iter()
            .find(|s| s.file.effective_name() == name || s.qualified_name() == name)
    }

    /// Filter skills that are invocable by the model.
    pub fn model_invocable(skills: &[LoadedSkill]) -> Vec<&LoadedSkill> {
        skills.iter().filter(|s| s.is_model_invocable()).collect()
    }

    /// Filter skills that are invocable by the user.
    pub fn user_invocable(skills: &[LoadedSkill]) -> Vec<&LoadedSkill> {
        skills.iter().filter(|s| s.is_user_invocable()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_scope_ordering() {
        assert!(SkillScope::Enterprise > SkillScope::Personal);
        assert!(SkillScope::Personal > SkillScope::Project);
        assert!(SkillScope::Plugin > SkillScope::Enterprise);
    }

    #[test]
    fn test_qualified_name_no_namespace() {
        let skill = LoadedSkill {
            file: create_mock_skill_file("test-skill"),
            scope: SkillScope::Project,
            namespace: None,
        };
        assert_eq!(skill.qualified_name(), "test-skill");
    }

    #[test]
    fn test_qualified_name_with_namespace() {
        let skill = LoadedSkill {
            file: create_mock_skill_file("helper"),
            scope: SkillScope::Plugin,
            namespace: Some("myplugin".to_string()),
        };
        assert_eq!(skill.qualified_name(), "myplugin:helper");
    }

    #[test]
    fn test_resolve_priority() {
        let project_skill = LoadedSkill {
            file: create_mock_skill_file("shared"),
            scope: SkillScope::Project,
            namespace: None,
        };
        let personal_skill = LoadedSkill {
            file: create_mock_skill_file("shared"),
            scope: SkillScope::Personal,
            namespace: None,
        };

        let skills = vec![project_skill, personal_skill];
        let resolved = SkillLoader::resolve_priority(skills);

        // Personal should win over project
        assert_eq!(resolved.get("shared").unwrap().scope, SkillScope::Personal);
    }

    fn create_mock_skill_file(name: &str) -> SkillFile {
        use crate::frontmatter::SkillFrontmatter;

        SkillFile {
            frontmatter: SkillFrontmatter {
                name: Some(name.to_string()),
                ..Default::default()
            },
            content: String::new(),
            path: PathBuf::new(),
            directory: PathBuf::new(),
            supporting_files: Vec::new(),
        }
    }
}
