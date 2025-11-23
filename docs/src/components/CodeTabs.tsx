import { createSignal, For } from "solid-js";
import { CodeBlock } from "./CodeBlock";

export type Tab = {
  key: string;
  label: string;
  command: string;
};

type CodeTabsProps = {
  tabs: Tab[];
  defaultKey: string;
  language: string;
};

export default function CodeTabs(props: CodeTabsProps) {
  const [active, setActive] = createSignal(
    props.defaultKey || props.tabs[0].key,
  );
  const current = () => props.tabs.find((tab) => tab.key === active());

  return (
    <div>
      <For each={props.tabs}>
        {(tab) => (
          <button
            type="button"
            onClick={() => setActive(tab.key)}
            class={`px-4 font-semibold text-base cursor-pointer bg-transparent border-0 relative transition-colors
                ${
                  active() === tab.key
                    ? "text-gray-800 dark:text-gray-100"
                    : "text-gray-500 dark:text-gray-400"
                }
                focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-600 dark:focus-visible:ring-blue-400 focus-visible:ring-offset-2 focus-visible:rounded
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
      <CodeBlock code={current()?.command || ""} language={props.language} />
    </div>
  );
}
