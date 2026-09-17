# JSON Schema Test Suite (Definition A)

Tombi runner for the official
[JSON-Schema-Test-Suite](https://github.com/json-schema-org/JSON-Schema-Test-Suite).

## Scope (Definition A)

- Dialects: `draft7`, `draft2019-09`, `draft2020-12`
- Tests: **required** only (`optional/` is excluded)
- Instances: **TOML-representable** only (object root, no `null`)
- Validation: `schema.strict = false` (JSON Schema default openness)

Non-representable instances are counted as `skipped`, not failures.

## Usage

```sh
# Fetch the pinned suite into json-schema-test/vendor/ (gitignored)
cargo xtask json-schema-test --fetch-only

# Run all Definition A drafts (exits non-zero while gaps remain)
cargo xtask json-schema-test --allow-fail

# Focus on one draft / file while fixing
cargo xtask json-schema-test --draft draft7 --filter dependencies --allow-fail
```

Or directly:

```sh
cargo run -p json-schema-test -- --allow-fail
```

Official dialect metaschemas are fetched into `vendor/metaschema-cache/` (same
layout as `tombi-cache`) so offline suite runs can resolve `$ref`s to
`json-schema.org` without the developer's `~/.cache/tombi`.
## Suite pin

Upstream commit is pinned in [`src/SUITE_PIN`](./src/SUITE_PIN). Updating the pin
re-downloads the suite on the next run.

## CI

`.github/workflows/json-schema-test-suite.yml` exists but is **manual only**
(`workflow_dispatch`) until Definition A is fully green. Do not enable
`push` / `pull_request` triggers until then.
