import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { JSDOM } from "jsdom";

const source = await readFile(new URL("./archive.js", import.meta.url), "utf8");
const styles = await readFile(new URL("./archive-ui.css", import.meta.url), "utf8");

test("archived mobile search toggles while keyboard shortcuts open and close it", async () => {
  const shell = '<header><a id="mobile-logo"></a><input><button aria-label="Search"><div class="absolute transition-all duration-300 opacity-0 -rotate-90"><svg></svg></div><div class="transition-all duration-300 opacity-100 rotate-0"><svg></svg></div></button><div class="header-dropdown"></div></header>';
  const dom = new JSDOM('<meta name="tombi-archive-base" content="/v1"><div id="archive-header"></div>', { url: "https://example.com/v1/docs" });
  try {
    const document = dom.window.document;
    await runInNewContext(`(async () => {${source}\n})()`, {
      document,
      location: dom.window.location,
      fetch: async () => ({
        ok: true,
        text: async () => shell,
        json: async () => [],
      }),
    });
    const header = document.querySelector("header");
    const button = header.querySelector("button");
    const input = header.querySelector("input");
    const results = document.querySelector("#archive-results");
    const assertSearch = (open) => {
      assert.equal(header.classList.contains("archive-search-open"), open);
      assert.equal(button.getAttribute("aria-label"), open ? "Close Search" : "Search");
      for (const [index, icon] of Array.from(button.children).entries()) {
        const visible = index === 0 ? open : !open;
        assert.equal(icon.classList.contains("opacity-100"), visible);
        assert.equal(icon.classList.contains("opacity-0"), !visible);
        assert.equal(icon.classList.contains("rotate-0"), visible);
        assert.equal(icon.classList.contains(index === 0 ? "-rotate-90" : "rotate-90"), !visible);
      }
      assert.equal(document.activeElement === input, open);
      if (!open) assert.equal(results.hidden, true);
    };

    assertSearch(false);
    button.click();
    assert.equal(header.classList.contains("archive-search-open"), true);
    assert.equal(document.activeElement, input);
    input.value = "formatter";
    input.dispatchEvent(new dom.window.Event("input"));
    await new Promise(setImmediate);
    assert.equal(results.hidden, false);
    button.click();
    assertSearch(false);

    for (const modifier of ["ctrlKey", "metaKey"]) {
      for (let press = 0; press < 2; press++) {
        const event = new dom.window.KeyboardEvent("keydown", { key: "k", [modifier]: true, cancelable: true });
        document.dispatchEvent(event);
        assert.equal(event.defaultPrevented, true);
        assertSearch(true);
      }
      document.dispatchEvent(new dom.window.KeyboardEvent("keydown", { key: "Escape" }));
      assertSearch(false);
    }
  } finally {
    dom.window.close();
  }
});

for (const closeWith of ["button", "Escape"]) {
  for (const responseState of ["loaded", "failed"]) {
    test(`archived search stays closed after ${closeWith} while pending search ${responseState}`, async () => {
      const shell = '<header><a id="mobile-logo"></a><input><button aria-label="Search"></button><div class="header-dropdown"></div></header>';
      const dom = new JSDOM('<meta name="tombi-archive-base" content="/v1"><div id="archive-header"></div>', { url: "https://example.com/v1/docs" });
      try {
        let finishSearch;
        const pendingSearch = new Promise((resolve) => { finishSearch = resolve; });
        const document = dom.window.document;
        await runInNewContext(`(async () => {${source}\n})()`, {
          document,
          location: dom.window.location,
          fetch: async (url) => url.endsWith("/search.json") ? pendingSearch : {
            ok: true,
            text: async () => shell,
            json: async () => [],
          },
        });
        const header = document.querySelector("header");
        const button = header.querySelector("button");
        const input = header.querySelector("input");
        const results = document.querySelector("#archive-results");
        button.click();
        input.value = "formatter";
        input.dispatchEvent(new dom.window.Event("input"));
        if (closeWith === "button") button.click();
        else document.dispatchEvent(new dom.window.KeyboardEvent("keydown", { key: "Escape" }));
        assert.equal(header.classList.contains("archive-search-open"), false);
        assert.equal(results.hidden, true);

        finishSearch({ ok: responseState === "loaded", json: async () => [] });
        await new Promise(setImmediate);
        assert.equal(results.hidden, true);
        assert.equal(header.classList.contains("archive-search-open"), false);
      } finally {
        dom.window.close();
      }
    });
  }
}

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
    if (frameState === "loaded") frame.contentDocument.body.innerHTML = "<main></main>";
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

    if (frameState === "loaded") {
      const frameMain = frame.contentDocument.querySelector("main");
      assert.equal(frame.contentDocument.body.style.margin, "0px");
      assert.equal(frameMain.style.paddingInline, "0");
      frame.contentDocument.body.style.margin = "8px";
      frameMain.style.paddingInline = "1em";
      frame.dispatchEvent(new dom.window.Event("load"));
      assert.equal(frame.contentDocument.body.style.margin, "0px");
      assert.equal(frameMain.style.paddingInline, "0");
    }

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
    const item = (name, path, contents) => `<div><a href="${base}${path}"><span>${name}</span><button aria-expanded="false"><svg></svg></button></a>${panel(contents)}</div>`;
    const base = "/tombi/v1.11.2";
    const currentPath = `${base}/docs/language-server/diagnostics`;
    const shell = `<header><a id="mobile-logo"></a><input><div class="header-dropdown ${grid ? "grid grid-rows-[0fr]" : "max-h-0"}"><nav>${item("Docs", "/docs", item("Language Server", "/docs/language-server", `<a href="${currentPath}"><span>Diagnostics</span></a>`) + item("CLI", "/docs/cli", `<a href="${base}/docs/cli/diagnostics-output" aria-current="page"><span>Diagnostics Output</span></a>`))}</nav><button id="dark-mode-toggle" aria-label="Toggle theme"><svg></svg></button><button id="other-menu" aria-expanded="false" aria-label="Other menu"><svg></svg></button><a href="/other"><button id="no-panel" aria-expanded="false" aria-label="No panel"><svg></svg></button></a></div></header>`;
    const dom = new JSDOM(`<style>${styles}</style><meta name="tombi-archive-base" content="${base}"><div id="archive-header"></div>`, { url: `https://example.com${currentPath}/` });
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

    const [docs, languageServer, cli] = mobile.querySelectorAll("nav button");
    const assertExpanded = (button, expanded) => {
      assert.equal(button.getAttribute("aria-expanded"), String(expanded));
      assert.equal(button.getAttribute("aria-label"), expanded ? "Collapse section" : "Expand section");
      assert.equal(button.querySelector("svg").classList.contains("rotate-90"), expanded);
      const children = button.closest("a").nextElementSibling;
      if (grid) {
        assert.equal(children.style.gridTemplateRows, expanded ? "1fr" : "0fr");
        assert.equal(children.inert, !expanded);
      } else {
        assert.equal(children.style.maxHeight, expanded ? "none" : "0px");
      }
    };
    assert.deepEqual(Array.from(mobile.querySelectorAll('[aria-current="page"]'), (anchor) => anchor.getAttribute("href")), [currentPath]);
    assertExpanded(docs, true);
    assertExpanded(languageServer, true);
    assertExpanded(cli, false);
    const currentLink = mobile.querySelector('[aria-current="page"]');
    assert.equal(dom.window.getComputedStyle(currentLink).color, "rgb(250, 204, 21)");
    document.documentElement.classList.add("dark");
    assert.equal(dom.window.getComputedStyle(currentLink).color, "rgb(250, 204, 21)");
    for (const [id, label, expanded] of [
      ["dark-mode-toggle", "Toggle theme", null],
      ["other-menu", "Other menu", "false"],
      ["no-panel", "No panel", "false"],
    ]) {
      const button = document.getElementById(id);
      assert.equal(button.getAttribute("aria-label"), label);
      assert.equal(button.getAttribute("aria-expanded"), expanded);
      const attributes = button.outerHTML;
      const event = new dom.window.MouseEvent("click", { bubbles: true, cancelable: true });
      button.dispatchEvent(event);
      assert.equal(event.defaultPrevented, false);
      assert.equal(button.outerHTML, attributes);
    }
    assert.equal(document.documentElement.classList.contains("dark"), false);
    for (const button of [docs, languageServer, docs, docs, languageServer, cli]) {
      const expanded = button.getAttribute("aria-expanded") !== "true";
      button.click();
      assertExpanded(button, expanded);
    }
    document.querySelector("#mobile-logo").click();
    assert.equal(mobile.hidden, true);
    document.querySelector("#mobile-logo").click();
    assert.equal(mobile.hidden, false);
    assertExpanded(docs, false);
    assertExpanded(languageServer, true);
    assertExpanded(cli, true);
    dom.window.close();
  });
}

test("archived mobile menu selects Overview without also selecting Docs", async () => {
  const base = "/tombi/v1.11.2";
  const shell = `<header><a id="mobile-logo"></a><nav><a href="${base}/docs">Docs</a></nav><input><div class="header-dropdown"><a href="${base}/docs"><span>Docs</span><button aria-expanded="false"><svg></svg></button></a><div><a href="${base}/docs"><span>Overview</span></a></div></div></header>`;
  const dom = new JSDOM(`<style>${styles}</style><meta name="tombi-archive-base" content="${base}"><div id="archive-header"></div>`, { url: `https://example.com${base}/docs/` });
  try {
    await runInNewContext(`(async () => {${source}\n})()`, {
      document: dom.window.document,
      location: dom.window.location,
      fetch: async () => ({
        ok: true,
        text: async () => shell,
        json: async () => [],
      }),
    });
    const mobile = dom.window.document.querySelector("#archive-mobile");
    assert.deepEqual(
      Array.from(mobile.querySelectorAll('[aria-current="page"]'), (anchor) => anchor.textContent),
      ["Overview"],
    );
    assert.equal(dom.window.getComputedStyle(mobile.querySelector("a")).color, "rgb(255, 255, 255)");
  } finally {
    dom.window.close();
  }
});

for (const [path, section] of [
  ["/", undefined],
  ["/docs", "Docs"],
  ["/docs/language-server/diagnostics/", "Docs"],
  ["/playground/", "Playground"],
  ["/playground/example/", "Playground"],
  ["/docs-other", undefined],
  ["/playground-other", undefined],
]) {
  test(`archived header highlights the current section for ${path}`, async () => {
    const base = "/tombi/v1.11.2";
    const shell = `<header><nav><a id="mobile-logo"></a><a href="${base}/docs" class="header-section-tab-active" aria-current="page">Docs</a><a href="${base}/playground">Playground</a><a href="https://other.example${base}/docs">External</a><input></nav><div class="header-dropdown"><a href="${base}/"><span class="text-yellow">Home</span></a><a href="${base}/docs"><span>Docs</span></a><a href="${base}/playground"><span>Playground</span></a></div></header>`;
    const dom = new JSDOM(`<style>.text-yellow { color: #facc15; }${styles}</style><meta name="tombi-archive-base" content="${base}"><div id="archive-header"></div>`, { url: `https://example.com${base}${path}` });
    try {
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
      const active = Array.from(document.querySelectorAll('header nav a[aria-current="location"]'));
      assert.deepEqual(active.map((anchor) => anchor.textContent), section ? [section] : []);
      const sections = Array.from(document.querySelectorAll('header nav a[data-archive-section]'));
      assert.deepEqual(sections.map((anchor) => anchor.textContent), ["Docs", "Playground"]);
      assert.deepEqual(
        sections
          .filter((anchor) => anchor.classList.contains("header-section-tab-active"))
          .map((anchor) => anchor.textContent),
        section ? [section] : [],
      );
      for (const dark of [false, true]) {
        document.documentElement.classList.toggle("dark", dark);
        for (const anchor of document.querySelectorAll("#archive-mobile a")) {
          const current = anchor.getAttribute("aria-current") === "page";
          const expected = current ? "rgb(250, 204, 21)" : "rgb(255, 255, 255)";
          assert.equal(dom.window.getComputedStyle(anchor).color, expected);
          assert.equal(dom.window.getComputedStyle(anchor.querySelector("span")).color, expected);
        }
      }
    } finally {
      dom.window.close();
    }
  });
}
