import type { Heading, Root, RootContent } from "mdast";
import { visit } from "unist-util-visit";

// Function to convert text to URL-safe slugs
function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/\./g, "-") // Replace dots with hyphens
    .replace(/\s+/g, "-") // Replace spaces with hyphens
    .replace(/[^a-z0-9-]/g, "") // Remove non-alphanumeric characters except hyphens
    .replace(/-+/g, "-") // Replace consecutive hyphens with a single hyphen
    .replace(/^-+|-+$/g, ""); // Remove hyphens at the beginning and end
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

  return "";
}

interface NodeData {
  id?: string;
  hProperties?: Record<string, string | number | boolean>;
  [key: string]: unknown;
}

export function remarkHeadingAnchor() {
  return (tree: Root) => {
    visit(tree, "heading", (node: Heading) => {
      // Extract text content
      const headingText = extractText(node);

      // Generate slug
      const slug = slugify(headingText);

      if (!slug) {
        return;
      }

      // Add ID to data properties
      if (!node.data) node.data = {};
      const data = node.data as NodeData;

      // Add id property
      data.id = slug;

      // Set up hProperties (for HTML rendering)
      if (!data.hProperties) data.hProperties = {};
      data.hProperties.id = slug;
      data.hProperties.className = "group relative heading-with-anchor";
    });
  };
}
