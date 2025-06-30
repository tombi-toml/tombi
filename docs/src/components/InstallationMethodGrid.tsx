import { createSignal, For } from "solid-js";
import type { Component } from "solid-js";

interface InstallationMethod {
  id: string;
  name: string;
  image: string;
  color: string;
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
    color: "#00D4FF",
  },
  {
    id: "python",
    name: "PyPI",
    image: "/pypi.svg",
    color: "#3776AB",
    category: "package-manager",
  },
  {
    id: "javascript",
    name: "npm",
    image: "/npm.svg",
    color: "#CB3837",
    category: "package-manager",
  },

  // Editors
  {
    id: "vscode",
    name: "VSCode",
    image: "/vscode.svg",
    color: "#007ACC",
    category: "editor",
  },
  {
    id: "cursor",
    name: "Cursor",
    image: "/cursor.svg",
    color: "#000000",
    category: "editor",
  },
  {
    id: "zed",
    name: "Zed",
    image: "/zed.jpeg",
    color: "#084CDF",
    category: "editor",
  },
  {
    id: "neovim",
    name: "Neovim",
    image: "/neovim.png",
    color: "#57A143",
    category: "editor",
  },
  {
    id: "emacs",
    name: "Emacs",
    image: "/emacs.png",
    color: "#7F5AB6",
    category: "editor",
  },

  // CI/CD
  {
    id: "github-actions",
    name: "GitHub Actions",
    image: "/github-action.png",
    color: "#2088FF",
    category: "ci",
  },
];

export const InstallationMethodGrid: Component<InstallationMethodGridProps> = (
  props,
) => {
  const [selectedCategory, setSelectedCategory] = createSignal<string | null>(
    null,
  );

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
          class={`px-4 py-2 rounded-lg transition-all ${
            !selectedCategory()
              ? "bg-blue-500 text-white shadow-md"
              : "bg-gray-100 text-gray-700 hover:bg-gray-200"
          }`}
        >
          All
        </button>
        <For each={categories}>
          {(category) => (
            <button
              type="button"
              onClick={() => setSelectedCategory(category.id)}
              class={`px-4 py-2 rounded-lg transition-all flex items-center gap-2 ${
                selectedCategory() === category.id
                  ? "bg-blue-500 text-white shadow-md"
                  : "bg-gray-100 text-gray-700 hover:bg-gray-200"
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
              class="group relative p-2 rounded-xl border-2 border-gray-200 bg-white hover:border-gray-300 hover:shadow-lg transition-all cursor-pointer"
              style={{
                "border-color": `${method.color}20`,
              }}
            >
              <div class="flex flex-col items-center gap-2">
                <img
                  src={`${import.meta.env.BASE_URL}${method.image}`}
                  alt={method.name}
                  class="w-8 h-8 object-contain group-hover:scale-110 transition-transform"
                  style={{
                    filter: `drop-shadow(0 2px 4px ${method.color}40)`,
                  }}
                />
                <span class="text-sm font-medium text-gray-700">
                  {method.name}
                </span>
              </div>
              <div
                class="absolute inset-0 rounded-xl opacity-0 group-hover:opacity-10 transition-opacity"
                style={{ "background-color": method.color }}
              />
            </button>
          )}
        </For>
      </div>
    </div>
  );
};
