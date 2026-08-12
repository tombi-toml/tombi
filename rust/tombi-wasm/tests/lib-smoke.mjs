import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import init, { format, lint } from "../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm.js";

const wasm = await readFile(new URL("../../../typescript/@tombi-toml/wasm-lib/dist/tombi_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

assert.deepEqual(await format("key=1", "playground.toml"), {
  formatted: "key = 1\n",
  diagnostics: undefined,
});
assert.deepEqual(
  await format("key={nested=1}", "playground.toml", {
    config: 'toml-version = "v1.1.0"',
  }),
  {
    formatted: "key = { nested = 1 }\n",
    diagnostics: undefined,
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
assert.deepEqual(formatDisabledByOverride, { formatted: "key=1", diagnostics: undefined });

const formatError = await format("key =", "playground.toml", {
  config: { content: 'toml-version = "v1.1.0"', path: "tombi.toml" },
});
assert.equal(formatError.formatted, undefined);
assert.ok(Object.hasOwn(formatError, "formatted"));
assert.ok(Array.isArray(formatError.diagnostics));
assert.ok(formatError.diagnostics.length > 0);

assert.deepEqual(await lint("key = 1", "playground.toml"), {});
const { diagnostics } = await lint("key =", "playground.toml", {
  config: { content: 'toml-version = "v1.1.0"', path: "tombi.toml" },
});
assert.ok(Array.isArray(diagnostics));
assert.ok(diagnostics.length > 0);

await assert.rejects(format("key = 1", "playground.toml", { config: "invalid =" }), (error) => {
  assert.equal(typeof error.error, "string");
  return true;
});
