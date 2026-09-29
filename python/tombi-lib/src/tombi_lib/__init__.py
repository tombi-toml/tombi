import importlib.metadata

from ._options import ConfigFile, Options
from ._tombi_lib import (
    Diagnostic,
    FormatResult,
    LintResult,
    Position,
    Range,
    TombiError,
    format,
    format_async,
    lint,
    lint_async,
)

__version__ = importlib.metadata.version("tombi-lib")

__all__ = [
    "ConfigFile",
    "Diagnostic",
    "FormatResult",
    "LintResult",
    "Options",
    "Position",
    "Range",
    "TombiError",
    "__version__",
    "format",
    "format_async",
    "lint",
    "lint_async",
]
