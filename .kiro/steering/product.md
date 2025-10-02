# Product Overview

## What is Tombi?

Tombi (鳶 /toɴbi/) is a feature-rich TOML toolkit that provides comprehensive tooling for working with TOML (Tom's Obvious Minimal Language) configuration files. It offers three core capabilities: formatting, linting, and language server protocol (LSP) support.

## Core Features

### Formatter
- **Automatic code formatting** - Consistent TOML file formatting
- **Schema-aware formatting** - Respects JSON Schema definitions for enhanced formatting decisions
- **Magic trailing comma** - Intelligent array and inline table formatting
- **Comment preservation** - Maintains and properly positions comments during formatting
- **Sort key-value pairs** - Organizes keys according to schema or configuration rules

### Linter
- **Schema validation** - Validates TOML against JSON Schema definitions
- **Rule-based linting** - Detects common issues and anti-patterns
- **Comment directives** - Supports inline directives to control linting behavior
- **Configurable rules** - Customizable severity levels and rule toggles

### Language Server (LSP)
- **Real-time diagnostics** - Immediate feedback on syntax and schema violations
- **Code completion** - Context-aware suggestions for keys and values
- **Hover information** - Display type information and documentation
- **Go to definition** - Navigate to schema definitions
- **Document symbols** - Outline view and quick navigation
- **Code actions** - Quick fixes and refactoring suggestions

## Target Use Cases

### Configuration File Management
- Formatting and validating project configuration files (Cargo.toml, pyproject.toml, package.json equivalents)
- Ensuring consistency across team repositories
- Enforcing organizational standards for TOML files

### Editor Integration
- Providing rich IDE experience for TOML editing
- Supporting VS Code, Zed, IntelliJ, and other LSP-compatible editors
- Real-time validation and formatting on save

### CI/CD Pipelines
- Automated formatting checks in continuous integration
- Schema validation before deployment
- Ensuring configuration correctness in build processes

### Schema-Driven Development
- Leveraging JSON Schema to define TOML structure
- Validating against industry-standard schemas (SchemaStore)
- Creating custom schemas for domain-specific configuration

## Key Value Propositions

### 🚀 Performance
Built in Rust for exceptional speed and efficiency, handling large TOML files and monorepos with ease.

### 🎯 Schema-Aware
First-class JSON Schema support enables intelligent formatting, validation, and editor features based on schema definitions.

### 🔧 Multi-Platform
Available as:
- **CLI tool** - Standalone binary via cargo, npm, or PyPI
- **Editor extensions** - VS Code, Zed, IntelliJ
- **WASM module** - Browser and JavaScript runtime integration
- **Python package** - Integration with Python tooling
- **Rust library** - Embeddable in Rust applications

### 📚 Standards Compliant
- TOML v1.0.0 support
- Compatible with SchemaStore JSON schemas
- Follows LSP specification for editor integration

### 🛠 Developer Experience
- **Quick start** - `uvx tombi format` for immediate use
- **Zero configuration** - Works out of the box with sensible defaults
- **Highly configurable** - Fine-tune behavior via `tombi.toml`
- **Comment directive support** - Per-line or per-block linting control

## Unique Differentiators

1. **Schema-first approach** - Unlike other TOML tools, Tombi deeply integrates JSON Schema for enhanced validation and formatting
2. **Cross-language support** - Native packages for Rust, Python, TypeScript/JavaScript ecosystems
3. **Extension system** - Built-in extensions for popular tools (Cargo, uv, etc.)
4. **Treatment of comments** - Sophisticated comment handling and positioning during formatting
5. **Active development** - Regular updates and community-driven feature additions
