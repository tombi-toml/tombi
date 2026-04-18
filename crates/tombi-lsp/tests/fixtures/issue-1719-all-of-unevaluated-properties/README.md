# Issue #1719: `allOf` and `unevaluatedProperties` on open

This fixture reproduces issue #1719:
https://github.com/tombi-toml/tombi/issues/1719

## Problem

When a TOML file references a local draft 2020-12 schema via `#:schema`, Tombi
incorrectly reports `Unevaluated property "x" is not allowed` on startup even
though `x` is defined through `allOf`.

## Expected Behavior

`test.toml` should not produce diagnostics immediately after `didOpen`.
