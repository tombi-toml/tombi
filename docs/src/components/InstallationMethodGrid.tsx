import { createSignal, For, createEffect } from "solid-js";
import type { Component } from "solid-js";

interface InstallationMethod {
  id: string;
  name: string;
  image: string;
  imageDark?: string;
  category?: "package-manager" | "editor" | "ci";
}

interface InstallationMethodGridProps {
  onSelect: (method: string) => void;
}

const installationMethods: InstallationMethod[] = [
  // Package Managers
  {
    id: "cli",
    name: "CLI",
    image: "/terminal.svg",
  },
  {
    id: "python",
    name: "PyPI",
    image: "/pypi.svg",
    category: "package-manager",
  },
  {
    id: "javascript",
    name: "npm",
    image: "/npm.svg",
    category: "package-manager",
  },

  // Editors
  {
    id: "vscode",
    name: "VSCode",
    image: "/vscode.svg",
    category: "editor",
  },
  {
    id: "open-vsx",
    name: "Cursor",
    image: "/cursor.svg",
    imageDark: "/cursor-dark.png",
    category: "editor",
  },
  {
    id: "open-vsx",
    name: "Windsurf",
    image: "/windsurf.svg",
    category: "editor",
  },
  {
    id: "zed",
    name: "Zed",
    image: "/zed.jpeg",
    category: "editor",
  },
  {
    id: "neovim",
    name: "Neovim",
    image: "/neovim.png",
    category: "editor",
  },
  {
    id: "emacs",
    name: "Emacs",
    image: "/emacs.png",
    category: "editor",
  },

  // CI/CD
  {
    id: "github-actions",
    name: "GitHub Actions",
    image: "/github-action.png",
    category: "ci",
  },
];

export const InstallationMethodGrid: Component<InstallationMethodGridProps> = (
  props,
) => {
  const [selectedCategory, setSelectedCategory] = createSignal<string | null>(
    null,
  );
  const [isDarkMode, setIsDarkMode] = createSignal(false);

  // Check for dark mode
  createEffect(() => {
    const checkDarkMode = () => {
      const storedTheme = localStorage.getItem("theme");
      const hasDarkClass = document.documentElement.classList.contains("dark");
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;

      // Check localStorage first, then dark class, then system preference
      const isDark =
        storedTheme === "dark" ||
        (!storedTheme && hasDarkClass) ||
        (!storedTheme && !hasDarkClass && prefersDark);
      setIsDarkMode(isDark);
    };

    // Initial check
    checkDarkMode();

    // Listen for class changes on document.documentElement
    const observer = new MutationObserver(() => {
      checkDarkMode();
    });

    // Set up observer
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    // Cleanup
    return () => {
      observer.disconnect();
    };
  });

  const categories = [
    { id: "package-manager", label: "Package Managers" },
    { id: "editor", label: "Editors" },
    { id: "ci", label: "CI/CD" },
  ];

  const filteredMethods = () => {
    if (!selectedCategory()) return installationMethods;
    return installationMethods.filter((m) => m.category === selectedCategory());
  };

  return (
    <div class="my-8 w-full">
      <div class="flex gap-2 mb-6 flex-wrap">
        <button
          type="button"
          onClick={() => setSelectedCategory(null)}
          class={`px-4 py-2 rounded-lg border-0 transition-all btn-focus ${
            !selectedCategory()
              ? "bg-tombi-primary text-white shadow-lg hover:shadow-xl"
              : "bg-gray-200 text-gray-800 hover:bg-gray-300 dark:bg-gray-600 dark:text-gray-100 dark:hover:bg-gray-500"
          }`}
        >
          All
        </button>
        <For each={categories}>
          {(category) => (
            <button
              type="button"
              onClick={() => setSelectedCategory(category.id)}
              class={`px-4 py-2 rounded-lg border-0 transition-all flex items-center gap-2 btn-focus ${
                selectedCategory() === category.id
                  ? "bg-tombi-primary text-white shadow-lg hover:shadow-xl"
                  : "bg-gray-200 text-gray-800 hover:bg-gray-300 dark:bg-gray-600 dark:text-gray-100 dark:hover:bg-gray-500"
              }`}
            >
              <span>{category.label}</span>
            </button>
          )}
        </For>
      </div>

      <div class="grid w-full grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
        <For each={filteredMethods()}>
          {(method) => (
            <button
              type="button"
              onClick={() => props.onSelect(method.id)}
              class="group relative p-2 rounded-xl border-0 bg-white hover:shadow-lg transition-all cursor-pointer dark:bg-gray-800"
            >
              <div class="flex flex-col items-center gap-2">
                <img
                  src={`${import.meta.env.BASE_URL}${
                    isDarkMode() && method.imageDark
                      ? method.imageDark
                      : method.image
                  }`}
                  alt={method.name}
                  class="w-8 h-8 object-contain group-hover:scale-110 transition-transform"
                />
                <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                  {method.name}
                </span>
              </div>
            </button>
          )}
        </For>
      </div>
    </div>
  );
};
