import pytest
import tombi_lib

SCHEMA_DISABLED = {"config": "[schema]\nenabled = false\n"}


def test_version():
    assert tombi_lib.__version__


def test_format_returns_formatted_source():
    result = tombi_lib.format("key=1", "example.toml", SCHEMA_DISABLED)
    assert result.formatted == "key = 1\n"
    assert result.diagnostics == []


def test_format_config_parse_failure_raises_config_error():
    with pytest.raises(tombi_lib.TombiConfigError):
        tombi_lib.format("key = 1", "example.toml", {"config": "invalid ="})
