import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Root } from "mdast";
import type { MdxJsxFlowElement } from "mdast-util-mdx-jsx";
import { visit } from "unist-util-visit";

const DOCS_ROOT = process.cwd();

// Embed an external file into a code block: ```toml file="snippets/example.toml"
export function readCodeFile(
  meta: string | null | undefined,
): string | undefined {
  const path = meta?.match(/(?:^|\s)file="([^"]+)"/)?.[1];
  if (!path) {
    return undefined;
  }
  return readFileSync(resolve(DOCS_ROOT, path), "utf-8").trimEnd();
}

export function remarkCode() {
  return (tree: Root) => {
    visit(tree, "code", (node) => {
      const { lang, meta } = node;
      const value = readCodeFile(meta) ?? node.value;
      const mdxNode: MdxJsxFlowElement = {
        type: "mdxJsxFlowElement",
        name: "CodeBlock",
        attributes: [
          {
            type: "mdxJsxAttribute",
            name: "code",
            value: value,
          },
          {
            type: "mdxJsxAttribute",
            name: "language",
            value: lang || undefined,
          },
        ],
        children: [],
      };

      Object.assign(node, mdxNode);
    });
  };
}
