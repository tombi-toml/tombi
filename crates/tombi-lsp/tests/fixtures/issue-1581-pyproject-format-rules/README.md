# Issue #1581: pyproject `tool.tombi.format.rules` triggers false additional-key warnings

This fixture is for testing issue #1581: https://github.com/tombi-toml/tombi/issues/1581

## Problem

In `pyproject.toml`, Tombi reports some valid `[tool.tombi.format.rules]` keys as disallowed:

- `indent-table-key-value-pairs`
- `key-value-equals-sign-alignment`

## Fixture Structure

```
├── tombi.toml      # Associates SchemaStore pyproject schema
└── pyproject.toml  # Minimal reproduction from issue #1581
```

## Expected Behavior

`pyproject.toml` should produce no diagnostics.

## Current Investigation

The embedded `tombi.json` schema still contains these formatter rule definitions, so the
regression appears to be in how the `tool.tombi` sub-schema is resolved from `pyproject.json`,
not in the raw schema content itself.
