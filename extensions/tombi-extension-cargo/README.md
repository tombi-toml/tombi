# Tombi Cargo Extension

An extension that provides `Cargo.toml`-specific functionality enhancements for the Tombi Language Server.

## Main Features

### Path Lints

Reports missing explicitly configured README, license file, build script, target source, local dependency paths, and workspace definitions for inherited package fields, dependencies, and lints. Each rule is an error by default and can be configured under `extensions."tombi-toml/cargo".lint.rules`.

### Workspace Root Navigation

Provides functionality to jump to the workspace root `Cargo.toml` file (containing the `[workspace]` section) from a `Cargo.toml` file that has `workspace = true` or `*.workspace = true` settings.

This feature enables quick access from member crates to workspace root configuration in large Rust projects.
