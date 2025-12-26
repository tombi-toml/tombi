import { A, useLocation } from "@solidjs/router";
import { IoChevronForward } from "solid-icons/io";
import { createMemo, createSignal, For } from "solid-js";
import type { DocIndex } from "~/utils/doc-index";
import { normalizePath } from "~/utils/path";
import docIndex from "../../doc-index.json";

const docIndexs: DocIndex[] = docIndex as DocIndex[];

const TreeItem = (props: { item: DocIndex; level: number }) => {
  const location = useLocation();
  const currentPath = createMemo(() => normalizePath(location.pathname));
  const isCurrentPage = createMemo(
    () => currentPath() === normalizePath(props.item.path),
  );

  const hasChildren = props.item.children && props.item.children.length > 0;
  const shouldBeExpanded = createMemo(
    () =>
      hasChildren &&
      (isCurrentPage() ||
        props.item.children?.some(
          (child) => normalizePath(child.path) === currentPath(),
        )),
  );

  const [isExpanded, setIsExpanded] = createSignal(shouldBeExpanded());

  return (
    <div class={`my-4 pl-${props.level * 2}`}>
      <div class="flex items-center">
        {isCurrentPage() ? (
          <span class="font-bold text-tombi-700 dark:text-color-yellow block pl-2 mr-2 py-1 flex-grow">
            {props.item.title}
          </span>
        ) : (
          <A
            href={props.item.path}
            class="text-[--color-text] no-underline block pl-2 mr-2 py-1 hover:text-[--color-primary] flex-grow"
          >
            {props.item.title}
          </A>
        )}
        {hasChildren && (
          <button
            type="button"
            onClick={() => setIsExpanded(!isExpanded())}
            class="w-5 h-5 flex items-center justify-center font-bold text-[--color-primary] hover:text-blue-600 bg-transparent border-none"
          >
            <div
              class="transform transition-transform duration-300"
              classList={{ "rotate-90": isExpanded() }}
            >
              <IoChevronForward size={16} />
            </div>
          </button>
        )}
      </div>
      {hasChildren && isExpanded() && (
        <div class="ml-2">
          <For each={props.item.children}>
            {(child) => <TreeItem item={child} level={props.level + 1} />}
          </For>
        </div>
      )}
    </div>
  );
};

export function Sidebar() {
  return (
    <nav class="w-[250px] h-screen sticky top-0 overflow-y-auto p-4 bg-[--color-bg-secondary] border-r border-[--color-border] md:block hidden">
      <For each={docIndexs}>{(item) => <TreeItem item={item} level={0} />}</For>
    </nav>
  );
}
