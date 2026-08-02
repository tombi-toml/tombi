import {
  type Accessor,
  createContext,
  createSignal,
  type ParentProps,
  useContext,
} from "solid-js";

type CodeTabsSelection = {
  scope: string;
  key: string;
};

type CodeTabsContextValue = {
  selectedKey: Accessor<string | undefined>;
  selectKey: (key: string) => void;
};

const CodeTabsContext = createContext<CodeTabsContextValue>();

export function CodeTabsProvider(props: ParentProps<{ scope: string }>) {
  const [selection, setSelection] = createSignal<CodeTabsSelection>();
  const selectedKey = () => {
    const currentSelection = selection();
    return currentSelection?.scope === props.scope
      ? currentSelection.key
      : undefined;
  };

  return (
    <CodeTabsContext.Provider
      value={{
        selectedKey,
        selectKey: (key) => setSelection({ scope: props.scope, key }),
      }}
    >
      {props.children}
    </CodeTabsContext.Provider>
  );
}

export function useCodeTabsSelection() {
  return useContext(CodeTabsContext);
}
