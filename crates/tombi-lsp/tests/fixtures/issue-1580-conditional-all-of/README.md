# Issue #1580: conditional branches inside `allOf`

This fixture is for testing issue #1580: https://github.com/tombi-toml/tombi/issues/1580

## Problem

With a JSON Schema using `allOf` plus `if`/`then` conditionals, Tombi previously accepted:

```toml
mode = "a"
component = "y"
```

even though `component` should be restricted to `"x"` when `mode = "a"`.

## Fixture Structure

```
├── schema.json   # Minimal draft 2020-12 schema from issue #1580
├── tombi.toml    # Associates the schema with TOML files in this fixture
└── input.toml    # Invalid reproduction from issue #1580
```

## Expected Behavior

`input.toml` should produce an enum diagnostic for `component = "y"`.

## After the Fix

Conditional schemas nested under `allOf` are evaluated correctly for this reproduction.
