import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { globSync } from "glob";
import matter from "gray-matter";

interface DocumentData {
  id: number;
  title: string;
  content: string;
  url: string;
}

function extractTextContent(markdown: string): string {
  return markdown
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1") // Link
    .replace(/[#*`]/g, "") // Headings, bold, code
    .replace(/\n+/g, " ") // Newline
    .trim();
}

function generateSearchIndex() {
  const docsDir = join(process.cwd(), "src/routes/docs");
  const files = globSync("**/*.mdx", { cwd: docsDir });

  const documents: DocumentData[] = files.map((file: string, index: number) => {
    const fullPath = join(docsDir, file);
    const fileContent = readFileSync(fullPath, "utf-8");
    const { data, content } = matter(fileContent);

    const url = `/docs/${file.replace(/\.mdx$/, "").replace(/\/index$/, "")}`;

    return {
      id: index + 1,
      title: (data as { title?: string }).title ?? url,
      content: extractTextContent(content),
      url,
    };
  });

  const outputPath = join(process.cwd(), "src/search-index.json");
  writeFileSync(outputPath, JSON.stringify(documents, null, 2));

  console.log(`Generated search index: ${outputPath}`);
  console.log(`Total documents: ${documents.length}`);
}

generateSearchIndex();
