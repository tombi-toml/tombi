import importlib.metadata

from ._tombi_lib import (
    Diagnostic,
    FormatResult,
    LintResult,
    TombiError,
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
    "TombiError",
    "__version__",
    "format",
    "format_async",
    "lint",
    "lint_async",
]
