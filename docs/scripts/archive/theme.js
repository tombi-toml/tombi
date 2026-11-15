try {
  document.documentElement.classList.toggle(
    "dark",
    (localStorage.getItem("theme") ||
      (matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light")) === "dark",
  );
} catch {}
