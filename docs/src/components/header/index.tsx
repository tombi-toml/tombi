import { useLocation } from "@solidjs/router";
import { createSignal } from "solid-js";
import { type DocIndex, flattenDocPages } from "~/utils/doc-index";
import docIndex from "../../../doc-index.json";
import { HeaderIcons } from "./HeaderIcons";
import { HeaderLogo } from "./HeaderLogo";
import { HeaderSearch } from "./HeaderSearch";
import { HeaderTab } from "./HeaderTab";

export function Header() {
  const [isSearchOpen, setIsSearchOpen] = createSignal(false);
  const location = useLocation();
  let previousActiveElement: HTMLElement | null = null;

  const getPageTitle = () => {
    const base = import.meta.env.BASE_URL;
    let path = location.pathname;
    if (base && path.startsWith(base)) {
      path = path.slice(base.length) || "/";
    }
    if (path === "/") return "Tombi";
    if (path === "/playground") return "Playground";

    const flattenedPages = flattenDocPages(docIndex as DocIndex[]);
    const page = flattenedPages.find((page) => page.path === path);
    return page?.title || "Tombi";
  };

  const savePreviousActiveElement = () => {
    previousActiveElement = document.activeElement as HTMLElement;
  };

  const getPreviousActiveElement = () => previousActiveElement;

  const clearPreviousActiveElement = () => {
    previousActiveElement = null;
  };

  return (
    <header class="fixed top-0 left-0 right-0 bg-tombi-primary shadow-lg z-40">
      <nav class="max-w-7xl mx-auto">
        <div class="flex justify-between h-20 items-center relative">
          <HeaderLogo />
          <div class="hidden md:flex items-center space-x-4 mx-4">
            <HeaderTab href="/docs">Docs</HeaderTab>
            <HeaderTab href="/playground">Playground</HeaderTab>
          </div>
          <h1
            class={`${
              !isSearchOpen() ? "opacity-100" : "opacity-0"
            } absolute left-1/2 transform -translate-x-1/2 text-white text-lg font-bold text-center md:hidden pointer-events-none transition-opacity duration-200 px-20 max-w-full overflow-hidden text-ellipsis whitespace-nowrap`}
          >
            {getPageTitle()}
          </h1>
          <HeaderSearch
            isSearchOpen={isSearchOpen()}
            setIsSearchOpen={setIsSearchOpen}
            getPreviousActiveElement={getPreviousActiveElement}
            savePreviousActiveElement={savePreviousActiveElement}
            clearPreviousActiveElement={clearPreviousActiveElement}
          />
          <HeaderIcons
            isSearchOpen={isSearchOpen()}
            setIsSearchOpen={setIsSearchOpen}
            savePreviousActiveElement={savePreviousActiveElement}
          />
        </div>
      </nav>
    </header>
  );
}
