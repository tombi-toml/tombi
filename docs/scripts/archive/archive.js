const base = document.querySelector('meta[name="tombi-archive-base"]').content;
const normalize = (path) => path.replace(/\/$/, "");
const currentPath = normalize(location.pathname);
const escape = (value) =>
  value.replace(
    /[&<>"']/g,
    (character) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        character
      ],
  );
const localPath = (path) => `${base}${path === "/" ? "" : path}`;
const link = (item) =>
  `<a href="${escape(localPath(item.path))}"${currentPath === normalize(localPath(item.path)) ? ' aria-current="page"' : ""} class="text-[--color-text] no-underline block pl-2 mr-2 py-1 hover:text-[--color-primary] flex-grow">${escape(item.title)}</a>`;
function tree(items, level = 0) {
  return items
    .map((item) => {
      const active =
        currentPath === normalize(localPath(item.path)) ||
        currentPath.startsWith(`${normalize(localPath(item.path))}/`);
      const child = item.children?.length;
      return `<div class="my-4" style="padding-left:${level * 8}px">${child ? `<details${active ? " open" : ""}><summary>${link(item)}</summary><div class="ml-2">${tree(item.children, level + 1)}</div></details>` : link(item)}</div>`;
    })
    .join("");
}
const [header, navigation] = await Promise.all([
  fetch(`${base}/shell.html`).then((response) => {
    if (!response.ok) throw new Error("Header unavailable");
    return response.text();
  }),
  fetch(`${base}/navigation.json`).then((response) => response.json()),
]);
document.querySelector("#archive-header").innerHTML = header;
const mobileTitle = document.querySelector("header h1");
if (mobileTitle)
  mobileTitle.textContent =
    document.querySelector("main h1")?.textContent || "Tombi";
const sidebar = document.querySelector("#archive-sidebar");
if (sidebar) sidebar.innerHTML = tree(navigation);
const mobile = document.querySelector(".header-dropdown");
const sectionPaths = ["/docs", "/playground"].map(localPath);
for (const anchor of document.querySelectorAll("header nav a")) {
  if (mobile.contains(anchor) || anchor.origin !== location.origin) continue;
  const path = normalize(anchor.pathname);
  if (!sectionPaths.includes(path)) continue;
  anchor.setAttribute("data-archive-section", "");
  anchor.removeAttribute("aria-current");
  const active = currentPath === path || currentPath.startsWith(`${path}/`);
  anchor.classList.toggle("header-section-tab-active", active);
  if (active) {
    anchor.setAttribute("aria-current", "location");
  }
}
mobile.id = "archive-mobile";
mobile.hidden = true;
mobile.classList.remove("max-h-0");
mobile.style.maxHeight = "calc(100vh - 5rem)";
if (mobile.classList.contains("grid")) mobile.style.gridTemplateRows = "1fr";
mobile.setAttribute("aria-label", "Site navigation");
for (const control of mobile.querySelectorAll("a,button")) control.tabIndex = 0;
const mobileSections = new Map();
const setMobileExpanded = (button, expanded) => {
  button.setAttribute("aria-expanded", String(expanded));
  button.setAttribute(
    "aria-label",
    expanded ? "Collapse section" : "Expand section",
  );
  button.querySelector("svg")?.classList.toggle("rotate-90", expanded);
  const panel = mobileSections.get(button);
  if (panel.classList.contains("grid")) {
    panel.style.removeProperty("height");
    panel.style.gridTemplateRows = expanded ? "1fr" : "0fr";
    panel.inert = !expanded;
  } else if (panel.style.height) {
    panel.style.height = expanded ? "auto" : "0px";
    panel.inert = !expanded;
  } else {
    panel.style.maxHeight = expanded ? "none" : "0px";
  }
};
let currentMobileLink;
for (const anchor of mobile.querySelectorAll("a")) {
  anchor.removeAttribute("aria-current");
  if (
    anchor.origin === location.origin &&
    normalize(anchor.pathname) === currentPath
  ) {
    currentMobileLink?.removeAttribute("aria-current");
    anchor.setAttribute("aria-current", "page");
    currentMobileLink = anchor;
  }
}
for (const button of mobile.querySelectorAll("a button[aria-expanded]")) {
  const panel = button.closest("a").nextElementSibling;
  if (!panel) continue;
  mobileSections.set(button, panel);
  setMobileExpanded(button, panel.contains(currentMobileLink));
}
mobile.addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (!mobileSections.has(button)) return;
  event.preventDefault();
  setMobileExpanded(button, button.getAttribute("aria-expanded") !== "true");
});
const mobileLogo = document.querySelector("#mobile-logo");
mobileLogo.setAttribute("aria-expanded", "false");
mobileLogo.setAttribute("aria-label", "Toggle navigation");
mobileLogo.addEventListener("click", (event) => {
  event.preventDefault();
  mobile.hidden = !mobile.hidden;
  mobileLogo.setAttribute("aria-expanded", String(!mobile.hidden));
});
document.querySelector("#dark-mode-toggle")?.addEventListener("click", () => {
  const dark = document.documentElement.classList.toggle("dark");
  document
    .querySelector(".archive-playground-frame")
    ?.contentDocument?.documentElement?.classList.toggle("dark", dark);
  try {
    localStorage.setItem("theme", dark ? "dark" : "light");
  } catch {}
});
const playgroundFrame = document.querySelector(".archive-playground-frame");
if (playgroundFrame) {
  const normalizeEmbeddedSpacing = () => {
    const document = playgroundFrame.contentDocument;
    document?.body?.style.setProperty("margin", "0");
    document
      ?.querySelector("main")
      ?.style.setProperty("padding-inline", "0");
  };
  playgroundFrame.addEventListener("load", normalizeEmbeddedSpacing);
  normalizeEmbeddedSpacing();
}
const input = document.querySelector("header input");
input.tabIndex = 0;
input.setAttribute("aria-label", "Search documentation");
const results = document.createElement("div");
results.id = "archive-results";
results.hidden = true;
results.setAttribute("aria-label", "Search results");
document.querySelector("header").append(results);
let searchIndex;
let searchRequest = 0;
input.addEventListener("input", async () => {
  const request = ++searchRequest;
  const query = input.value.trim().toLowerCase();
  if (!query) {
    results.hidden = true;
    return;
  }
  try {
    searchIndex ||= fetch(`${base}/search.json`).then((response) => {
      if (!response.ok) throw new Error("Search unavailable");
      return response.json();
    });
    const documents = await searchIndex;
    if (request !== searchRequest) return;
    const words = query.split(/\s+/);
    const matches = documents
      .filter((item) =>
        words.every((word) =>
          `${item.title} ${item.content}`.toLowerCase().includes(word),
        ),
      )
      .sort(
        (a, b) =>
          Number(b.title.toLowerCase().includes(query)) -
          Number(a.title.toLowerCase().includes(query)),
      )
      .slice(0, 12);
    results.innerHTML = matches.length
      ? matches
          .map((item) => {
            const position = Math.max(
              0,
              item.content.toLowerCase().indexOf(query) - 50,
            );
            return `<a href="${escape(item.url)}"><strong>${escape(item.title)}</strong><p>${escape(item.content.slice(position, position + 180))}</p></a>`;
          })
          .join("")
      : "No results found.";
    results.hidden = false;
  } catch {
    searchIndex = undefined;
    if (request !== searchRequest) return;
    results.textContent = "Search could not be loaded. Please try again.";
    results.hidden = false;
  }
});
const searchHeader = document.querySelector("header");
const searchButton = document.querySelector('button[aria-label="Search"]');
const setSearchOpen = (open) => {
  searchHeader.classList.toggle("archive-search-open", open);
  searchButton?.setAttribute("aria-label", open ? "Close Search" : "Search");
  for (const [index, icon] of Array.from(searchButton?.children || []).entries()) {
    const visible = index === 0 ? open : !open;
    icon.classList.toggle("opacity-100", visible);
    icon.classList.toggle("opacity-0", !visible);
    icon.classList.toggle("rotate-0", visible);
    icon.classList.toggle(index === 0 ? "-rotate-90" : "rotate-90", !visible);
  }
  if (open) input.focus();
  else {
    searchRequest++;
    results.hidden = true;
    input.blur();
  }
};
searchButton?.addEventListener("click", () => {
  setSearchOpen(!searchHeader.classList.contains("archive-search-open"));
});
document.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    setSearchOpen(true);
  }
  if (event.key === "Escape") {
    setSearchOpen(false);
    mobile.hidden = true;
    mobileLogo.setAttribute("aria-expanded", "false");
  }
});
document.addEventListener("click", async (event) => {
  if (!event.target.closest("header")) {
    results.hidden = true;
    mobile.hidden = true;
    mobileLogo.setAttribute("aria-expanded", "false");
  }
  const button = event.target.closest("button");
  if (!button) return;
  if (button.closest(".copy-button")) {
    const code = button
      .closest(".code-block-wrapper")
      .querySelector("code").textContent;
    try {
      await navigator.clipboard.writeText(code);
      const label = button.getAttribute("aria-label");
      button.setAttribute("aria-label", "Copied");
      button.style.color = "#4ade80";
      setTimeout(() => {
        button.setAttribute("aria-label", label);
        button.style.color = "";
      }, 1000);
    } catch {
      button.setAttribute("aria-label", "Copy failed");
    }
  }
  const group = button.closest("[data-tabs]");
  if (group && button.dataset.key) {
    const groups =
      group.dataset.tabs === "code"
        ? document.querySelectorAll('[data-tabs="code"]')
        : [group];
    for (const target of groups) {
      if (
        !Array.from(target.querySelectorAll("button[data-key]")).some(
          (tab) => tab.dataset.key === button.dataset.key,
        )
      )
        continue;
      for (const panel of target.querySelectorAll("[data-panel]"))
        panel.hidden = panel.dataset.panel !== button.dataset.key;
      for (const tab of target.querySelectorAll("button[data-key]")) {
        const selected = tab.dataset.key === button.dataset.key;
        tab.setAttribute("aria-selected", String(selected));
        const indicator = tab.querySelector("div");
        if (indicator) indicator.hidden = !selected;
        else if (selected) {
          const marker = document.createElement("div");
          marker.className =
            "absolute bottom-0 left-0 w-full h-1 bg-tombi-700 dark:bg-yellow";
          tab.append(marker);
        }
      }
    }
  }
});

for (const heading of document.querySelectorAll(".heading-with-anchor[id]")) {
  const anchor = document.createElement("a");
  anchor.className = "anchor-link";
  anchor.href = `#${heading.id}`;
  anchor.setAttribute("aria-label", "Copy section link");
  anchor.innerHTML = `<svg fill="currentColor" stroke-width="0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="m7.775 3.275 1.25-1.25a3.5 3.5 0 1 1 4.95 4.95l-2.5 2.5a3.5 3.5 0 0 1-4.95 0 .751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018 1.998 1.998 0 0 0 2.83 0l2.5-2.5a2.002 2.002 0 0 0-2.83-2.83l-1.25 1.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042Zm-4.69 9.64a1.998 1.998 0 0 0 2.83 0l1.25-1.25a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042l-1.25 1.25a3.5 3.5 0 1 1-4.95-4.95l2.5-2.5a3.5 3.5 0 0 1 4.95 0 .751.751 0 0 1-.018 1.042.751.751 0 0 1-1.042.018 1.998 1.998 0 0 0-2.83 0l-2.5 2.5a1.998 1.998 0 0 0 0 2.83Z"></path></svg>`;
  anchor.addEventListener("click", async (event) => {
    event.preventDefault();
    const url = new URL(location.href);
    url.hash = heading.id;
    try {
      await navigator.clipboard.writeText(url.href);
      anchor.textContent = "✓";
      setTimeout(() => {
        anchor.innerHTML = `<svg fill="currentColor" stroke-width="0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" height="1em" width="1em" style="overflow: visible; color: currentcolor;"><path d="m7.775 3.275 1.25-1.25a3.5 3.5 0 1 1 4.95 4.95l-2.5 2.5a3.5 3.5 0 0 1-4.95 0 .751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018 1.998 1.998 0 0 0 2.83 0l2.5-2.5a2.002 2.002 0 0 0-2.83-2.83l-1.25 1.25a.751.751 0 0 1-1.042-.018.751.751 0 0 1-.018-1.042Zm-4.69 9.64a1.998 1.998 0 0 0 2.83 0l1.25-1.25a.751.751 0 0 1 1.042.018.751.751 0 0 1 .018 1.042l-1.25 1.25a3.5 3.5 0 1 1-4.95-4.95l2.5-2.5a3.5 3.5 0 0 1 4.95 0 .751.751 0 0 1-.018 1.042.751.751 0 0 1-1.042.018 1.998 1.998 0 0 0-2.83 0l-2.5 2.5a1.998 1.998 0 0 0 0 2.83Z"></path></svg>`;
      }, 500);
    } catch {
      location.hash = heading.id;
    }
  });
  heading.append(anchor);
}
