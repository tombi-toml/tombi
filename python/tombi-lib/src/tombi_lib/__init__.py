from importlib.metadata import version as _version

from ._tombi_lib import (
    Diagnostic,
    FormatResult,
    LintResult,
    TombiConfigError,
    TombiError,
    TombiSchemaError,
    format,
    lint,
)

__version__ = _version("tombi-lib")

__all__ = [
    "Diagnostic",
    "FormatResult",
    "LintResult",
    "TombiConfigError",
    "TombiError",
    "TombiSchemaError",
    "__version__",
    "format",
    "lint",
]
