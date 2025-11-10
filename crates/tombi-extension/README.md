# Tombi Extension System

## Overview

This crate defines the standard interface for Tombi extensions, providing a consistent framework for extending Tombi's functionality. It establishes a set of traits that extensions must implement to integrate with the Tombi ecosystem.

## Purpose

The main goal of this crate is to:

1. Define clear boundaries between the core Tombi system and its extensions
2. Provide a standardized way for extensions to interact with Tombi
3. Enable a modular architecture where functionality can be extended without modifying the core system
4. Ensure compatibility across different extension implementations

## Usage

Extension developers should implement the traits defined in this crate to create new extensions for Tombi. By following this standardized interface, extensions can be easily integrated into the Tombi Language Server and other Tombi components.

```rust
use tombi_extension::Extension;

// Implement the Extension trait for your custom extension
struct MyCustomExtension;

impl Extension for MyCustomExtension {
    // Implement required methods...
}
```

## Extension Types

The crate provides interfaces for different types of extensions:

- Language features extensions (completion, hover, etc.)
- Document analysis extensions
- Custom commands extensions
- File format specific extensions (like the `Cargo.toml` extension)

Each extension type has its own trait that extends the base `Extension` trait with specific functionality.

## Integration with Tombi

Tombi's core components consume these extension interfaces to provide extended functionality. The extension system allows Tombi to remain lightweight while supporting a wide range of features through optional extensions.
