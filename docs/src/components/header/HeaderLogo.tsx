import { A } from "@solidjs/router";
import { createSignal, For } from "solid-js";
import { HeaderDropdown } from "./HeaderDropdown";

type LogoProps = {
  id: string;
  src: string;
  class: string;
  linkClass: string;
  preventDefault: boolean;
};

const logoProps: LogoProps[] = [
  {
    id: "mobile-logo",
    src: `${import.meta.env.BASE_URL}/icon.svg`,
    class: "h-16 w-16",
    linkClass: "md:hidden flex",
    preventDefault: true,
  },
  {
    id: "desktop-logo",
    src: `${import.meta.env.BASE_URL}/tombi-transparent.svg`,
    class: "h-16 w-auto",
    linkClass: "hidden md:flex",
    preventDefault: false,
  },
];

export function HeaderLogo() {
  const [isOpen, setIsOpen] = createSignal(false);

  const toggleMenu = (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
    setIsOpen(!isOpen());
  };

  const handleSelect = () => {
    setIsOpen(false);
  };

  const handleLogoClick = (e: MouseEvent, props: LogoProps) => {
    if (props.preventDefault) {
      e.preventDefault();
      e.stopPropagation();
      toggleMenu(e);
    }
  };

  const handleLogoKeyDown = (e: KeyboardEvent, props: LogoProps) => {
    if (props.preventDefault && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      e.stopPropagation();
      toggleMenu(e);
    }
  };

  return (
    <div class="flex-shrink-0 flex items-center relative">
      <div class="ml-4 menu-toggle">
        <For each={logoProps}>
          {(props) => (
            <A
              id={props.id}
              href="/"
              class={`${props.linkClass} outline-none items-center no-underline transition-all duration-300 ease-in-out focus-visible:ring-2 focus-visible:ring-tombi-focus focus:rounded-lg relative cursor-pointer md:cursor-default`}
              onClick={(e) => handleLogoClick(e, props)}
              onKeyDown={(e) => handleLogoKeyDown(e, props)}
            >
              <img
                src={props.src}
                alt="Tombi Logo"
                class={`${props.class} rounded-lg`}
              />
            </A>
          )}
        </For>
      </div>

      <HeaderDropdown isExpanded={isOpen} onSelect={handleSelect} />
    </div>
  );
}
