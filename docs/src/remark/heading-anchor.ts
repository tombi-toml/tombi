import type { Root, Heading } from "mdast";
import { visit } from "unist-util-visit";

// Function to convert text to URL-safe slugs
function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/\s+/g, "-") // Replace spaces with hyphens
    .replace(/[^\w-]/g, "") // Remove non-alphanumeric characters except hyphens
    .replace(/-+/g, "-") // Replace consecutive hyphens with a single hyphen
    .replace(/^-+|-+$/g, ""); // Remove hyphens at the beginning and end
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
      let headingText = "";
      visit(node, "text", (textNode) => {
        headingText += textNode.value;
      });

      // Generate slug
      const slug = slugify(headingText);

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
