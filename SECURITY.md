# Security policy

## Report a vulnerability

Use [GitHub's private vulnerability reporting for Tombi](https://github.com/tombi-toml/tombi/security/advisories/new). Do not disclose a suspected vulnerability in a public issue or discussion before the maintainers have had a chance to investigate it.

Include the affected Tombi version and distribution, the operating system and editor if relevant, steps to reproduce, the expected and actual behavior, and the security impact. A small TOML file, schema, or configuration that demonstrates the problem is helpful. Remove credentials and other private data from the report.

The maintainers will assess the report and coordinate any fix and disclosure through the private report. Fixes and backports are decided for each case; this policy does not promise support for a specific release line or response deadline.

## Scope and trust boundaries

This policy covers the Tombi code in this repository, including the parser, formatter, linter, language server, editor integrations, extensions, schema handling, cache, and distributed CLI and library packages. TOML documents, project configuration, local and remote schemas, schema catalogs, file paths, and language server messages can contain untrusted data.

Security reviews should pay particular attention to these boundaries:

- Parsing and processing untrusted documents and schemas must not permit code execution, unintended file access or modification, or resource use that makes a normal editor or CLI session unusable.
- Schema and catalog URLs, redirects, DNS results, and proxies must not let project-controlled input reach local or private network services without a grant from the user or embedding application.
- Project files and fetched content must not grant themselves access to credentials or expand the scope of filesystem or network access. Credentials used for authenticated schema requests must not be sent to an unrelated destination or exposed in logs and diagnostics.
- Cache operations and editor or CLI file writes must stay within their intended paths and must not follow attacker-controlled paths or symlinks to modify unrelated files.
- Build and release automation must protect credentials and published artifacts when processing contributions or external inputs.

These are properties to check, not claims that every implementation already satisfies them. A report needs a reachable path from attacker-controlled input to a meaningful security impact. Ordinary formatting differences, validation mistakes without security impact, and requests for third-party schemas belong in the public issue tracker.
