# Issue #1566: pyproject `license-files` triggers false additional-key warnings

This fixture is for testing issue #1566: https://github.com/tombi-toml/tombi/issues/1566

## Problem

With strict validation enabled, adding `project.license-files` caused false warnings such as:

- `project` does not allow "name" key
- `project` does not allow "license-files" key
- `project` does not allow "keywords" key
- `project` does not allow "authors" key
- `project` does not allow "classifiers" key
- `project` does not allow "dependencies" key

## Fixture Structure

```
├── tombi.toml      # Associates SchemaStore pyproject schema
└── pyproject.toml  # Minimal reproduction from issue #1566
```

## Expected Behavior

`pyproject.toml` should produce no diagnostics.

## Before the Fix

When `license-files = ["LICENSE"]` existed, dependency schema validation in strict mode
re-validated `[project]` against a partial schema and reported unrelated keys as disallowed.

## After the Fix

Dependency schema validation no longer emits strict additional-key warnings for keys that are
valid in the parent `project` schema.
