from typing import TypedDict


class ConfigFile(TypedDict):
    """The content of a `tombi.toml` config file at a given path."""

    content: str
    path: str


class Options(TypedDict, total=False):
    """Options shared by the formatter and linter."""

    config: str | ConfigFile
    """An in-memory `tombi.toml` configuration.

    When a string is provided, it is treated as the content of a virtual
    `tombi.toml`. When omitted, `tombi.toml` is searched from the current
    working directory.
    """
