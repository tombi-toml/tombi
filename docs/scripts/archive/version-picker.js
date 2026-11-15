(() => {
  const match = location.pathname.match(/^\/tombi\/(main|v\d+\.\d+\.\d+(?:[.-][\dA-Za-z.-]+)?)(?:\/|$)/);
  if (!match) return;

  const currentVersion = match[1];
  const validVersion = /^(?:main|v\d+\.\d+\.\d+(?:[.-][\dA-Za-z.-]+)?)$/;

  function destination(version, route) {
    return `/tombi/${version}${route === "/" ? "/" : `${route}/`}`;
  }

  async function switchVersion(version) {
    if (version === currentVersion) return;
    const currentRoute =
      location.pathname.slice(`/tombi/${currentVersion}`.length).replace(/\/$/, "") || "/";
    let route = currentRoute;
    while (true) {
      const path = destination(version, route);
      try {
        const response = await fetch(path);
        if (response.ok && response.headers.get("content-type")?.includes("text/html")) {
          let hash = "";
          if (route === currentRoute && location.hash) {
            const document = new DOMParser().parseFromString(await response.text(), "text/html");
            let id;
            try {
              id = decodeURIComponent(location.hash.slice(1));
            } catch {
              id = location.hash.slice(1);
            }
            if (document.getElementById(id)) hash = location.hash;
          }
          location.assign(`${path}${location.search}${hash}`);
          return;
        }
      } catch {
        // Continue to the parent page if a route cannot be loaded.
      }
      if (route === "/") break;
      route = route.slice(0, route.lastIndexOf("/")) || "/";
    }
    location.assign(destination(version, "/"));
  }

  function option(version) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "tombi-version-option";
    button.textContent = version;
    if (version === currentVersion) button.setAttribute("aria-current", "page");
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      switchVersion(version);
    });
    return button;
  }

  async function versions() {
    try {
      const response = await fetch("/tombi/versions.json");
      if (!response.ok) throw new Error("Version list unavailable");
      const list = await response.json();
      if (!Array.isArray(list)) throw new Error("Invalid version list");
      return [...new Set(["main", ...list.filter((item) => typeof item === "string" && validVersion.test(item)), currentVersion])];
    } catch {
      return [...new Set(["main", currentVersion])];
    }
  }

  async function mount(header) {
    if (header.dataset.versionPicker) return;
    header.dataset.versionPicker = "true";
    const logo = header.querySelector(".menu-toggle");
    const mobileNav = header.querySelector(".header-dropdown nav");
    if (!logo || !mobileNav) return;

    const list = await versions();
    const desktop = document.createElement("div");
    desktop.className = "tombi-version-desktop";
    const trigger = document.createElement("button");
    trigger.type = "button";
    trigger.className = "tombi-version-trigger";
    trigger.setAttribute("aria-label", `Version ${currentVersion}. Choose version`);
    trigger.setAttribute("aria-expanded", "false");
    trigger.textContent = currentVersion;
    const menu = document.createElement("div");
    menu.className = "tombi-version-menu";
    menu.hidden = true;
    menu.setAttribute("aria-label", "Versions");
    list.forEach((version) => menu.append(option(version)));
    trigger.addEventListener("click", () => {
      menu.hidden = !menu.hidden;
      trigger.setAttribute("aria-expanded", String(!menu.hidden));
    });
    document.addEventListener("click", (event) => {
      if (!desktop.contains(event.target)) {
        menu.hidden = true;
        trigger.setAttribute("aria-expanded", "false");
      }
    });
    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        menu.hidden = true;
        trigger.setAttribute("aria-expanded", "false");
      }
    });
    desktop.append(trigger, menu);
    logo.after(desktop);

    const mobile = document.createElement("div");
    mobile.className = "tombi-version-mobile";
    const mobileTrigger = document.createElement("button");
    mobileTrigger.type = "button";
    mobileTrigger.className = "tombi-version-mobile-trigger";
    mobileTrigger.setAttribute("aria-expanded", "false");
    mobileTrigger.setAttribute("aria-label", `Version ${currentVersion}. Choose version`);
    mobileTrigger.innerHTML = `<span>${currentVersion}</span>`;
    const chevron = mobileNav.querySelector("a button svg")?.cloneNode(true);
    if (chevron) {
      chevron.removeAttribute("data-hk");
      chevron.classList.remove("rotate-90");
      chevron.setAttribute("aria-hidden", "true");
      const chevronBox = document.createElement("span");
      chevronBox.className = "tombi-version-mobile-chevron";
      chevronBox.append(chevron);
      mobileTrigger.append(chevronBox);
    }
    const panel = document.createElement("div");
    panel.className = "tombi-version-mobile-panel";
    panel.style.maxHeight = "0px";
    panel.inert = true;
    list.forEach((version) => panel.append(option(version)));
    mobileTrigger.addEventListener("click", (event) => {
      event.stopPropagation();
      const expanded = mobileTrigger.getAttribute("aria-expanded") !== "true";
      mobileTrigger.setAttribute("aria-expanded", String(expanded));
      panel.inert = !expanded;
      panel.style.maxHeight = expanded ? `${panel.scrollHeight}px` : "0px";
      chevron?.classList.toggle("rotate-90", expanded);
    });
    mobile.append(mobileTrigger, panel);
    mobileNav.prepend(mobile);

    const themeButton = header.querySelector("#dark-mode-toggle");
    const socialLinks = [...header.querySelectorAll('nav a[target="_blank"][aria-label]')];
    const wireArchiveTheme = (mobileTheme) => {
      if (!themeButton || !mobileTheme) return;
      const sunIcon = mobileTheme.querySelector("svg");
      if (!sunIcon) return;
      const moonIcon = sunIcon.cloneNode(false);
      moonIcon.classList.replace("icon-tabler-sun-filled", "icon-tabler-moon-filled");
      moonIcon.innerHTML = '<path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 1.992a10 10 0 1 0 9.236 13.838c.341 -.82 -.476 -1.644 -1.298 -1.31a6.5 6.5 0 0 1 -6.864 -10.787l.077 -.08c.551 -.63 .113 -1.653 -.758 -1.653h-.266l-.068 -.006l-.06 -.002z" stroke-width="0" fill="currentColor"/>';
      const syncIcon = () => {
        mobileTheme.replaceChildren(document.documentElement.classList.contains("dark") ? moonIcon : sunIcon);
      };
      new MutationObserver(syncIcon).observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
      syncIcon();
      mobileTheme.addEventListener("click", (event) => {
        event.stopPropagation();
        themeButton.click();
      });
    };
    const existingFooter = mobileNav.querySelector(".tombi-mobile-social-links");
    if (existingFooter && document.querySelector('meta[name="tombi-archive-base"]')) {
      wireArchiveTheme(existingFooter.querySelector('button[aria-label="Toggle dark mode"]'));
    }
    if ((themeButton || socialLinks.length) && !existingFooter) {
      const footer = document.createElement("div");
      footer.className = "tombi-mobile-social-links";
      for (const control of [themeButton, ...socialLinks].filter(Boolean)) {
        const copy = control.cloneNode(true);
        copy.classList.remove("hidden", "sm:flex", "md:flex", "sm:block", "md:block", "w-6", "h-6");
        copy.classList.add("tombi-mobile-social-link");
        copy.removeAttribute("id");
        copy.removeAttribute("data-hk");
        copy.querySelectorAll("[data-hk]").forEach((node) => node.removeAttribute("data-hk"));
        if (control === themeButton) {
          wireArchiveTheme(copy);
        }
        footer.append(copy);
      }
      mobileNav.append(footer);
    }
  }

  function start() {
    const header = document.querySelector("header");
    if (header) {
      mount(header);
    } else {
      const observer = new MutationObserver(() => {
        const header = document.querySelector("header");
        if (!header) return;
        observer.disconnect();
        mount(header);
      });
      observer.observe(document.body, { childList: true, subtree: true });
    }
  }

  if (document.querySelector('meta[name="tombi-archive-base"]') || document.documentElement.dataset.tombiHydrated === "true") {
    start();
  } else {
    document.addEventListener("tombi:hydrated", start, { once: true });
  }
})();
