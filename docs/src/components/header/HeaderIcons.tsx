import { TbMoonFilled, TbSearch, TbSunFilled, TbX } from "solid-icons/tb";
import { createSignal, onMount } from "solid-js";
import { IconButton } from "../button/IconButton";
import { HeaderSocialLinks } from "./HeaderSocialLinks";

interface HeaderIconsProps {
  isSearchOpen: boolean;
  setIsSearchOpen: (open: boolean) => void;
  savePreviousActiveElement?: () => void;
}

export function HeaderIcons(props: HeaderIconsProps) {
  const [isDark, setIsDark] = createSignal(false);
  const [rotation, setRotation] = createSignal(0);

  onMount(() => {
    if (typeof window !== "undefined") {
      const darkModePreference = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      const storedTheme = localStorage.getItem("theme");
      setIsDark(storedTheme === "dark" || (!storedTheme && darkModePreference));
      document.documentElement.classList.toggle("dark", isDark());
    }
  });

  const toggleDarkMode = () => {
    const newDarkMode = !isDark();
    setIsDark(newDarkMode);
    if (rotation() % 360 === 0 && newDarkMode) {
      setRotation(rotation() + 180);
    }
    setRotation((rotation() + 180) % 36000);
    localStorage.setItem("theme", newDarkMode ? "dark" : "light");
    document.documentElement.classList.toggle("dark", newDarkMode);
  };

  return (
    <div class="flex items-center md:pl-4 pr-4 space-x-1">
      <IconButton
        onMouseDown={() => {
          if (!props.isSearchOpen) {
            // Save the currently focused element before the button gets focus
            props.savePreviousActiveElement?.();
          }
        }}
        onClick={() => {
          props.setIsSearchOpen(!props.isSearchOpen);
        }}
        class="md:hidden py-1 relative"
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
      <IconButton
        id="dark-mode-toggle"
        onClick={toggleDarkMode}
        alt="Toggle dark mode"
        class="hidden sm:flex items-center justify-center transition-transform duration-300 ease-out forwards"
        style={`transform: rotate(${rotation()}deg)`}
      >
        {isDark() ? <TbMoonFilled size={28} /> : <TbSunFilled size={28} />}
      </IconButton>
      <HeaderSocialLinks />
    </div>
  );
}
