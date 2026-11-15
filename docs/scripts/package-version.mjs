import {
  readdir,
  readFile,
  writeFile,
  mkdir,
  rm,
  stat,
} from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
const require = createRequire(
  new URL("./archive/package.json", import.meta.url),
);
const { JSDOM } = require("jsdom");
const Prism = require("prismjs");
require("prismjs/components/prism-bash.js");
require("prismjs/components/prism-toml.js");
require("prismjs/components/prism-json.js");
require("prismjs/components/prism-typescript.js");
require("prismjs/components/prism-python.js");
require("prismjs/components/prism-powershell.js");

const [input, output, version] = process.argv.slice(2);
if (
  !input ||
  !output ||
  !/^v\d+\.\d+\.\d+(?:[.-][\dA-Za-z.-]+)?$/.test(version || "")
) {
  throw new Error("Usage: node package-version.mjs INPUT OUTPUT vX.Y.Z");
}
if (resolve(input) === resolve(output))
  throw new Error("Output must differ from input");
const base = `/tombi/${version}`;
const remote = `https://raw.githubusercontent.com/tombi-toml/tombi/${version}/docs/public`;
const scripts = dirname(fileURLToPath(import.meta.url));
const pages = [];
async function files(directory, prefix = "") {
  const entries = await readdir(directory, { withFileTypes: true });
  return (
    await Promise.all(
      entries.map((entry) =>
        entry.isDirectory()
          ? files(join(directory, entry.name), `${prefix}${entry.name}/`)
          : `${prefix}${entry.name}`,
      ),
    )
  ).flat();
}
function clean(document) {
  document
    .querySelectorAll("script,link[rel=modulepreload]")
    .forEach((node) => node.remove());
  const walker = document.createTreeWalker(document, 128);
  const comments = [];
  while (walker.nextNode()) comments.push(walker.currentNode);
  comments.forEach((node) => node.remove());
  document.querySelectorAll("*").forEach((node) => {
    for (const attribute of [...node.attributes]) {
      if (attribute.name === "data-hk" || attribute.name === "link")
        node.removeAttribute(attribute.name);
      if (
        ["href", "src", "poster"].includes(attribute.name) &&
        attribute.value.startsWith(`${base}/`)
      ) {
        const path = attribute.value.slice(base.length);
        if (/\.(?:svg|png|jpg|jpeg|gif|webp|ico|sh|ps1)(?:$|[?#])/.test(path))
          node.setAttribute(attribute.name, `${remote}${path}`);
      }
    }
  });
}
function archiveTabs(document) {
  for (const container of document.querySelectorAll(
    "[data-archive-code-tabs]",
  )) {
    const tabs = JSON.parse(container.getAttribute("data-archive-code-tabs"));
    const original = container.querySelector(".code-block-wrapper");
    if (!original) throw new Error("Code tab has no rendered panel");
    const language = original
      .querySelector("code")
      .className.replace("language-", "");
    for (const tab of tabs) {
      const panel = original.cloneNode(true);
      panel.dataset.panel = tab.key;
      panel.hidden = !container.querySelector(
        `button[data-key="${tab.key}"] > div`,
      );
      const code = panel.querySelector("code");
      const panelLanguage = tab.language || language;
      code.classList.replace(
        `language-${language}`,
        `language-${panelLanguage}`,
      );
      panel
        .querySelector("pre")
        .classList.replace(`language-${language}`, `language-${panelLanguage}`);
      code.textContent = tab.command;
      container.append(panel);
    }
    original.remove();
    container.removeAttribute("data-archive-code-tabs");
    container.dataset.tabs = "code";
  }
  for (const container of document.querySelectorAll(
    "[data-archive-image-tabs]",
  )) {
    const tabs = JSON.parse(container.getAttribute("data-archive-image-tabs"));
    const image = container.querySelector("img");
    for (const tab of tabs) {
      const panel = image.cloneNode();
      panel.src = tab.src.startsWith("/") ? `${remote}${tab.src}` : tab.src;
      panel.alt = tab.alt;
      panel.dataset.panel = tab.key;
      panel.hidden = !container.querySelector(
        `button[data-key="${tab.key}"] > div`,
      );
      container.append(panel);
    }
    image.remove();
    container.removeAttribute("data-archive-image-tabs");
    container.dataset.tabs = "image";
  }
}
const tokenColors = [
  ["c", ["comment"]],
  ["s", ["string", "builtin"]],
  ["n", ["number"]],
  ["p", ["property", "variable", "parameter", "constant", "interpolation"]],
  ["k", ["keyword", "boolean", "important", "interpolation-punctuation"]],
  ["f", ["function"]],
  ["t", ["table", "class-name"]],
];
function highlightCodeBlocks(document) {
  for (const code of document.querySelectorAll("pre > code")) {
    const language = [...code.classList]
      .find((name) => name.startsWith("language-"))
      ?.slice("language-".length);
    const grammar = Prism.languages[language];
    if (!grammar) continue;
    code.innerHTML = Prism.highlight(code.textContent, grammar, language);
    // Release pages store short color classes to stay within the byte budget.
    for (const token of code.querySelectorAll("span.token")) {
      const color = tokenColors.find(([, names]) =>
        names.some((name) => token.classList.contains(name)),
      )?.[0];
      if (color) token.className = color;
      else token.replaceWith(...token.childNodes);
    }
  }
}
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
const sourceFiles = await files(input);
const css = sourceFiles.filter(
  (path) => path.endsWith(".css") && path.includes("/client-"),
);
if (css.length !== 1)
  throw new Error(`Expected one client stylesheet, found ${css.length}`);
await writeFile(
  join(output, "archive.css"),
  await readFile(join(input, css[0])),
);
let header;
const symbols = new Map();
function shareIcons(document) {
  for (const svg of document.querySelectorAll("svg")) {
    const content = svg.innerHTML;
    let id = symbols.get(content);
    if (!id) {
      id = `icon-${symbols.size}`;
      symbols.set(content, id);
    }
    svg.innerHTML = `<use href="${base}/icons.svg#${id}"></use>`;
  }
}
for (const path of sourceFiles.filter(
  (path) => path.endsWith(".html") && !path.startsWith("_"),
)) {
  const document = new JSDOM(await readFile(join(input, path), "utf8")).window
    .document;
  if (!document.querySelector(`link[href^="${base}/_build/"]`))
    throw new Error(`Unexpected build base URL in ${path}`);
  archiveTabs(document);
  highlightCodeBlocks(document);
  for (const button of document.querySelectorAll("button[data-key]")) {
    if (!button.closest("[data-tabs]") && !button.querySelector("div")) {
      button.disabled = true;
      button.title = "This tab is unavailable in this older archived version.";
    }
  }
  clean(document);
  shareIcons(document);
  if (!header) {
    const element = document.querySelector("header");
    if (element) {
      header = element.outerHTML;
    }
  }
  const article =
    document.querySelector("main main") || document.querySelector("main");
  if (!article) throw new Error(`Missing article in ${path}`);
  const route = path.replace(/(?:^|\/)index\.html$/, "").replace(/\.html$/, "");
  pages.push({ path, route, document, article });
}
if (!header || pages.length < 2)
  throw new Error("Missing prerendered documentation");
await writeFile(join(output, "shell.html"), header);
const index = JSON.parse(
  await readFile(join(scripts, "../doc-index.json"), "utf8"),
);
await writeFile(join(output, "navigation.json"), JSON.stringify(index));
const search = [];
for (const { path, route, document, article } of pages) {
  const isDocs = route === "docs" || route.startsWith("docs/");
  if (route === "playground") {
    article.innerHTML = `<div class="p-4 mdx-content"><h1>Playground</h1><p>The interactive playground runs the latest main build. Archived documentation remains pinned to ${version}.</p><p><a href="/tombi/main/playground">Open the main Playground</a></p></div>`;
  }
  if (isDocs)
    search.push({
      title: document.title.replace(/ \| Tombi$/, ""),
      url: `${base}/${route}`,
      content: article.textContent.replace(/\s+/g, " ").trim(),
    });
  const metadata = [...document.head.querySelectorAll("title,meta")]
    .map((node) => node.outerHTML)
    .join("");
  const html = `<!doctype html><html lang="en"><head>${metadata}<link rel="icon" href="${remote}/favicon.ico"><link rel="stylesheet" href="${base}/archive.css"><link rel="stylesheet" href="${base}/archive-ui.css"><script src="${base}/theme.js"></script><script type="module" src="${base}/archive.js"></script></head><body><div class="min-h-screen bg-gray-50 dark:bg-gray-800 text-gray-900 dark:text-gray-100"><div id="archive-header"></div><main class="flex-1 mt-20 pt-0"><div class="max-w-7xl mx-auto"><div class="${isDocs ? "flex w-full max-w-[100vw]" : ""}">${isDocs ? '<nav id="archive-sidebar" class="w-[250px] shrink-0 h-screen sticky top-0 overflow-y-auto p-4 bg-[--color-bg-secondary] border-r border-[--color-border] md:block hidden" aria-label="Documentation"></nav>' : ""}${article.outerHTML}</div></div></main></div></body></html>`;
  const destination = join(output, path);
  await mkdir(dirname(destination), { recursive: true });
  await writeFile(destination, html);
}
await writeFile(
  join(output, "404.html"),
  `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Page not found | Tombi ${version}</title><h1>Page not found</h1><p><a href="${base}/docs">Open ${version} documentation</a></p></html>`,
);
await writeFile(join(output, "search.json"), JSON.stringify(search));
await writeFile(
  join(output, "icons.svg"),
  `<svg xmlns="http://www.w3.org/2000/svg">${[...symbols].map(([content, id]) => `<g id="${id}">${content}</g>`).join("")}</svg>`,
);
for (const name of ["archive.js", "archive-ui.css", "theme.js"])
  await writeFile(
    join(output, name),
    await readFile(join(scripts, "archive", name)),
  );
const emitted = await files(output);
const sizes = await Promise.all(
  emitted.map(async (path) => ({
    path,
    bytes: (await stat(join(output, path))).size,
  })),
);
const bytes = sizes.reduce((total, entry) => total + entry.bytes, 0);
console.log(
  JSON.stringify(
    {
      version,
      files: sizes.length,
      pages: pages.length,
      bytes,
      limit: 1_000_000,
    },
    null,
    2,
  ),
);
if (bytes > 1_000_000)
  throw new Error(
    `Release documentation exceeds 1,000,000 raw bytes (${bytes})`,
  );
