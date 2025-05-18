# @tombi-toml/tombi

🦅 Rust binary installer for tombi TOML toolkit

## Overview

This package provides a way to install the Rust-built tombi binary through npm. The appropriate binary for your platform is automatically downloaded during installation.

## Installation

```
npm install -g @tombi-toml/tombi
```

After installation, the `tombi` command becomes available globally.

## Usage

### Format

Format TOML files:

```
tombi format path/to/file.toml
```

Use the `-i` option to edit files in place:

```
tombi format -i path/to/file.toml
```

### Lint

Lint TOML files:

```
tombi lint path/to/file.toml
```

Use the `--fix` option to automatically fix issues when possible:

```
tombi lint --fix path/to/file.toml
```

## Supported Platforms

- macOS (x86_64, aarch64)
- Linux (x86_64)
- Windows (x86_64)
