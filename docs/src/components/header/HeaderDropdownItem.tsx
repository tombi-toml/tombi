import { A } from "@solidjs/router";
import { IoChevronForward } from "solid-icons/io";
import type { ParentProps } from "solid-js";
import { createEffect, createSignal, For } from "solid-js";
import type { DocIndex } from "~/utils/doc-index";

interface HeaderDropdownItemProps {
  href: string;
  hasBorder?: boolean;
  onSelect: () => void;
  childrenItems?: DocIndex[];
  level?: number;
  isExpanded: boolean;
}

export function HeaderDropdownItem(
  props: ParentProps<HeaderDropdownItemProps>,
) {
  const [isExpanded, setIsExpanded] = createSignal(false);
  const childrenVisible = () => props.isExpanded && isExpanded();

  const toggleOpen = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsExpanded(!isExpanded());
  };

  const level = props.level ?? 0;
  const hasChildren = !!props.childrenItems?.length;
  let childrenRef: HTMLDivElement | undefined;
  let contentRef: HTMLDivElement | undefined;

  createEffect(() => {
    const expanded = isExpanded();
    if (!childrenRef || !contentRef) return;

    const height = childrenRef.getBoundingClientRect().height;
    const targetHeight = expanded ? contentRef.scrollHeight : 0;
    childrenRef.style.height = `${height}px`;
    void childrenRef.offsetHeight;
    childrenRef.style.height =
      expanded && height === targetHeight ? "auto" : `${targetHeight}px`;
  });

  const handleClick = (e: MouseEvent) => {
    if (hasChildren && (e.target as HTMLElement).closest("button")) {
      e.preventDefault();
      return;
    }
    props.onSelect();
  };

  return (
    <div
      class={`flex flex-col ${props.hasBorder ? "border-b border-white/10" : ""}`}
    >
      <A
        href={props.href}
        class={`flex items-center pl-3 m-1 h-16 text-white ${level === 0 ? "text-lg" : "text-base"} font-medium no-underline hover:bg-white/5 active:bg-white/10 transition-colors`}
        onClick={handleClick}
        tabindex={props.isExpanded ? 0 : -1}
      >
        <span class="flex-grow" style={{ "padding-left": `${level}rem` }}>
          {props.children}
        </span>
        {hasChildren && (
          <button
            type="button"
            onClick={toggleOpen}
            class="w-8 h-16 p-x-8 flex items-center justify-center text-white/60 hover:text-white hover:bg-white/10 bg-transparent border-none rounded cursor-pointer"
            aria-expanded={isExpanded()}
            aria-label={isExpanded() ? "Collapse section" : "Expand section"}
            tabindex={props.isExpanded ? 0 : -1}
          >
            <IoChevronForward
              class="transition-transform duration-200 ease-in-out"
              classList={{ "rotate-90": isExpanded() }}
              size={16}
            />
          </button>
        )}
      </A>
      {hasChildren && (
        <div
          ref={childrenRef}
          class="overflow-hidden transition-[height] duration-200 ease-in-out"
          style={{ height: "0px" }}
          onTransitionEnd={(event) => {
            if (
              event.target === event.currentTarget &&
              event.propertyName === "height" &&
              isExpanded()
            ) {
              event.currentTarget.style.height = "auto";
            }
          }}
          inert={!childrenVisible()}
        >
          <div ref={contentRef} class="overflow-hidden">
            <div class="flex flex-col pl-4 space-y-1">
              <For each={props.childrenItems}>
                {(child) => (
                  <HeaderDropdownItem
                    href={child.path}
                    onSelect={props.onSelect}
                    childrenItems={child.children}
                    level={level + 1}
                    hasBorder={false}
                    isExpanded={childrenVisible()}
                  >
                    {child.title}
                  </HeaderDropdownItem>
                )}
              </For>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
