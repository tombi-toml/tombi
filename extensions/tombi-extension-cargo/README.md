# Tombi Cargo Extension

An extension that provides Cargo.toml-specific functionality enhancements for the Tombi Language Server.

## Main Features

### Workspace Root Navigation

Provides functionality to jump to the workspace root `Cargo.toml` file (containing the `[workspace]` section)
from a `Cargo.toml` file that has `workspace = true` or `*.workspace = true` settings.

This feature enables quick access from member crates to workspace root configuration in large Rust projects.
