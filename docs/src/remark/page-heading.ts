import type { Root } from "mdast";
import type { MdxJsxAttribute, MdxJsxFlowElement } from "mdast-util-mdx-jsx";
import docIndex from "../../doc-index.json";

export const DEFAULT_TITLE = "Tombi";
export const DEFAULT_DESCRIPTION =
  "a TOML toolkit that provides Formatter, Linter, and Language Server.";
export const DEFAULT_URL = "https://tombi-toml.github.io/tombi/";

interface DocIndexItem {
  title: string;
  description: string;
  path: string;
  children?: DocIndexItem[];
}

function getTitle(routePath: string): string {
  return (
    findTitleAndDescriptionDocIndex(`/${routePath}`, docIndex as DocIndexItem[])
      ?.title || DEFAULT_TITLE
  );
}

// Get description from doc-index.json
function getPageDescription(routePath: string): string {
  return (
    findTitleAndDescriptionDocIndex(`/${routePath}`, docIndex as DocIndexItem[])
      ?.description || DEFAULT_DESCRIPTION
  );
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
function findTitleAndDescriptionDocIndex(
  path: string,
  items: DocIndexItem[],
): { title: string; description: string } | undefined {
  for (const item of items) {
    if (item.path === path) {
      return { title: item.title, description: item.description || "" };
    }
    if (item.children) {
      const found = findTitleAndDescriptionDocIndex(path, item.children);
      if (found) return found;
    }
  }
  return undefined;
}

export function remarkPageHeading() {
  return (tree: Root, vfile: { path?: string }) => {
    // Get route path
    const match = vfile?.path?.match(/src\/routes\/(.*?)(?:\/index)?\.mdx?$/);
    const routePath = match ? match[1] : "";

    // Extract text from h1 (only from the heading node's children)
    const title = getTitle(routePath).trim();
    const description = getPageDescription(routePath);

    // Generate metadata
    const og_url = getPageUrl(vfile);

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
    tree.children.splice(0, 0, pageHeadingNode);
  };
}
