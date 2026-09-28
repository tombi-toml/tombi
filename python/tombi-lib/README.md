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
Errors raise exactly one of `tombi_lib.TombiConfigError`, `tombi_lib.TombiSchemaError`, or `tombi_lib.TombiIOError`, all subclasses of `tombi_lib.TombiError` (catch-all base).
`tombi_lib.TombiErrorType` is their union, for exhaustive handling. Malformed `options` raise `TypeError`.

```python
from typing import assert_never


def describe(error: tombi_lib.TombiErrorType) -> str:
    match error:
        case tombi_lib.TombiConfigError():
            return "fix tombi.toml"
        case tombi_lib.TombiSchemaError():
            return "schema unavailable"
        case tombi_lib.TombiIOError():
            return "I/O failure"
        case _:
            assert_never(error)
```

`format_async`/`lint_async` are also available for `asyncio` callers:

```python
result = await tombi_lib.format_async("key=1", "example.toml")
```
