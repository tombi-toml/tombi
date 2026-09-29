from typing import Literal

from ._options import ConfigFile as ConfigFile
from ._options import Options as Options

class Position:
    line: int
    column: int

class Range:
    start: Position
    end: Position

class Diagnostic:
    level: Literal["error", "warning"]
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

def format(
    source: str, source_path: str, options: Options | None = None
) -> FormatResult: ...
def lint(
    source: str, source_path: str, options: Options | None = None
) -> LintResult: ...
async def format_async(
    source: str, source_path: str, options: Options | None = None
) -> FormatResult: ...
async def lint_async(
    source: str, source_path: str, options: Options | None = None
) -> LintResult: ...

__version__: str
