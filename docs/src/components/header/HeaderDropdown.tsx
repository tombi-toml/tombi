import type { Accessor } from "solid-js";
import { createEffect, For } from "solid-js";
import { isServer } from "solid-js/web";
import type { DicIndex } from "~/utils/doc-index";
import docIndex from "../../../doc-index.json";
import { HeaderDropdownItem } from "./HeaderDropdownItem";

interface HeaderDropdownProps {
  isExpanded: Accessor<boolean>;
  onSelect: () => void;
}

const menuItems: DicIndex[] = [
  { title: "Home", path: "/" },
  { title: "Docs", path: "/docs", children: docIndex },
  { title: "Playground", path: "/playground" },
];

export function HeaderDropdown(props: HeaderDropdownProps) {
  const handleClickOutside = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (
      !target.closest(".header-dropdown") &&
      !target.closest(".menu-toggle")
    ) {
      props.onSelect();
    }
  };

  createEffect(() => {
    if (!isServer && props.isExpanded()) {
      document.addEventListener("click", handleClickOutside);
      return () => {
        document.removeEventListener("click", handleClickOutside);
      };
    }
  });

  return (
    <div
      class="header-dropdown fixed inset-x-0 top-20 bg-tombi-primary shadow-lg z-40 md:hidden overflow-hidden transition-all duration-300 ease-linear"
      classList={{
        "max-h-0": !props.isExpanded(),
        "max-h-[calc(100vh-5rem)]": props.isExpanded(),
      }}
    >
      <nav class="flex flex-col px-4 pb-2 max-h-[calc(100vh-5rem)] overflow-y-auto">
        <For each={menuItems}>
          {(item, idx) => (
            <HeaderDropdownItem
              href={item.path}
              onSelect={props.onSelect}
              childrenItems={item.children}
              level={0}
              hasBorder={idx() < menuItems.length - 1}
              isExpanded={props.isExpanded()}
            >
              {item.title}
            </HeaderDropdownItem>
          )}
        </For>
      </nav>
    </div>
  );
}
