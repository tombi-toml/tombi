import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdtemp, mkdir, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

test("published versions put stable releases before prereleases", async () => {
  const pages = await mkdtemp(join(tmpdir(), "tombi-versions-"));
  try {
    for (const name of [
      "v1.11.2-rc.2",
      "v1.11.2",
      "v1.9.9",
      "v1.11.2-rc.10",
      "v1.12.0",
      "v1.11.2-rc.1",
      "common",
    ]) {
      await mkdir(join(pages, name));
    }
    await promisify(execFile)(
      process.execPath,
      [fileURLToPath(new URL("./create-docs-versions.mjs", import.meta.url)), pages],
    );
    assert.deepEqual(JSON.parse(await readFile(join(pages, "versions.json"), "utf8")), [
      "main",
      "v1.12.0",
      "v1.11.2",
      "v1.11.2-rc.10",
      "v1.11.2-rc.2",
      "v1.11.2-rc.1",
      "v1.9.9",
    ]);
  } finally {
    await rm(pages, { recursive: true, force: true });
  }
});
