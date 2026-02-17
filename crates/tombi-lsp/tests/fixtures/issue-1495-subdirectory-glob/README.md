# Issue #1495: Local schema and subdirectories

This fixture is for testing issue #1495: https://github.com/tombi-toml/tombi/issues/1495

## Problem

Glob patterns with subdirectories (e.g., `subdir/*.toml`) were not correctly matching files in subdirectories.

## Fixture Structure

```
├── tombi.toml       # Config with schema glob pattern
├── schema.json      # Schema requiring 'name' to be a string
├── product.toml     # Valid: name = "taro"
└── subdir/
    └── subproduct.toml  # Valid: name = "jiro"
```

## Expected Behavior

The fixture files are valid by default.  
In `crates/tombi-lsp/tests/test_diagnostic.rs`, diagnostics are asserted by opening each file path with injected text `name = false`.

When glob association works correctly, both `product.toml` and `subdir/subproduct.toml` receive the same schema and both produce:
- `Expected a value of type String, but found Boolean`

## Before the Fix

Before the fix, only `product.toml` would show an error. The `subdir/subproduct.toml` file was not matched by the glob pattern.

## After the Fix

After the fix, both files correctly show validation errors, proving that the glob pattern `subdir/*.toml` now correctly matches files in subdirectories.
