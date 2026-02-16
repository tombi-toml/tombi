# Issue #1495: Local schema and subdirectories

This fixture is for testing issue #1495: https://github.com/tombi-toml/tombi/issues/1495

## Problem

Glob patterns with subdirectories (e.g., `subdir/*.toml`) were not correctly matching files in subdirectories.

## Fixture Structure

```
├── tombi.toml       # Config with schema glob pattern
├── schema.json      # Schema requiring 'name' to be an integer
├── product.toml     # Invalid: name = true (Boolean)
└── subdir/
    └── subproduct.toml  # Invalid: name = "some-str" (String)
```

## Expected Behavior

Both `product.toml` and `subdir/subproduct.toml` should fail validation because:
- `product.toml`: `name` is Boolean, but schema requires Integer
- `subdir/subproduct.toml`: `name` is String, but schema requires Integer

The glob pattern `subdir/*.toml` in `tombi.toml` should match `subdir/subproduct.toml`.

## Manual Test

From this directory:

```bash
tombi lint
```

Expected output:
```
2 files linted successfully
2 files failed to be linted

Error: Expected a value of type Integer, but found Boolean
  at .../product.toml:1:8
Error: Expected a value of type Integer, but found String
  at .../subdir/subproduct.toml:1:8
```

## Before the Fix

Before the fix, only `product.toml` would show an error. The `subdir/subproduct.toml` file was not matched by the glob pattern.

## After the Fix

After the fix, both files correctly show validation errors, proving that the glob pattern `subdir/*.toml` now correctly matches files in subdirectories.
