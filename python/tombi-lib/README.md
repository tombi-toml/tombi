# tombi-lib

Tombi formatter and linter library API for Python.

```python
import tombi_lib

result = tombi_lib.format("key=1", "example.toml")
assert result.formatted == "key = 1\n"

result = tombi_lib.lint("key =", "example.toml")
assert result.diagnostics
```

`options` accepts a `dict` matching a `tombi.toml` configuration, e.g. `{"config": "[schema]\nenabled = false\n"}`.
Configuration, schema resolution, and I/O failures raise `tombi_lib.TombiError`.
Malformed `options` raise `TypeError`.

`format_async`/`lint_async` are also available for `asyncio` callers:

```python
result = await tombi_lib.format_async("key=1", "example.toml")
```
