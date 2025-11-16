import type { Heading, Root, RootContent } from "mdast";
import type { MdxJsxAttribute, MdxJsxFlowElement } from "mdast-util-mdx-jsx";
import docIndex from "../../doc-index.json";

export const DEFAULT_TITLE = "Tombi";
export const DEFAULT_DESCRIPTION =
  "a TOML toolkit that provides Formatter, Linter, and Language Server.";
export const DEFAULT_URL = "https://tombi-toml.github.io/tombi/";

interface DocIndexItem {
  title: string;
  description?: string;
  path: string;
  children?: DocIndexItem[];
}

// Recursively extract textual content from a heading node
function extractText(node: Heading | RootContent): string {
  if ("value" in node && typeof node.value === "string") {
    return node.value;
  }

  if ("children" in node && Array.isArray(node.children)) {
    return (node.children as RootContent[])
      .map((child) => extractText(child))
      .join("");
  }

  return DEFAULT_TITLE;
}

// Generate page URL from file path
function getPageUrl(vfile: { path?: string }): string {
  if (!vfile?.path) return DEFAULT_URL;

  // Extract relative path from src/routes
  const match = vfile.path.match(/src\/routes\/(.*?)(?:\/index)?\.mdx?$/);
  if (!match) return DEFAULT_URL;

  const routePath = match[1];
  return `${DEFAULT_URL}${routePath}`;
}

// Find description from doc-index.json recursively
function findDescriptionInDocIndex(
  path: string,
  items: DocIndexItem[],
): string | undefined {
  for (const item of items) {
    if (item.path === path) {
      return item.description;
    }
    if (item.children) {
      const found = findDescriptionInDocIndex(path, item.children);
      if (found) return found;
    }
  }
  return undefined;
}

// Get description from doc-index.json
function getPageDescription(routePath: string): string {
  const path = `/${routePath}`;
  const description = findDescriptionInDocIndex(
    path,
    docIndex as DocIndexItem[],
  );
  return description || DEFAULT_DESCRIPTION;
}

export function remarkPageHeading() {
  return (tree: Root, vfile: { path?: string }) => {
    let firstH1: Heading | null = null;
    let firstH1Index = -1;

    // Find the first h1
    tree.children.forEach((node, index) => {
      if (
        node.type === "heading" &&
        (node as Heading).depth === 1 &&
        !firstH1
      ) {
        firstH1 = node as Heading;
        firstH1Index = index;
      }
    });

    // If no h1 found, skip
    if (!firstH1 || firstH1Index === -1) {
      return;
    }

    // Extract text from h1 (only from the heading node's children)
    const title = extractText(firstH1).trim();

    // Get route path
    const match = vfile?.path?.match(/src\/routes\/(.*?)(?:\/index)?\.mdx?$/);
    const routePath = match ? match[1] : "";

    // Generate metadata
    const og_url = getPageUrl(vfile);
    const description = getPageDescription(routePath);

    // Create PageHeading JSX component attributes
    const attributes: MdxJsxAttribute[] = [
      {
        type: "mdxJsxAttribute",
        name: "title",
        value: title,
      },
      {
        type: "mdxJsxAttribute",
        name: "description",
        value: description,
      },
      {
        type: "mdxJsxAttribute",
        name: "og_url",
        value: og_url,
      },
    ];

    // Create PageHeading JSX component (self-closing)
    const pageHeadingNode: MdxJsxFlowElement = {
      type: "mdxJsxFlowElement",
      name: "PageHeading",
      attributes: attributes,
      children: [],
    };

    // Replace the h1 with PageHeading (keep the original h1)
    // Insert PageHeading before the h1
    tree.children.splice(firstH1Index, 0, pageHeadingNode);
  };
}
