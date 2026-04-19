import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
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

function collectMdxFiles(dir: string, baseDir = dir): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = join(dir, entry.name);

    if (entry.isDirectory()) {
      return collectMdxFiles(fullPath, baseDir);
    }

    if (!entry.isFile() || !entry.name.endsWith(".mdx")) {
      return [];
    }

    return [fullPath.slice(baseDir.length + 1)];
  });
}

function generateSearchIndex() {
  const docsDir = join(process.cwd(), "src/routes/docs");
  const files = collectMdxFiles(docsDir);

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
