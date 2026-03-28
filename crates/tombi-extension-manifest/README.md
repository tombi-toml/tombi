# Tombi Extension Manifest Helpers

## What "manifest" means here

In this crate, a "manifest" means a TOML file that defines a project or workspace entry point for an extension target.

Concrete examples:

- `Cargo.toml` for Rust/Cargo projects
- `pyproject.toml` for Python projects

The helpers in this crate are intentionally format-agnostic. They only handle path resolution and ancestor lookup for those manifest files.

## Why this crate exists

Not every `tombi-extension` consumer needs manifest lookup helpers. Keeping these utilities in a dedicated crate avoids giving `tombi-extension` manifest-specific responsibilities that only some extensions use.

## Current users

This crate is currently used by:

- `tombi-extension-cargo`
- `tombi-extension-pyproject`

`tombi-extension-cargo` uses it to locate workspace and path dependency `Cargo.toml` files.

`tombi-extension-pyproject` uses it to locate workspace and member `pyproject.toml` files.
