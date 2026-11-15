import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { JSDOM } from "jsdom";

const source = await readFile(new URL("./archive.js", import.meta.url), "utf8");

for (const frameState of ["loaded", "unloaded", "absent"]) {
  test(`archived theme toggle supports playground iframe state: ${frameState}`, async () => {
    const shell = '<header><a id="mobile-logo"></a><button id="dark-mode-toggle"></button><input><div class="header-dropdown"></div></header>';
    const dom = new JSDOM(`<meta name="tombi-archive-base" content="/v1"><div id="archive-header"></div>${frameState === "absent" ? "" : '<iframe class="archive-playground-frame"></iframe>'}`, { url: "https://example.com/v1/playground" });
    const document = dom.window.document;
    const frame = document.querySelector("iframe");
    if (frameState === "unloaded") {
      Object.defineProperty(frame, "contentDocument", { value: null });
    }
    const frameRoot = frame?.contentDocument?.documentElement;
    await runInNewContext(`(async () => {${source}\n})()`, {
      document,
      location: dom.window.location,
      localStorage: dom.window.localStorage,
      fetch: async () => ({
        ok: true,
        text: async () => shell,
        json: async () => [],
      }),
    });

    for (const dark of [true, false]) {
      document.querySelector("#dark-mode-toggle").click();
      assert.equal(document.documentElement.classList.contains("dark"), dark);
      assert.equal(dom.window.localStorage.getItem("theme"), dark ? "dark" : "light");
      if (frameRoot) assert.equal(frameRoot.classList.contains("dark"), dark);
    }
    dom.window.close();
  });
}

for (const layout of ["max-height", "grid"]) {
  test(`archived mobile navigation supports ${layout} shells`, async () => {
    const grid = layout === "grid";
    const panel = (contents) => grid
      ? `<div class="grid grid-rows-[0fr]" inert><div class="min-h-0 overflow-hidden">${contents}</div></div>`
      : `<div style="max-height:0px">${contents}</div>`;
    const item = (name, contents) => `<div><a href="/docs/${name}">${name}<button aria-expanded="false"><svg></svg></button></a>${panel(contents)}</div>`;
    const shell = `<header><a id="mobile-logo"></a><input><div class="header-dropdown ${grid ? "grid grid-rows-[0fr]" : "max-h-0"}"><nav>${item("Docs", item("Formatter", '<a href="/docs/formatter/options">Options</a>'))}</nav></div></header>`;
    const dom = new JSDOM('<meta name="tombi-archive-base" content="/v1"><div id="archive-header"></div>', { url: "https://example.com/v1/docs" });
    await runInNewContext(`(async () => {${source}\n})()`, {
      document: dom.window.document,
      location: dom.window.location,
      fetch: async () => ({
        ok: true,
        text: async () => shell,
        json: async () => [],
      }),
    });
    const document = dom.window.document;
    const mobile = document.querySelector(".header-dropdown");
    document.querySelector("#mobile-logo").click();
    assert.equal(mobile.hidden, false);
    if (grid) assert.equal(mobile.style.gridTemplateRows, "1fr");

    const [docs, formatter] = mobile.querySelectorAll("button");
    for (const button of [docs, formatter, docs, docs, formatter]) {
      const expanded = button.getAttribute("aria-expanded") !== "true";
      button.click();
      assert.equal(button.getAttribute("aria-expanded"), String(expanded));
      assert.equal(button.querySelector("svg").classList.contains("rotate-90"), expanded);
      const children = button.closest("a").nextElementSibling;
      if (grid) {
        assert.equal(children.style.gridTemplateRows, expanded ? "1fr" : "0fr");
        assert.equal(children.inert, !expanded);
      } else {
        assert.equal(children.style.maxHeight, expanded ? "none" : "0px");
      }
    }
    document.querySelector("#mobile-logo").click();
    assert.equal(mobile.hidden, true);
    dom.window.close();
  });
}
