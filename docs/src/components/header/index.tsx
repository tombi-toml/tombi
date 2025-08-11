import { useLocation } from "@solidjs/router";
import { createSignal } from "solid-js";
import { flattenDocPages } from "~/utils/doc-index";
import docIndex from "../../../doc-index.json";
import { HeaderIcons } from "./HeaderIcons";
import { HeaderLogo } from "./HeaderLogo";
import { HeaderSearch } from "./HeaderSearch";
import { HeaderTab } from "./HeaderTab";

export function Header() {
  const [isSearchOpen, setIsSearchOpen] = createSignal(false);
  const location = useLocation();

  const getPageTitle = () => {
    const base = import.meta.env.BASE_URL;
    let path = location.pathname;
    if (base && path.startsWith(base)) {
      path = path.slice(base.length) || "/";
    }
    if (path === "/") return "Tombi";
    if (path === "/playground") return "Playground";

    const flattenedPages = flattenDocPages(docIndex);
    const page = flattenedPages.find((page) => page.path === path);
    return page?.title || "Tombi";
  };

  return (
    <header class="fixed top-0 left-0 right-0 bg-tombi-primary shadow-lg z-40">
      <nav class="max-w-7xl mx-auto">
        <div class="flex justify-between h-20 items-center">
          <HeaderLogo />
          <div class="hidden md:flex items-center space-x-4 mx-4">
            <HeaderTab href="/docs">Docs</HeaderTab>
            <HeaderTab href="/playground">Playground</HeaderTab>
          </div>
          <h1
            class={`${
              !isSearchOpen() ? "w-full md:opacity-100" : "w-0 opacity-0"
            } flex justify-center text-white text-lg font-bold text-center md:hidden`}
          >
            {getPageTitle()}
          </h1>
          <HeaderSearch
            isSearchOpen={isSearchOpen()}
            setIsSearchOpen={setIsSearchOpen}
          />
          <HeaderIcons />
        </div>
      </nav>
    </header>
  );
}
