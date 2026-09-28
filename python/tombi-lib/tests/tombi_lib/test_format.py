import pytest
import tombi_lib

SCHEMA_DISABLED = {"config": "[schema]\nenabled = false\n"}


def test_version():
    assert tombi_lib.__version__


def test_format_returns_formatted_source():
    result = tombi_lib.format("key=1", "example.toml", SCHEMA_DISABLED)
    assert result.formatted == "key = 1\n"
    assert result.diagnostics == []


def test_format_config_parse_failure_raises_tombi_error():
    with pytest.raises(tombi_lib.TombiError):
        tombi_lib.format("key = 1", "example.toml", {"config": "invalid ="})


def test_format_invalid_options_raise_type_error():
    with pytest.raises(TypeError):
        tombi_lib.format("key = 1", "example.toml", {"unknown": True})
