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

try:
    # Only present when the extension was built with the `async` Cargo
    # feature (the published wheel enables it; a custom build may not).
    from ._tombi_lib import format_async, lint_async
except ImportError:
    pass
else:
    __all__ += ["format_async", "lint_async"]
