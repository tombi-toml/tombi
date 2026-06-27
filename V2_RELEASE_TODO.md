# v2 Release TODO

This document tracks deprecated features that should be removed or migrated before releasing Tombi v2.

The goal is not only to list deprecated behavior, but to make the pre-release work explicit:

- remove compatibility code that should not survive into v2
- update generated schemas and user-facing docs
- add or update migration notes
- verify that removed configuration no longer appears in completions or schemas

## Checklist

| Task | Status | Notes |
| --- | --- | --- |
| Remove `toml-version = "v1.1.0-preview"` support | TODO | Replace with `v1.1.0` everywhere. |
| Remove `lint.rules.enumerate` comment directive support | TODO | Keep `lint.rules.enum` as the supported spelling. |
| Remove deprecated extension-specific document link feature keys | TODO | See the document link section below. |
| Remove ignored extension-specific declaration navigation feature keys | TODO | See the declaration navigation section below. |
| Regenerate JSON Schemas | TODO | Ensure removed keys disappear from `www.schemastore.org/tombi.json` and directive schemas. |
| Update docs and migration notes | TODO | Document removed v1 compatibility behavior and replacements. |
| Add or update tests for removed keys | TODO | Removed keys should be rejected or absent from schema/completion surfaces. |

## TOML v1.1.0 Preview

`v1.1.0-preview` was used before the TOML v1.1.0 specification was finalized.
Before v2, remove support for the preview spelling and require `v1.1.0`.

```toml
# Remove this
toml-version = "v1.1.0-preview"

# Use this
toml-version = "v1.1.0"
```

Check every place where a TOML version can be specified:

- `toml-version` in `tombi.toml`
- `toml-version` in `[[schemas]]`
- `#:tombi toml-version = "v1.1.0-preview"` document directives
- `x-tombi-toml-version` in JSON Schema

Relevant source:

- `crates/tombi-toml-version/src/lib.rs`
- `docs/src/routes/docs/reference/toml-versions.mdx`

## Comment Directive: lint.rules.enumerate

`lint.rules.enumerate` is a deprecated alias for `lint.rules.enum`.
Before v2, remove the alias and keep only `lint.rules.enum`.

```toml
# Remove this
# tombi: lint.rules.enumerate.disabled = true
license = "Unknown"

# Use this
# tombi: lint.rules.enum.disabled = true
license = "Unknown"
```

Relevant source:

- `crates/tombi-comment-directive/src/value.rs`
- `docs/src/routes/docs/comment-directive/tombi-value-directive.mdx`

## Extension Document Link Feature Keys

The following extension-specific document link feature keys are deprecated.
Before v2, remove these compatibility keys and keep the current document-link controls.

| Deprecated key | Migration |
| --- | --- |
| `extensions."tombi-toml/cargo".lsp.document-link.cargo-toml` | Remove the key. Use `extensions."tombi-toml/cargo".lsp.document-link.enabled` for the whole feature. |
| `extensions."tombi-toml/cargo".lsp.document-link.git` | Remove the key. Use `extensions."tombi-toml/cargo".lsp.document-link.enabled` for the whole feature. |
| `extensions."tombi-toml/cargo".lsp.document-link.path` | Remove the key. Use `extensions."tombi-toml/cargo".lsp.document-link.enabled` for the whole feature. |
| `extensions."tombi-toml/cargo".lsp.document-link.workspace` | Remove the key. Use `extensions."tombi-toml/cargo".lsp.document-link.enabled` for the whole feature. |
| `extensions."tombi-toml/pyproject".lsp.document-link.pyproject-toml` | Remove the key. Use `extensions."tombi-toml/pyproject".lsp.document-link.enabled` or `extensions."tombi-toml/pyproject".lsp.document-link.pypi-org.enabled`. |
| `extensions."tombi-toml/tombi".lsp.document-link.path` | Remove the key. Use `extensions."tombi-toml/tombi".lsp.document-link.enabled` for the whole feature. |

Relevant source:

- `crates/tombi-config/src/extensions/cargo/lsp/document_link.rs`
- `crates/tombi-config/src/extensions/pyproject/lsp/document_link.rs`
- `crates/tombi-config/src/extensions/tombi/lsp/document_link.rs`

## Extension Declaration Navigation Feature Keys

The following declaration navigation feature keys are deprecated and ignored.
Before v2, remove them from the configuration model.

| Deprecated key | Migration |
| --- | --- |
| `extensions."tombi-toml/cargo".lsp.goto-declaration.member` | Remove the key. |
| `extensions."tombi-toml/cargo".lsp.goto-declaration.path` | Remove the key. |
| `extensions."tombi-toml/pyproject".lsp.goto-declaration.path` | Remove the key. |

Relevant source:

- `crates/tombi-config/src/extensions/cargo/lsp/goto_declaration.rs`
- `crates/tombi-config/src/extensions/pyproject/lsp/goto_declaration.rs`

## Verification Before v2

Run the relevant checks after removing deprecated behavior:

- Rust format and crate checks for touched crates
- configuration schema generation
- docs build if user-facing docs are updated
- targeted tests that prove removed keys are no longer accepted or exposed

Also confirm that the JSON Schema `deprecated` keyword itself still works.
Only Tombi's deprecated compatibility options listed above are v2 removal targets.
