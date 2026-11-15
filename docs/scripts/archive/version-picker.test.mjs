import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { JSDOM } from "jsdom";

const source = await readFile(new URL("./version-picker.js", import.meta.url), "utf8");

async function setup(pathname, hash, pages) {
  const dom = new JSDOM(
    '<header><div class="menu-toggle"></div><div class="header-dropdown"><nav><a>Home</a><a>Docs<button aria-label="Expand section"><svg viewBox="0 0 24 24"></svg></button></a></nav></div></header>',
  );
  let assigned;
  const location = {
    pathname,
    hash,
    search: "",
    assign: (url) => {
      assigned = url;
    },
  };
  const fetch = async (path) => {
    if (path === "/tombi/versions.json") {
      return { ok: true, json: async () => ["main", "v1.11.2", "v1.9.9"] };
    }
    const html = pages[path];
    return {
      ok: html !== undefined,
      headers: { get: () => "text/html" },
      text: async () => html,
    };
  };
  runInNewContext(source, {
    location,
    document: dom.window.document,
    DOMParser: dom.window.DOMParser,
    MutationObserver: dom.window.MutationObserver,
    fetch,
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  return { document: dom.window.document, location, getAssigned: () => assigned };
}

test("desktop switching preserves a section that exists in the target version", async () => {
  const path = "/tombi/v1.11.2/docs/formatter/auto-sorting/";
  const page = await setup(
    "/tombi/main/docs/formatter/auto-sorting/",
    "#example",
    { [path]: '<h2 id="example">Example</h2>' },
  );
  page.document.querySelector(".tombi-version-trigger").click();
  const versions = [...page.document.querySelectorAll(".tombi-version-menu button")];
  assert.deepEqual(versions.map((item) => item.textContent), [
    "main",
    "v1.11.2",
    "v1.9.9",
  ]);
  versions[1].click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(page.getAssigned(), `${path}#example`);
});

test("missing page falls back to its nearest available parent", async () => {
  const path = "/tombi/v1.9.9/docs/formatter/";
  const page = await setup(
    "/tombi/main/docs/formatter/new-feature/",
    "#example",
    { [path]: "<h1>Formatter</h1>" },
  );
  page.document.querySelectorAll(".tombi-version-menu button")[2].click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(page.getAssigned(), path);
});

test("switching after client-side navigation uses the visible page", async () => {
  const path = "/tombi/v1.11.2/docs/installation/";
  const page = await setup(
    "/tombi/main/docs/",
    "",
    { [path]: "<h1>Installation</h1>" },
  );
  page.location.pathname = "/tombi/main/docs/installation/";
  page.document.querySelectorAll(".tombi-version-menu button")[1].click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(page.getAssigned(), path);
});

test("missing section opens the top of the matching page from the mobile menu", async () => {
  const path = "/tombi/v1.11.2/docs/formatter/";
  const page = await setup(
    "/tombi/main/docs/formatter/",
    "#new-section",
    { [path]: "<h1>Formatter</h1>" },
  );
  const mobile = page.document.querySelector(".header-dropdown nav");
  assert.equal(mobile.firstElementChild.className, "tombi-version-mobile");
  const mobileTrigger = mobile.querySelector(".tombi-version-mobile-trigger");
  assert.equal(mobileTrigger.textContent.trim(), "main");
  assert.ok(mobileTrigger.querySelector("svg"));
  mobileTrigger.click();
  assert.ok(mobileTrigger.querySelector("svg").classList.contains("rotate-90"));
  assert.equal(mobile.querySelector(".tombi-version-mobile-panel").hidden, false);
  mobile.querySelectorAll(".tombi-version-mobile-panel .tombi-version-option")[1].click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(page.getAssigned(), path);
});
