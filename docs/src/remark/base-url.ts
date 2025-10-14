import type { Link, Root } from "mdast";
import type { MdxJsxFlowElement } from "mdast-util-mdx";
import { visit } from "unist-util-visit";

export function remarkBaseUrl() {
  return (tree: Root) => {
    const processUrl = (url: string): string => {
      if (url.startsWith("/")) {
        let baseUrl = process.env.BASE_URL || "/";
        if (baseUrl === "/") {
          baseUrl = "/_build/";
        }
        if (baseUrl.endsWith("/")) {
          baseUrl = baseUrl.slice(0, -1);
        }
        return `${baseUrl}${url}`;
      }
      return url;
    };

    // Process URLs in links
    visit(tree, "link", (node: Link) => {
      node.url = processUrl(node.url);
    });

    // Process URLs in images
    visit(tree, "mdxJsxFlowElement", (node: MdxJsxFlowElement) => {
      if (node.name === "img") {
        for (const attr of node.attributes) {
          if (
            attr.type === "mdxJsxAttribute" &&
            attr.name === "src" &&
            typeof attr.value === "string"
          ) {
            attr.value = processUrl(attr.value);
          }
        }
      }
    });
  };
}
