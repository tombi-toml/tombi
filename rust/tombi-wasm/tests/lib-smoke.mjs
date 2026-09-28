import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, {
  format,
  lint,
  remove_workspace_file,
  set_workspace_file,
} from "../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm.js";

const wasm = await readFile(new URL("../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

assert.deepEqual(await format("", "playground.toml"), {
  formatted: "",
  diagnostics: [],
});
assert.deepEqual(await format("key=1", "playground.toml"), {
  formatted: "key = 1\n",
  diagnostics: [],
});
assert.deepEqual(
  await format("key={nested=1}", "playground.toml", {
    config: 'toml-version = "v1.1.0"',
  }),
  {
    formatted: "key = { nested = 1 }\n",
    diagnostics: [],
  },
);
const formattedWithConfig = await format("[package]\nname=1", "Cargo.toml", {
  config: {
    content: `
[format.rules]
indent-table-key-value-pairs = true
indent-width = 4
`,
    path: "/workspace/tombi.toml",
  },
});
assert.equal(formattedWithConfig.formatted, "[package]\n    name = 1\n");

const formatDisabledByOverride = await format("key=1", "/workspace/generated/output.toml", {
  config: {
    content: `
[[overrides]]
files.include = ["generated/*.toml"]

[overrides.format]
enabled = false
`,
    path: "/workspace/tombi.toml",
  },
});
assert.deepEqual(formatDisabledByOverride, { formatted: "key=1", diagnostics: [] });

const formatError = await format("key =", "playground.toml", {
  config: { content: 'toml-version = "v1.1.0"', path: "tombi.toml" },
});
assert.equal(formatError.formatted, undefined);
assert.equal(Object.hasOwn(formatError, "formatted"), false);
assert.ok(Array.isArray(formatError.diagnostics));
assert.ok(formatError.diagnostics.length > 0);
assert.ok(formatError.diagnostics.every((diagnostic) => diagnostic.level === "error"));

assert.deepEqual(await lint("key = 1", "playground.toml"), { diagnostics: [] });
const { diagnostics } = await lint("key =", "playground.toml", {
  config: { content: 'toml-version = "v1.1.0"', path: "tombi.toml" },
});
assert.ok(Array.isArray(diagnostics));
assert.ok(diagnostics.length > 0);
assert.ok(diagnostics.every((diagnostic) => diagnostic.level === "error"));

const warningResult = await lint(
  `
[fruit.apple]
color = "red"

[animal]
type = "mammal"

[fruit.orange]
color = "orange"
`,
  "playground.toml",
);
assert.ok(warningResult.diagnostics.length > 0);
assert.ok(warningResult.diagnostics.every((diagnostic) => diagnostic.level === "warning"));

await assert.rejects(format("key = 1", "playground.toml", { config: "invalid =" }), (error) => {
  assert.ok(error instanceof Error);
  assert.equal(error.name, "TombiError");
  assert.equal(typeof error.message, "string");
  assert.ok(error.message.length > 0);
  assert.equal(Object.hasOwn(error, "error"), false);
  return true;
});

await assert.rejects(lint("key = 1", "playground.toml", { config: "invalid =" }), (error) => {
  assert.ok(error instanceof Error);
  assert.equal(error.name, "TombiError");
  assert.equal(typeof error.message, "string");
  assert.ok(error.message.length > 0);
  assert.equal(Object.hasOwn(error, "error"), false);
  return true;
});

// Malformed options are a caller bug, rejected as a standard `TypeError`
// rather than a `TombiError`.
for (const run of [format, lint]) {
  for (const options of [{ unknown: true }, { config: 1 }]) {
    await assert.rejects(run("key = 1", "playground.toml", options), TypeError);
  }
}

// wasm-lib's `lint`/`format` resolve `file://` schemas through the same
// injected virtual filesystem as wasm-lsp (set_workspace_file/tombi_fs).
set_workspace_file(
  "file:///workspace/schema.json",
  '{"type":"object","properties":{"key":{"type":"integer"}}}',
);

const schemaConfig = `
[[schemas]]
path = "schema.json"
include = ["data.toml"]
`;

const schemaViolation = await lint('key = "not-an-integer"', "/workspace/data.toml", {
  config: { content: schemaConfig, path: "/workspace/tombi.toml" },
});
assert.ok(
  schemaViolation.diagnostics.length > 0,
  `wasm-lib lint should resolve a local file:// schema injected via set_workspace_file: ${JSON.stringify(schemaViolation)}`,
);

const schemaCompliant = await lint("key = 1", "/workspace/data.toml", {
  config: { content: schemaConfig, path: "/workspace/tombi.toml" },
});
assert.deepEqual(schemaCompliant.diagnostics, []);

remove_workspace_file("file:///workspace/schema.json");

const schemaAfterRemoval = await lint('key = "not-an-integer"', "/workspace/data.toml", {
  config: { content: schemaConfig, path: "/workspace/tombi.toml" },
});
assert.deepEqual(
  schemaAfterRemoval.diagnostics,
  [],
  "removing the injected schema file should stop it from being applied",
);

// `schema.catalog.paths` with a `file://` entry must also resolve through the
// injected virtual filesystem (this exercises load_catalog_from_uri's "file"
// scheme, not just fetch_schema_value's).
set_workspace_file(
  "file:///workspace/schema-from-catalog.json",
  '{"type":"object","properties":{"key":{"type":"integer"}}}',
);
set_workspace_file(
  "file:///workspace/catalog.json",
  JSON.stringify({
    schemas: [
      {
        name: "test",
        description: "desc",
        fileMatch: ["catalog-data.toml"],
        url: "file:///workspace/schema-from-catalog.json",
      },
    ],
  }),
);

const catalogConfig = `
[schema]
enabled = true

[schema.catalog]
paths = ["file:///workspace/catalog.json"]
`;

const catalogViolation = await lint('key = "not-an-integer"', "/workspace/catalog-data.toml", {
  config: { content: catalogConfig, path: "/workspace/tombi.toml" },
});
assert.ok(
  catalogViolation.diagnostics.length > 0,
  `wasm-lib lint should resolve a local file:// catalog injected via set_workspace_file: ${JSON.stringify(catalogViolation)}`,
);

const catalogCompliant = await lint("key = 1", "/workspace/catalog-data.toml", {
  config: { content: catalogConfig, path: "/workspace/tombi.toml" },
});
assert.deepEqual(catalogCompliant.diagnostics, []);
