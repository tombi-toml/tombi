import type { Root } from "mdast";
import type { MdxJsxFlowElement } from "mdast-util-mdx-jsx";
import { visit } from "unist-util-visit";

export function remarkCode() {
  return (tree: Root) => {
    visit(tree, "code", (node) => {
      const { lang, value } = node;
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
