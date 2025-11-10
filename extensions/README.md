# Tombi Plugins

This directory contains internal crates that will potentially be released as plugins in the future.

## Overview

Tombi is primarily a Language Server for TOML files. However, we recognize that there is a growing demand for more specialized features, particularly for `Cargo.toml` files, such as:
- Definition jumping
- Enhanced completion suggestions
- Other Cargo-specific functionality

## Development Strategy

Instead of immediately releasing these features as plugins, we are taking a more measured approach:

1. **Interface Maturation**: This allows us to refine and mature the plugin interface without external constraints.
2. **Future Plugin Release**: Once the interface is stable and we have confirmed sufficient demand, we will consider releasing these features as official plugins.

## Benefits of This Approach

- **Quality Control**: We can ensure high-quality implementation and proper integration with the core Tombi functionality.
- **Flexible Development**: We can iterate on the design without being constrained by public API commitments.
- **User Feedback**: We can gather feedback on the features before committing to a public plugin API.

## Current Status

The plugins in this directory are considered experimental and subject to change. They are not yet ready for public use as standalone plugins.
