import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { JSDOM } from "jsdom";

const source = await readFile(new URL("./version-picker.js", import.meta.url), "utf8");

async function setup(pathname, hash, pages, { docsExpanded = false, existingFooter = false, hydrated = true, archived = false } = {}) {
  const dom = new JSDOM(
    '<header><div class="menu-toggle"></div><nav><button id="dark-mode-toggle" class="hidden sm:flex" aria-label="Toggle dark mode"><svg class="icon-tabler-sun-filled" data-hk="theme-icon"></svg></button><a href="https://x.com/tombi_toml" target="_blank" aria-label="X (Twitter)" class="hidden sm:block w-6 h-6"><svg data-hk="x-icon"></svg></a><a href="https://github.com/tombi-toml/tombi" target="_blank" aria-label="GitHub" class="hidden sm:block w-6 h-6"><svg data-hk="github-icon"></svg></a></nav><div class="header-dropdown"><nav><a>Home</a><a>Docs<button aria-label="Expand section"><svg data-hk="docs-chevron" viewBox="0 0 24 24"></svg></button></a><a>Playground</a></nav></div></header>',
  );
  dom.window.document.querySelector("#dark-mode-toggle").addEventListener("click", () => {
    dom.window.document.documentElement.classList.toggle("dark");
  });
  if (docsExpanded) {
    dom.window.document.querySelector(".header-dropdown nav a button svg").classList.add("rotate-90");
  }
  if (existingFooter) {
    const footer = dom.window.document.createElement("div");
    footer.className = "tombi-mobile-social-links";
    footer.innerHTML = '<button aria-label="Toggle dark mode"><svg class="icon-tabler-sun-filled"></svg></button>';
    dom.window.document.querySelector(".header-dropdown nav").append(footer);
  }
  if (archived) {
    const meta = dom.window.document.createElement("meta");
    meta.name = "tombi-archive-base";
    dom.window.document.head.append(meta);
  }
  if (hydrated) dom.window.document.documentElement.dataset.tombiHydrated = "true";
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
  const panel = mobile.querySelector(".tombi-version-mobile-panel");
  Object.defineProperty(panel, "scrollHeight", { value: 160 });
  assert.equal(panel.style.maxHeight, "0px");
  assert.equal(panel.inert, true);
  mobileTrigger.click();
  assert.ok(mobileTrigger.querySelector("svg").classList.contains("rotate-90"));
  assert.equal(panel.style.maxHeight, "160px");
  assert.equal(panel.inert, false);
  assert.equal(mobile.children[1].textContent, "Home");
  assert.equal(mobile.querySelector(".tombi-version-back"), null);
  mobileTrigger.click();
  assert.equal(panel.style.maxHeight, "0px");
  assert.equal(panel.inert, true);
  assert.equal(mobileTrigger.getAttribute("aria-expanded"), "false");
  mobileTrigger.click();
  mobile.querySelectorAll(".tombi-version-mobile-panel .tombi-version-option")[1].click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(page.getAssigned(), path);
});

test("mobile version arrow starts closed independently of an open Docs section", async () => {
  const page = await setup("/tombi/main/docs/", "", {}, { docsExpanded: true });
  const versionArrow = page.document.querySelector(".tombi-version-mobile-trigger svg");
  const docsArrow = page.document.querySelector(".header-dropdown nav a button svg");
  assert.ok(docsArrow.classList.contains("rotate-90"));
  assert.ok(!versionArrow.classList.contains("rotate-90"));
  assert.equal(versionArrow.hasAttribute("data-hk"), false);
  docsArrow.classList.remove("rotate-90");
  assert.ok(!versionArrow.classList.contains("rotate-90"));
});

test("mobile menu ends with visible theme and social controls", async () => {
  const page = await setup("/tombi/main/docs/", "", {});
  const nav = page.document.querySelector(".header-dropdown nav");
  const footer = nav.lastElementChild;
  assert.equal(footer.className, "tombi-mobile-social-links");
  assert.equal(footer.previousElementSibling.textContent, "Playground");
  const themeButton = footer.querySelector("button");
  assert.equal(themeButton.getAttribute("aria-label"), "Toggle dark mode");
  assert.ok(themeButton.classList.contains("tombi-mobile-social-link"));
  assert.ok(!themeButton.classList.contains("hidden"));
  assert.equal(themeButton.id, "");
  assert.equal(themeButton.querySelector("[data-hk]"), null);
  assert.ok(themeButton.querySelector("svg").classList.contains("icon-tabler-sun-filled"));
  themeButton.click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(page.document.documentElement.classList.contains("dark"));
  assert.ok(themeButton.querySelector("svg").classList.contains("icon-tabler-moon-filled"));
  themeButton.click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(!page.document.documentElement.classList.contains("dark"));
  assert.ok(themeButton.querySelector("svg").classList.contains("icon-tabler-sun-filled"));
  const links = [...footer.querySelectorAll("a")];
  assert.deepEqual(links.map((link) => link.getAttribute("aria-label")), ["X (Twitter)", "GitHub"]);
  assert.deepEqual(links.map((link) => link.href), [
    "https://x.com/tombi_toml",
    "https://github.com/tombi-toml/tombi",
  ]);
  for (const link of links) {
    assert.ok(link.classList.contains("tombi-mobile-social-link"));
    assert.ok(!link.classList.contains("hidden"));
    assert.ok(!link.classList.contains("sm:block"));
    assert.equal(link.querySelector("[data-hk]"), null);
  }
});

test("main menu keeps its source-rendered social row", async () => {
  const page = await setup("/tombi/main/docs/", "", {}, { existingFooter: true });
  assert.equal(page.document.querySelectorAll(".tombi-mobile-social-links").length, 1);
});

test("archived source-rendered mobile theme button remains interactive", async () => {
  const page = await setup("/tombi/v1.12.0/docs/", "", {}, { existingFooter: true, archived: true });
  const themeButton = page.document.querySelector('.tombi-mobile-social-links button[aria-label="Toggle dark mode"]');
  assert.equal(page.document.querySelectorAll(".tombi-mobile-social-links").length, 1);
  themeButton.click();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(page.document.documentElement.classList.contains("dark"));
  assert.ok(themeButton.querySelector("svg").classList.contains("icon-tabler-moon-filled"));
});

test("main menu is not changed until hydration completes", async () => {
  const page = await setup("/tombi/main/docs/", "", {}, { hydrated: false });
  assert.equal(page.document.querySelector(".tombi-version-mobile"), null);
  page.document.documentElement.dataset.tombiHydrated = "true";
  page.document.dispatchEvent(new page.document.defaultView.Event("tombi:hydrated"));
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.ok(page.document.querySelector(".tombi-version-mobile"));
});
