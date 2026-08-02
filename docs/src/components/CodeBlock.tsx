import Prism from "prismjs";
import "prismjs/components/prism-toml";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-json";
import "prismjs/components/prism-typescript";
import "prismjs/components/prism-powershell";
import type { ParentComponent } from "solid-js";
import { CopyButton } from "./CopyButton";

interface CodeBlockProps {
  code: string;
  language?: string;
}

function escapeHtml(value: string) {
  return value.replace(
    /[&<>"']/g,
    (character) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[character] || character,
  );
}

export const CodeBlock: ParentComponent<CodeBlockProps> = (props) => {
  const language = () => props.language || "text";
  const highlightedCode = () => {
    const grammar = Prism.languages[language()];
    return grammar
      ? Prism.highlight(props.code, grammar, language())
      : escapeHtml(props.code);
  };

  return (
    <div class="code-block-wrapper relative max-w-full overflow-hidden my-4">
      <pre
        class={`language-${language()} overflow-x-auto max-w-full pr-20`}
        tabindex="-1"
      >
        <code
          class={`language-${language()}`}
          tabindex="-1"
          innerHTML={highlightedCode()}
        />
      </pre>
      <div class="copy-button language-text">
        <CopyButton text={props.code} />
      </div>
    </div>
  );
};
