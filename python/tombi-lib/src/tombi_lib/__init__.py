import importlib.metadata

from ._tombi_lib import (
    Diagnostic,
    FormatResult,
    LintResult,
    TombiConfigError,
    TombiError,
    TombiIOError,
    TombiSchemaError,
    format,
    format_async,
    lint,
    lint_async,
)

# Every error raised by `format`/`lint` is exactly one of these; annotate with
# this alias to narrow them exhaustively (e.g. `match` + `assert_never`).
type TombiErrorType = TombiConfigError | TombiSchemaError | TombiIOError

__version__ = importlib.metadata.version("tombi-lib")

__all__ = [
    "Diagnostic",
    "FormatResult",
    "LintResult",
    "TombiConfigError",
    "TombiError",
    "TombiErrorType",
    "TombiIOError",
    "TombiSchemaError",
    "__version__",
    "format",
    "format_async",
    "lint",
    "lint_async",
]
