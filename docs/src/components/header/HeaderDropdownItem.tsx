import { A } from "@solidjs/router";
import { IoChevronForward } from "solid-icons/io";
import type { ParentProps } from "solid-js";
import { createEffect, createSignal, For, onCleanup, onMount } from "solid-js";
import type { DicIndex } from "~/utils/doc-index";

interface HeaderDropdownItemProps {
  href: string;
  hasBorder?: boolean;
  onSelect: () => void;
  onChildrenResize?: () => void;
  childrenItems?: DicIndex[];
  level?: number;
  isExpanded: boolean;
}

export function HeaderDropdownItem(
  props: ParentProps<HeaderDropdownItemProps>,
) {
  const [isExpanded, setIsExpanded] = createSignal(false);

  const toggleOpen = (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsExpanded(!isExpanded());
  };

  const level = props.level ?? 0;
  const hasChildren = !!props.childrenItems?.length;

  const handleClick = (e: MouseEvent) => {
    if (hasChildren && (e.target as HTMLElement).closest("button")) {
      e.preventDefault();
      return;
    }
    props.onSelect();
  };

  let contentRef: HTMLDivElement | undefined;
  const [maxHeightStyle, setMaxHeightStyle] = createSignal("0px");

  const updateHeight = () => {
    if (contentRef) {
      const newHeight = isExpanded() ? contentRef.scrollHeight : 0;
      setMaxHeightStyle(`${newHeight}px`);
      props.onChildrenResize?.();
    }
  };

  createEffect(updateHeight);

  onMount(() => {
    if (contentRef) {
      const observer = new ResizeObserver(updateHeight);
      observer.observe(contentRef);
      onCleanup(() => observer.disconnect());
    }
  });

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
            class="w-8 h-16 p-x-8 flex items-center justify-center text-white/60 hover:text-white hover:bg-white/10 bg-transparent border-none rounded"
            aria-expanded={isExpanded()}
            aria-label={isExpanded() ? "Collapse section" : "Expand section"}
            tabindex={props.isExpanded ? 0 : -1}
          >
            <IoChevronForward
              class="transition-transform duration-400 ease-in-out"
              classList={{ "rotate-90": isExpanded() }}
              size={16}
            />
          </button>
        )}
      </A>
      {hasChildren && (
        <div
          ref={(el) => {
            contentRef = el;
          }}
          class="flex flex-col pl-4 space-y-1 overflow-hidden transition-all duration-400 ease-linear"
          style={{ "max-height": maxHeightStyle() }}
        >
          <For each={props.childrenItems}>
            {(child) => (
              <HeaderDropdownItem
                href={child.path}
                onSelect={props.onSelect}
                childrenItems={child.children}
                level={level + 1}
                hasBorder={false}
                onChildrenResize={updateHeight}
                isExpanded={isExpanded()}
              >
                {child.title}
              </HeaderDropdownItem>
            )}
          </For>
        </div>
      )}
    </div>
  );
}
