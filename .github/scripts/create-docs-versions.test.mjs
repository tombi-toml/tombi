import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { copyFile, mkdtemp, mkdir, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { runInNewContext } from "node:vm";

const script = fileURLToPath(new URL("./create-docs-versions.mjs", import.meta.url));
const template = fileURLToPath(new URL("../pages/index.html", import.meta.url));

async function generate(pages) {
  await copyFile(template, join(pages, "index.html"));
  await promisify(execFile)(process.execPath, [script, pages]);
  return readFile(join(pages, "index.html"), "utf8");
}

function assertRedirect(index, version) {
  assert.ok(index.includes(`content="0; url=${version}/"`));
  assert.ok(index.includes(`<a href="${version}/">`));
  const inlineScript = index.match(/<script>([\s\S]*?)<\/script>/)?.[1];
  assert.ok(inlineScript);
  let destination;
  runInNewContext(inlineScript, {
    location: {
      search: "?lang=ja",
      hash: "#install",
      replace(url) { destination = url; },
    },
  });
  assert.equal(destination, `/tombi/${version}/?lang=ja#install`);
}

test("published versions put stable releases before prereleases", async () => {
  const pages = await mkdtemp(join(tmpdir(), "tombi-versions-"));
  try {
    for (const name of [
      "v1.13.0-rc.1",
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
    const index = await generate(pages);
    assert.deepEqual(JSON.parse(await readFile(join(pages, "versions.json"), "utf8")), [
      "main",
      "v1.13.0-rc.1",
      "v1.12.0",
      "v1.11.2",
      "v1.11.2-rc.10",
      "v1.11.2-rc.2",
      "v1.11.2-rc.1",
      "v1.9.9",
    ]);
    assertRedirect(index, "v1.12.0");
  } finally {
    await rm(pages, { recursive: true, force: true });
  }
});

test("root redirect falls back to main when only prereleases are deployed", async () => {
  const pages = await mkdtemp(join(tmpdir(), "tombi-versions-"));
  try {
    await mkdir(join(pages, "v2.0.0-rc.1"));
    const index = await generate(pages);
    assertRedirect(index, "main");
  } finally {
    await rm(pages, { recursive: true, force: true });
  }
});

test("root redirect falls back to main when no release is deployed", async () => {
  const pages = await mkdtemp(join(tmpdir(), "tombi-versions-"));
  try {
    const index = await generate(pages);
    assert.deepEqual(JSON.parse(await readFile(join(pages, "versions.json"), "utf8")), ["main"]);
    assertRedirect(index, "main");
  } finally {
    await rm(pages, { recursive: true, force: true });
  }
});
