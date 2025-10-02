# Project Structure

## Root Directory Organization

```
tombi/
├── crates/              # Rust crates (core implementation)
├── rust/                # Published Rust packages
├── python/              # Python bindings and packages
├── typescript/          # TypeScript/JavaScript packages
├── editors/             # Editor integrations
├── extensions/          # Tombi extensions (plugin system)
├── docs/                # Documentation website (SolidStart)
├── schemas/             # JSON schemas for TOML validation
├── json.schemastore.org/  # Cached SchemaStore schemas
├── json.tombi.dev/      # Custom Tombi schemas
├── design/              # Design documents and specifications
├── toml-test/           # TOML compliance test suite
├── xtask/               # Workspace automation tasks
├── .kiro/               # Kiro spec-driven development files
├── .claude/             # Claude Code configuration
└── dist/                # Build artifacts
```

## Subdirectory Structures

### `crates/` - Core Rust Crates

Organized by functionality layer:

#### Lexing & Parsing
- `tombi-lexer` - TOML tokenizer
- `tombi-parser` - TOML parser (token → AST)
- `tombi-syntax` - Syntax tree definitions
- `tombi-ast` - Abstract syntax tree nodes and traversal

#### JSON Support
- `tombi-json` - JSON parser
- `tombi-json-lexer` - JSON tokenizer
- `tombi-json-syntax` - JSON syntax definitions
- `tombi-json-value` - JSON value representation

#### Analysis & Validation
- `tombi-validator` - Schema validation engine
- `tombi-linter` - Linting rules and diagnostics
- `tombi-schema-store` - Schema loading and caching
- `tombi-comment-directive` - Comment directive parsing
- `tombi-comment-directive-store` - Comment directive storage

#### Transformation
- `tombi-formatter` - Code formatting engine
- `tombi-ast-editor` - AST manipulation utilities

#### Language Server
- `tombi-lsp` - LSP server implementation
- `tombi-extension` - LSP extension traits and implementations
- `tombi-completion` - Code completion logic
- `tombi-definition` - Go-to-definition logic

#### Utilities
- `tombi-config` - Configuration file handling
- `tombi-diagnostic` - Diagnostic message formatting
- `tombi-text` - Text position and range utilities
- `tombi-uri` - URI parsing and handling
- `tombi-cache` - File and HTTP caching
- `tombi-glob` - File pattern matching
- `tombi-document` - TOML document representation
- `tombi-document-tree` - Document tree with full fidelity

#### Specialized Libraries
- `tombi-date-time` - TOML date-time handling
- `tombi-toml-version` - TOML version detection
- `tombi-version-sort` - Semantic version sorting
- `tombi-severity-level` - Diagnostic severity levels
- `tombi-x-keyword` - Extension keyword handling
- `tombi-rg-tree` - Red-green tree implementation (rowan-like)
- `tombi-future` - Platform-agnostic async utilities

### `rust/` - Published Packages

- `tombi-cli/` - Command-line interface
  - `src/main.rs` - Entry point
  - `src/app/command/` - Subcommands (format, lint, lsp, completion)
- `tombi/` - Library crate for embedding
- `tombi-wasm/` - WebAssembly bindings
- `serde_tombi/` - Serde integration for TOML deserialization

### `python/` - Python Packages

- `tombi/` - Main Python package (via Maturin)
  - `src/tombi/` - Python module source
  - `tests/` - Python tests
- `tombi-beta/` - Beta/experimental features

### `typescript/` - TypeScript Packages

- `@tombi-toml/tombi/` - WASM bindings for JavaScript
  - `src/index.ts` - TypeScript entry point
  - `scripts/postinstall.js` - Post-installation setup

### `editors/` - Editor Integrations

- `vscode/` - Visual Studio Code extension
  - `src/main.ts` - Extension entry point
  - `src/lsp/` - LSP client setup
  - `src/command/` - VS Code commands
  - `dist/` - Bundled extension
- `zed/` - Zed editor extension
  - `src/lib.rs` - Zed extension API implementation
- `intellij/` - IntelliJ IDEA plugin

### `extensions/` - Tombi Extensions

Plugin system for domain-specific enhancements:
- `tombi-extension-cargo/` - Cargo.toml specific features
- `tombi-extension-uv/` - uv/Python specific features
- `tombi-extension-tombi/` - Tombi configuration features

### `docs/` - Documentation Website

SolidStart-based documentation site:
- `src/` - Source files (MDX documentation)
- `.output/` - Build output
- `.vinxi/` - Vinxi build artifacts
- `public/` - Static assets

### `.kiro/` - Spec-Driven Development

Kiro framework for AI-assisted development:
- `steering/` - Project-wide AI guidance documents
- `specs/` - Feature specifications and implementation plans

### `.claude/` - Claude Code Configuration

- `commands/` - Custom slash commands for Claude Code

## Code Organization Patterns

### Crate Structure Pattern

Standard layout for each crate:
```
crate-name/
├── Cargo.toml           # Crate manifest
├── src/
│   ├── lib.rs           # Library root (for libraries)
│   ├── main.rs          # Binary root (for binaries)
│   ├── error.rs         # Error types
│   ├── module_name.rs   # Single-file modules
│   └── module_name/     # Multi-file modules
│       ├── mod.rs       # Module root
│       └── submodule.rs # Submodules
├── tests/               # Integration tests
│   └── test_*.rs        # Test files
└── examples/            # Example usage
    └── example.rs
```

### Generated Code Pattern

Some crates use code generation:
- `src/generated/` - Auto-generated code
- `src/generated.rs` - Generated code module
- Generation happens at build time (build.rs) or via xtask

Example: `tombi-ast/src/generated/` contains AST node definitions from `toml.ungram`

### Test Organization Pattern

- **Unit tests**: Inline in source files with `#[cfg(test)]`
- **Integration tests**: Separate `tests/` directory
- **Test utilities**: `tombi-test-lib` for shared fixtures
- **Test naming**: `test_*.rs` for test files, `test_*()` for test functions

## File Naming Conventions

### Rust
- **Modules**: `snake_case.rs` (e.g., `syntax_kind.rs`)
- **Crates**: `kebab-case` (e.g., `tombi-ast-editor`)
- **Types**: `PascalCase` in code
- **Functions**: `snake_case` in code

### TypeScript/JavaScript
- **Files**: `kebab-case.ts` or `camelCase.ts`
- **Components**: `PascalCase.tsx` for React/Solid components
- **Types**: `PascalCase` interfaces and types
- **Functions**: `camelCase`

### Python
- **Modules**: `snake_case.py`
- **Packages**: `snake_case/`
- **Classes**: `PascalCase`
- **Functions**: `snake_case`

## Import Organization

### Rust Imports
Organized by scope:
```rust
// 1. Standard library
use std::collections::HashMap;

// 2. External crates
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::*;

// 3. Workspace crates
use tombi_ast::Root;
use tombi_syntax::SyntaxKind;

// 4. Local crate modules
use crate::error::Error;
use crate::util::helpers;
```

### TypeScript Imports
```typescript
// 1. External packages
import { createEffect } from 'solid-js';
import type { Position } from 'vscode-languageserver-types';

// 2. Local modules
import { formatDocument } from './formatter';
import type { Config } from './types';
```

### Python Imports
```python
# 1. Standard library
import os
from pathlib import Path

# 2. Third-party
import pytest

# 3. Local modules
from tombi import format_file
```

## Key Architectural Principles

### 1. Red-Green Tree Architecture
- **Green tree**: Immutable, memory-efficient tree stored in `tombi-rg-tree`
- **Red tree**: Lazy, reference-based view with parent pointers
- Enables fast, incremental parsing and editing

### 2. Workspace-Centric Development
- All Rust crates share dependencies via workspace `Cargo.toml`
- Version synchronization across the monorepo
- Shared development dependencies

### 3. Platform Abstraction
- HTTP client abstraction: `reqwest` (native), `gloo-net` (WASM), `surf` (alternative)
- Future abstraction: `tombi-future` handles tokio (native) and wasm-bindgen-futures (WASM)
- Feature flags control platform-specific code

### 4. Schema-First Design
- JSON Schema as source of truth for validation
- Schema drives formatting decisions, completion, and documentation
- Extensible schema store with caching

### 5. Comment Preservation
- Full-fidelity AST preserves all trivia (whitespace, comments)
- Comment directives parsed and associated with nodes
- Formatter respects comment positioning

### 6. Extension System
- Domain-specific logic in separate `tombi-extension-*` crates
- Extensions hook into LSP, formatter, and linter
- Examples: Cargo-specific validation, Python PEP 508 parsing

### 7. Layered Architecture
```
CLI / Editor Extensions / WASM
         ↓
    Language Server
         ↓
Formatter / Linter / Validator
         ↓
   AST Editor / Document Tree
         ↓
   Parser / AST / Syntax
         ↓
        Lexer
```

### 8. Error Handling
- Custom error types per crate (typically in `error.rs`)
- `thiserror` for error derivation
- Rich diagnostic information via `tombi-diagnostic`
- Position tracking for all errors

### 9. Testing Strategy
- Unit tests for individual functions
- Integration tests for end-to-end workflows
- Snapshot testing for formatter output
- TOML test suite compliance testing
- Cross-platform testing (native + WASM)
