import asyncio

import tombi_lib

SCHEMA_DISABLED = {"config": "[schema]\nenabled = false\n"}


def test_format_async_returns_formatted_source():
    async def run():
        return await tombi_lib.format_async("key=1", "example.toml", SCHEMA_DISABLED)

    result = asyncio.run(run())
    assert result.formatted == "key = 1\n"


def test_lint_async_reports_diagnostics_for_invalid_toml():
    async def run():
        return await tombi_lib.lint_async("key =", "example.toml", SCHEMA_DISABLED)

    result = asyncio.run(run())
    assert result.diagnostics
