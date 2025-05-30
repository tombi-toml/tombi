import { TbSearch, TbX, TbLoaderQuarter } from "solid-icons/tb";
import { createSignal, onMount } from "solid-js";
import { detectOperatingSystem } from "~/utils/platform";
import { IconButton } from "../button/IconButton";
import { searchDocumentation, type SearchResult } from "~/utils/search";
import { SearchResults } from "../search/SearchResults";
import breakpoints from "../../../breakpoints.json";

interface HeaderSearchProps {
  isSearchOpen: boolean;
  setIsSearchOpen: (open: boolean) => void;
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

  return (
    <div
      class={`
        ${props.isSearchOpen ? "w-full" : "w-10"}
        flex justify-end md:w-full items-center max-w-200 gap-2
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
      <IconButton
        onClick={() => {
          props.setIsSearchOpen(!props.isSearchOpen);
          if (!props.isSearchOpen) {
            setSearchQuery("");
            setSearchResults([]);
          }
        }}
        class="md:hidden sm:mr-0 ml-2 mr-4 py-1 relative"
        alt={props.isSearchOpen ? "Close Search" : "Search"}
      >
        <div
          class={`absolute transition-all duration-300 ${props.isSearchOpen ? "opacity-100 rotate-0" : "opacity-0 -rotate-90"}`}
        >
          <TbX size={24} />
        </div>
        <div
          class={`transition-all duration-300 ${props.isSearchOpen ? "opacity-0 rotate-90" : "opacity-100 rotate-0"}`}
        >
          <TbSearch size={24} />
        </div>
      </IconButton>
    </div>
  );
}
