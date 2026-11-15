import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { JSDOM } from "jsdom";

const source = await readFile(new URL("./archive.js", import.meta.url), "utf8");

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
