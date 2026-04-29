<div align="center" style="display: flex; flex-direction: column; gap: 0;">
    <img src="https://raw.githubusercontent.com/tombi-toml/tombi/refs/heads/main/docs/public/tombi.svg" alt="Logo" style="display: block; margin: 0;">
    <img src="https://raw.githubusercontent.com/tombi-toml/tombi/refs/heads/main/docs/public/demo.gif" style="display: block; margin: 0;" />
</div>

<div align="center">

[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/tombi-toml.tombi?label=VS%20Code%20Marketplace&logo=visual-studio-code&labelColor=374151&color=60a5fa)](https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi)
[![Open VSX Registry](https://img.shields.io/open-vsx/v/tombi-toml/tombi?label=Open%20VSX%20Registry&labelColor=374151&color=60a5fa)](https://open-vsx.org/extension/tombi-toml/tombi)
[![JetBrains Marketplace](https://img.shields.io/jetbrains/plugin/v/28017-tombi?label=JetBrains%20Marketplace&labelColor=374151&color=60a5fa)](https://plugins.jetbrains.com/plugin/28017-tombi)
[![Zed Extension](https://img.shields.io/badge/Zed%20Extension-v0.2.0-blue?labelColor=374151&color=60a5fa)](https://zed.dev/extensions/tombi)
[![homebrew](https://img.shields.io/homebrew/v/tombi.jpg?labelColor=374151&color=60a5fa)](https://formulae.brew.sh/formula/tombi)
[![pypi](https://img.shields.io/pypi/v/tombi.jpg?labelColor=374151&color=60a5fa)](https://pypi.python.org/pypi/tombi)
[![npm](https://img.shields.io/npm/v/tombi.jpg?labelColor=374151&color=60a5fa)](https://www.npmjs.com/package/tombi)
[![toml-test](https://github.com/tombi-toml/tombi/actions/workflows/toml-test.yml/badge.svg)](https://github.com/tombi-toml/tombi/actions)
[![GitHub license](https://badgen.net/github/license/tombi-toml/tombi?style=flat-square&labelColor=374151)](https://github.com/tombi-toml/tombi/blob/main/LICENSE)

</div>

<br>

<div align="center">
    <h2 align="center" style="font-size: 2.0em; margin-bottom: 30px;">
        <span aria-hidden="true">🦅&nbsp;</span>
        <strong>TOML Toolkit</strong>
        <span aria-hidden="true">&nbsp;🦅</span>
    </h2>
    Tombi(鳶 <a href="https://ipa-reader.com/?text=toɴbi" style="font-size: 1.2em; color: #007acc; text-decoration: none;">/toɴbi/</a>) provides a Formatter, Linter, and Language Server
    <br><br>
    <span aria-hidden="true">📚</span>
    <a href="https://tombi-toml.github.io/tombi" style="font-size: 1.2em; color: #007acc; text-decoration: none;">Documentation</a>
    <span aria-hidden="true">📚</span>
</div>

<br>

<div align="center">
<h2 align="center" style="font-size: 2.0em; margin-bottom: 30px;">
<strong>Quick Start</strong>
</h2>

To quickly try out Tombi's formatter, you can run:

<code style="display: block; white-space: pre-wrap;">uvx tombi format ./pyproject.toml</code>
</div>

## Why Tombi?

TOML is now critical infrastructure for modern tooling, from `pyproject.toml` to `Cargo.toml`.
Tombi focuses on making those files easier to trust and maintain with one toolkit for formatting, linting, and editor integration.

- Rust implementation built for fast CLI and editor workflows
- Formatter with predictable output, configurable style, and magic trailing comma support
- Linter and schema-aware validation powered by JSON Schema Store
- Language Server with completion, diagnostics, formatting, navigation, and code actions
- Official integrations for VS Code, Open VSX-based editors, JetBrains, and Zed
- Strong standards alignment with TOML spec compliance and `toml-test`

## What You Can Do

| Workflow | Tombi support |
| --- | --- |
| Format TOML consistently | `tombi format` with configurable formatting rules |
| Catch problems early | `tombi lint` with schema-aware validation |
| Improve editor UX | LSP-powered completion, diagnostics, and formatting |
| Work with common ecosystems | Extensions for Cargo, `pyproject.toml`, and Tombi config |
| Keep large files maintainable | Auto-sorting based on schema metadata |

## Installation

Choose the distribution that fits your workflow:

```sh
brew install tombi
```

```sh
uv tool install tombi
```

```sh
npm install -g tombi
```

For one-off usage:

```sh
uvx tombi format ./pyproject.toml
npx tombi lint ./Cargo.toml
```

For more installation options, including `pipx`, `poetry`, and the install script, see the [Installation Guide](https://tombi-toml.github.io/tombi/docs/installation).

## Editor Support

Tombi integrates with editors through its Language Server and official extensions.

- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi)
- [Open VSX Registry](https://open-vsx.org/extension/tombi-toml/tombi)
- [JetBrains Marketplace](https://plugins.jetbrains.com/plugin/28017-tombi)
- [Zed Extension](https://zed.dev/extensions/tombi)
- [Editor setup documentation](https://tombi-toml.github.io/tombi/docs/editors)

## Learn More

- [Documentation](https://tombi-toml.github.io/tombi)
- [Installation](https://tombi-toml.github.io/tombi/docs/installation)
- [Configuration](https://tombi-toml.github.io/tombi/docs/configuration)
- [CLI](https://tombi-toml.github.io/tombi/docs/cli)
- [Formatter](https://tombi-toml.github.io/tombi/docs/formatter)
- [Language Server](https://tombi-toml.github.io/tombi/docs/language-server)

<div align="center">
    <h2 align="center" style="font-size: 2.0em; margin-bottom: 30px;">
        <strong>Support us</strong>
    </h2>
    If you like this project and would like to support us
    <br><br>
    <a href="https://github.com/sponsors/tombi-toml">
        <img src="https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&color=ff69b4" alt="GitHub Sponsor">
    </a>
</div>
