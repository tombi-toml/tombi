import type { ParentComponent } from "solid-js";

export type HighlightProps = {
  icon: string;
  title: string;
};

const createHighlight = (props: HighlightProps): ParentComponent => {
  return (childProps) => (
    <div class="my-4 p-4 bg-blue-50 border-l-4 border-blue-500 dark:bg-blue-950 dark:border-blue-400">
      <div class="flex items-center gap-2 font-medium text-blue-800 dark:text-blue-200 mb-2">
        <span>{props.icon}</span>
        <span>{props.title}</span>
      </div>
      <div class="text-gray-700 dark:text-gray-300">{childProps.children}</div>
    </div>
  );
};

export const Note: ParentComponent = createHighlight({
  icon: "🗒️",
  title: "Note",
});

export const Tip: ParentComponent = createHighlight({
  icon: "💡",
  title: "Tip",
});

export const Warning: ParentComponent = createHighlight({
  icon: "⚠️",
  title: "Warning",
});
