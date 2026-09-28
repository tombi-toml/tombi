import importlib.metadata

from ._tombi_lib import (
    Diagnostic,
    FormatResult,
    LintResult,
    TombiConfigError,
    TombiError,
    TombiSchemaError,
    format,
    format_async,
    lint,
    lint_async,
)

__version__ = importlib.metadata.version("tombi-lib")

__all__ = [
    "Diagnostic",
    "FormatResult",
    "LintResult",
    "TombiConfigError",
    "TombiError",
    "TombiSchemaError",
    "__version__",
    "format",
    "format_async",
    "lint",
    "lint_async",
]
