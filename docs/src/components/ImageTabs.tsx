import { createSignal, For } from "solid-js";

export type ImageTab = {
  key: string;
  label: string;
  src: string;
  alt: string;
};

type ImageTabsProps = {
  tabs: ImageTab[];
  defaultKey?: string;
};

export default function ImageTabs(props: ImageTabsProps) {
  const [active, setActive] = createSignal(
    props.defaultKey || props.tabs[0].key,
  );
  const current = () => props.tabs.find((tab) => tab.key === active());

  return (
    <div>
      <div class="flex justify-center mb-4">
        <For each={props.tabs}>
          {(tab) => (
            <button
              type="button"
              onClick={() => setActive(tab.key)}
              class={`px-4 font-semibold text-base cursor-pointer bg-transparent border-0 relative
                  ${
                    active() === tab.key
                      ? "text-gray-800 dark:text-gray-100"
                      : "text-gray-500 dark:text-gray-400"
                  }
                  focus-visible:outline-none
                `}
              style="min-width: 64px; height: 40px;"
              data-key={tab.key}
            >
              {tab.label}
              {active() === tab.key && (
                <div class="absolute bottom-0 left-0 w-full h-1 bg-tombi-700 dark:bg-yellow" />
              )}
            </button>
          )}
        </For>
      </div>
      <img
        src={current()?.src || ""}
        alt={current()?.alt || ""}
        style="display: block; margin: 0 auto; width: 80%"
      />
    </div>
  );
}
