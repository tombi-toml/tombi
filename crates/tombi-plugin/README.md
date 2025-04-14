# Tombi Plugin System

## Overview

This crate defines the standard interface for Tombi plugins, providing a consistent framework for extending Tombi's functionality. It establishes a set of traits that plugins must implement to integrate with the Tombi ecosystem.

## Purpose

The main goal of this crate is to:

1. Define clear boundaries between the core Tombi system and its plugins
2. Provide a standardized way for plugins to interact with Tombi
3. Enable a modular architecture where functionality can be extended without modifying the core system
4. Ensure compatibility across different plugin implementations

## Usage

Plugin developers should implement the traits defined in this crate to create new plugins for Tombi. By following this standardized interface, plugins can be easily integrated into the Tombi Language Server and other Tombi components.

```rust
use tombi_plugin::TombiPlugin;

// Implement the TombiPlugin trait for your custom plugin
struct MyCustomPlugin;

impl TombiPlugin for MyCustomPlugin {
    // Implement required methods...
}
```

## Plugin Types

The crate provides interfaces for different types of plugins:

- Language features plugins (completion, hover, etc.)
- Document analysis plugins
- Custom commands plugins
- File format specific plugins (like the Cargo.toml plugin)

Each plugin type has its own trait that extends the base `Plugin` trait with specific functionality.

## Integration with Tombi

Tombi's core components consume these plugin interfaces to provide extended functionality. The plugin system allows Tombi to remain lightweight while supporting a wide range of features through optional plugins.
