# Technology Stack

## Architecture

Tombi follows a **monorepo architecture** with a Rust core and language-specific bindings for cross-platform distribution. The project uses a workspace-based structure to manage multiple crates, packages, and editor integrations.

### Core Architecture
- **Language**: Rust (edition 2021)
- **Parser**: Custom TOML parser using red-green tree (rowan-like) architecture
- **AST**: Custom abstract syntax tree with full fidelity (preserves all tokens including whitespace)
- **Schema Store**: JSON Schema loading and caching system with HTTP client abstraction
- **LSP Server**: Built on `tower-lsp` for editor integration

## Primary Technologies

### Rust Ecosystem
- **Build System**: Cargo with workspace configuration
- **Toolchain**: Managed via `rust-toolchain.toml`
- **Linter**: Clippy with custom configuration (`clippy.toml`)
- **Formatter**: Rustfmt with custom rules (`rustfmt.toml`)
- **Testing**: Built-in Rust test framework + `rstest` for parameterized tests

#### Key Rust Dependencies
- `tower-lsp` - Language Server Protocol implementation
- `serde` / `serde_json` - Serialization and JSON handling
- `tokio` - Async runtime
- `clap` - CLI argument parsing
- `tracing` - Structured logging and diagnostics
- `indexmap` - Ordered maps for preserving key order
- `reqwest` - HTTP client (native)
- `gloo-net` - HTTP client (WASM)

### JavaScript/TypeScript
- **Package Manager**: pnpm (>=9.12.2, enforced via `only-allow pnpm`)
- **Monorepo Tool**: pnpm workspaces
- **Formatter/Linter**: Biome
- **TypeScript**: Version 5.9.2+
- **Build**: Native modules via Maturin bindings

### Python
- **Package Manager**: uv (modern Python package manager)
- **Build System**: Maturin (Rust to Python bindings)
- **Minimum Version**: Python 3.10+
- **Testing**: pytest
- **Linter**: Ruff

### WebAssembly
- **Target**: `wasm32-unknown-unknown`
- **Bindings**: wasm-bindgen
- **Browser Runtime**: gloo-net for HTTP requests
- **Distribution**: npm package with WASM module

## Development Environment

### Required Tools
```bash
# Core tools
- Rust toolchain (see rust-toolchain.toml)
- Node.js 22+ (see .node-version)
- pnpm 10.12.1+ (managed via packageManager field)
- Python 3.10+ (see .python-version)
- uv (Python package manager)

# Optional tools
- cargo-shear (dependency cleanup)
- maturin (Python bindings)
```

### Common Commands

#### Rust Development
```bash
cargo build              # Build all workspace members
cargo test               # Run all tests
cargo clippy             # Run linter
cargo fmt                # Format code
cargo run --bin tombi    # Run CLI tool
```

#### JavaScript/TypeScript
```bash
pnpm install             # Install dependencies
pnpm format              # Format with Biome
pnpm lint                # Lint with Biome
pnpm vscode              # Run VS Code extension commands
```

#### Python
```bash
uv sync                  # Sync dependencies
uv run pytest            # Run tests
uvx tombi format         # Run tombi formatter
```

#### CLI Usage
```bash
tombi format <file>      # Format TOML file
tombi lint <file>        # Lint TOML file
tombi lsp                # Start language server
tombi completion <shell> # Generate shell completions
```

## Environment Variables

### Development
- `RUST_LOG` - Tracing/logging level (e.g., `debug`, `info`, `tombi_lsp=debug`)
- `RUST_BACKTRACE` - Enable backtrace on panic (1 or full)

### Configuration
- Schema cache location determined by OS:
  - macOS: `~/Library/Caches/tombi/`
  - Linux: `~/.cache/tombi/`
  - Windows: `%LOCALAPPDATA%\tombi\cache\`

## Port Configuration

### Development Servers
- **Documentation site**: Typically runs on port 3000 (SolidStart app in `docs/`)
- **LSP Server**: Communicates via stdio (no port)
- **WASM playground**: Embedded in docs, shares documentation port

## Build Targets

### Native
- **CLI Binary**: `rust/tombi-cli` → `tombi` executable
- **Python Wheel**: Built via Maturin → `tombi` PyPI package
- **Rust Library**: `rust/tombi` → Cargo crate

### WASM
- **Browser Module**: `rust/tombi-wasm` → `@tombi-toml/tombi` npm package
- **Target Triple**: `wasm32-unknown-unknown`

### Editor Extensions
- **VS Code**: TypeScript extension → `.vsix` package
- **Zed**: Rust extension using `zed_extension_api`
- **IntelliJ**: Kotlin/Java plugin (in `editors/intellij`)

## Testing Infrastructure

### Test Organization
- Unit tests: Inline with source code (`#[cfg(test)]`)
- Integration tests: `tests/` directories in each crate
- Snapshot tests: Using `similar` crate for diff comparison
- TOML compliance: `toml-test` directory with official test suite

### Test Utilities
- `tombi-test-lib` - Shared test utilities and fixtures
- `pretty_assertions` - Better assertion failure output
- `rstest` - Parameterized test cases
- `tempfile` - Temporary file handling

## Deployment Targets

### Package Registries
- **Cargo**: crates.io (Rust packages)
- **npm**: npmjs.com (TypeScript/WASM packages)
- **PyPI**: pypi.org (Python packages)
- **VS Code Marketplace**: Visual Studio Marketplace
- **Open VSX**: Open VSX Registry (VS Code alternative)

### Distribution Methods
- **Binary releases**: GitHub Releases with pre-built binaries
- **Package managers**: cargo, npm, pip, uvx
- **Editor marketplaces**: VS Code, Open VSX, JetBrains Marketplace

## Version Management

- **Git tags** used for versioning (not version fields in manifests)
- Placeholder version: `0.0.0-dev` in Cargo.toml and pyproject.toml
- CI/CD triggered by tag creation for releases
- Workspace-wide version synchronization
