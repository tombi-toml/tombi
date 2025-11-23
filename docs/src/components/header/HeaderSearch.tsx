import { TbLoaderQuarter, TbSearch } from "solid-icons/tb";
import { createSignal, onMount } from "solid-js";
import { detectOperatingSystem } from "~/utils/platform";
import { type SearchResult, searchDocumentation } from "~/utils/search";
import breakpoints from "../../../breakpoints.json";
import { SearchResults } from "../search/SearchResults";

interface HeaderSearchProps {
  isSearchOpen: boolean;
  setIsSearchOpen: (open: boolean) => void;
  getPreviousActiveElement?: () => HTMLElement | null;
  savePreviousActiveElement?: () => void;
  clearPreviousActiveElement?: () => void;
}

export function HeaderSearch(props: HeaderSearchProps) {
  const [isMac, setIsMac] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [searchResults, setSearchResults] = createSignal<SearchResult[]>([]);
  const [isLoading, setIsLoading] = createSignal(false);
  const [isFocused, setIsFocused] = createSignal(false);
  const [isDesktop, setIsDesktop] = createSignal(false);
  let searchInputRef: HTMLInputElement | undefined;

  onMount(() => {
    setIsMac(detectOperatingSystem() === "mac");
    setIsDesktop(window.innerWidth >= breakpoints.md);

    if (typeof window !== "undefined") {
      window.addEventListener("resize", () => {
        setIsDesktop(window.innerWidth >= breakpoints.md);
      });

      document.addEventListener("keydown", (e) => {
        if ((e.metaKey || e.ctrlKey) && e.key === "k") {
          e.preventDefault();
          // Save the currently focused element before opening search
          props.savePreviousActiveElement?.();
          searchInputRef?.focus();
          props.setIsSearchOpen(true);
        }
      });
    }
  });

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (query.trim()) {
      setIsLoading(true);
      try {
        const results = await searchDocumentation(query);
        console.log(results);
        setSearchResults(results);
      } catch (error) {
        console.error("An error occurred during search:", error);
        setSearchResults([]);
      } finally {
        setIsLoading(false);
      }
    } else {
      setSearchResults([]);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && props.isSearchOpen) {
      e.preventDefault();
      setSearchQuery("");
      setSearchResults([]);
      props.setIsSearchOpen(false);
      // Restore focus to the previously focused element
      setTimeout(() => {
        const previousElement = props.getPreviousActiveElement?.();
        if (previousElement && typeof previousElement.focus === "function") {
          previousElement.focus();
        }
        props.clearPreviousActiveElement?.();
      }, 100);
    }
  };

  return (
    <div
      class={`
        ${props.isSearchOpen ? "w-full" : "w-0"}
        md:w-full flex justify-end items-center max-w-200 gap-2
      `}
    >
      <div
        class={`${
          props.isSearchOpen ? "w-full opacity-100" : "w-0 opacity-0"
        } md:w-full md:opacity-100 transition-all duration-300 ease-in-out overflow-hidden flex items-center relative`}
      >
        <div
          class="relative w-full min-w-5"
          classList={{
            "ml-4": !isDesktop(),
          }}
        >
          <div class="absolute left-3 top-1/2 -translate-y-1/2 text-white/60">
            <TbSearch size={24} />
          </div>
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search"
            value={searchQuery()}
            onInput={(e) => handleSearch(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            class="w-full h-11 pl-12 bg-white/20 text-white placeholder-white/60 text-lg focus:bg-white/30 outline-none border-none box-border rounded-2"
            tabindex={isDesktop() || props.isSearchOpen ? 0 : -1}
          />
          <div
            class={`absolute right-4 top-1/2 -translate-y-1/2 text-white/60 text-lg transition-opacity duration-50 ${isFocused() ? "opacity-0" : "opacity-100"}`}
          >
            {isMac() ? "⌘K" : "Ctrl+K"}
          </div>
          <div
            class={`absolute right-4 top-1/2 -translate-y-1/2 text-white/60 transition-opacity duration-0 ${isFocused() && isLoading() ? "opacity-100" : "opacity-0"}`}
          >
            <TbLoaderQuarter class="animate-spin-fast" size={24} />
          </div>
        </div>
        <div
          class={`${
            isFocused() && searchQuery().trim().length > 0
              ? "opacity-100"
              : "opacity-0"
          }`}
        >
          <SearchResults results={searchResults()} />
        </div>
      </div>
    </div>
  );
}
