# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-01

### Added
- SkillExecutor trait for pluggable execution strategies
- ExecutionContext for managing inputs/outputs between steps
- ExecutionHooks trait with TracingHooks and CompositeHooks
- DefaultSkillExecutor implementation using Transport
- Session management with turn counting and limits
- SessionManager for async file-based persistence
- Session filtering and querying capabilities
- thulp-skill-files crate for SKILL.md file parsing
- YAML frontmatter parsing for skill files
- Preprocessor for variable substitution
- Multi-directory skill discovery with scope priority
- Timeout and retry support for skill execution
- TimeoutConfig, RetryConfig, ExecutionConfig types

## [0.2.0] - 2026-01-15

### Added
- Initial public release
- Core types and traits (Tool, Parameter, Transport)
- MCP (Model Context Protocol) integration
- Query engine for tool filtering
- Tool adapter for external definitions
- Browser automation with CDP support
- Skill composition and execution
- Tool registry management
- Guidance and orchestration primitives
- CLI tool for thulp operations

## [0.1.0] - 2025-12-01

### Added
- Initial internal release
- Basic project structure
