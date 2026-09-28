import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { format, lint } from "../index.js";

// Without an explicit `config`, `tombi.toml` is searched from the cwd and
// would pick up this repository's own config (which enables a network schema
// catalog), so tests pass this to stay hermetic and network-free.
const schemaDisabled = { config: "[schema]\nenabled = false\n" };

test("format", async () => {
  assert.deepEqual(await format("", "playground.toml", schemaDisabled), {
    formatted: "",
    diagnostics: [],
  });
  assert.deepEqual(await format("key=1", "playground.toml", schemaDisabled), {
    formatted: "key = 1\n",
    diagnostics: [],
  });

  const formattedWithConfig = await format("[package]\nname=1", "Cargo.toml", {
    config: {
      content: `
[format.rules]
indent-table-key-value-pairs = true
indent-width = 4

[schema]
enabled = false
`,
      path: "/workspace/tombi.toml",
    },
  });
  assert.equal(formattedWithConfig.formatted, "[package]\n    name = 1\n");

  const formatDisabledByOverride = await format("key=1", "/workspace/generated/output.toml", {
    config: {
      content: `
[schema]
enabled = false

[[overrides]]
files.include = ["generated/*.toml"]

[overrides.format]
enabled = false
`,
      path: "/workspace/tombi.toml",
    },
  });
  assert.deepEqual(formatDisabledByOverride, { formatted: "key=1", diagnostics: [] });

  const formatError = await format("key =", "playground.toml", schemaDisabled);
  assert.equal(Object.hasOwn(formatError, "formatted"), false);
  assert.ok(formatError.diagnostics.length > 0);
  for (const diagnostic of formatError.diagnostics) {
    assert.equal(diagnostic.level, "error");
    assert.equal(typeof diagnostic.code, "string");
    assert.equal(typeof diagnostic.message, "string");
    assert.equal(typeof diagnostic.range.start.line, "number");
    assert.equal(typeof diagnostic.range.end.column, "number");
    assert.ok(Object.hasOwn(diagnostic, "source_file"));
  }
});

test("lint", async () => {
  assert.deepEqual(await lint("key = 1", "playground.toml", schemaDisabled), {
    diagnostics: [],
  });

  const { diagnostics } = await lint("key =", "playground.toml", schemaDisabled);
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
    schemaDisabled,
  );
  assert.ok(warningResult.diagnostics.length > 0);
  assert.ok(warningResult.diagnostics.every((diagnostic) => diagnostic.level === "warning"));
});

test("config errors reject the promise", async () => {
  for (const run of [format, lint]) {
    await assert.rejects(run("key = 1", "playground.toml", { config: "invalid =" }), (error) => {
      assert.ok(error instanceof Error);
      assert.equal(error.name, "TombiConfigError");
      assert.ok(error.message.length > 0);
      return true;
    });
  }
});

test("lint resolves a local schema from the real filesystem", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "tombi-lib-"));
  try {
    await writeFile(
      join(workspace, "schema.json"),
      '{"type":"object","properties":{"key":{"type":"integer"}}}',
    );
    const options = {
      config: {
        content: `
[[schemas]]
path = "schema.json"
include = ["data.toml"]

[schema.catalog]
paths = []
`,
        path: join(workspace, "tombi.toml"),
      },
    };
    const dataPath = join(workspace, "data.toml");

    const violation = await lint('key = "not-an-integer"', dataPath, options);
    assert.ok(violation.diagnostics.length > 0, JSON.stringify(violation));

    const compliant = await lint("key = 1", dataPath, options);
    assert.deepEqual(compliant.diagnostics, []);
  } finally {
    await rm(workspace, { recursive: true, force: true });
  }
});
