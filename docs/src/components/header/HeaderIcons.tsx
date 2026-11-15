import { TbSearch, TbX } from "solid-icons/tb";
import { IconButton } from "../button/IconButton";
import { HeaderSocialLinks } from "./HeaderSocialLinks";
import { HeaderThemeToggle } from "./HeaderThemeToggle";

interface HeaderIconsProps {
  isSearchOpen: boolean;
  setIsSearchOpen: (open: boolean) => void;
  savePreviousActiveElement?: () => void;
}

export function HeaderIcons(props: HeaderIconsProps) {
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
      <HeaderThemeToggle />
      <HeaderSocialLinks />
    </div>
  );
}
