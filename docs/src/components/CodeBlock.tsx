import type { ParentComponent } from "solid-js";
import { CopyButton } from "./CopyButton";

interface CodeBlockProps {
  code: string;
  language?: string;
}

export const CodeBlock: ParentComponent<CodeBlockProps> = (props) => {
  return (
    <div class="code-block-wrapper relative max-w-full overflow-hidden my-4">
      <pre
        class={`language-${props.language || "text"} overflow-x-auto max-w-full pr-20`}
      >
        <code class={`language-${props.language || "text"}`}>
          {`${props.code}`}
        </code>
      </pre>
      <div class="copy-button language-text">
        <CopyButton text={props.code} />
      </div>
    </div>
  );
};
