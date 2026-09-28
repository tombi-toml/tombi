from typing import TypedDict

class Position(TypedDict):
    line: int
    column: int

class Range(TypedDict):
    start: Position
    end: Position

class Diagnostic:
    level: str
    code: str
    message: str
    range: Range
    source_file: str | None

class FormatResult:
    formatted: str | None
    diagnostics: list[Diagnostic]

class LintResult:
    diagnostics: list[Diagnostic]

class TombiError(Exception): ...
class TombiConfigError(TombiError): ...
class TombiSchemaError(TombiError): ...

def format(
    source: str, source_path: str, options: dict | None = None
) -> FormatResult: ...
def lint(source: str, source_path: str, options: dict | None = None) -> LintResult: ...

# Only present when the extension was built with the `async` Cargo feature
# (the published wheel enables it; a custom build may not).
async def format_async(
    source: str, source_path: str, options: dict | None = None
) -> FormatResult: ...
async def lint_async(
    source: str, source_path: str, options: dict | None = None
) -> LintResult: ...

__version__: str
