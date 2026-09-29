import tombi_lib

SCHEMA_DISABLED = {"config": "[schema]\nenabled = false\n"}


def test_lint_reports_diagnostics_for_invalid_toml():
    result = tombi_lib.lint("key =", "example.toml", SCHEMA_DISABLED)
    assert result.diagnostics

    diagnostic = result.diagnostics[0]
    assert diagnostic.level in ("error", "warning")
    assert diagnostic.message
    assert isinstance(diagnostic.range.start.line, int)
    assert isinstance(diagnostic.range.start.column, int)
    assert isinstance(diagnostic.range, tombi_lib.Range)
    assert isinstance(diagnostic.range.start, tombi_lib.Position)


def test_lint_reports_no_diagnostics_for_valid_toml():
    result = tombi_lib.lint("key = 1", "example.toml", SCHEMA_DISABLED)
    assert result.diagnostics == []
