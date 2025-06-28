import FlexSearch from "flexsearch";
import searchIndex from "../search-index.json";

export interface SearchResult {
  title: string;
  content: string;
  url: string;
  score: number;
  highlight: {
    title: {
      text: string;
      matches: [number, number][];
    };
    content: {
      text: string;
      matches: [number, number][];
    };
  };
}

interface DocumentIndex {
  id: number;
  title: string;
  content: string;
  url: string;
  score?: number;
}

// Create FlexSearch instance
const index = new FlexSearch.Document<DocumentIndex, true>({
  document: {
    id: "id",
    index: ["title", "content"] as const,
    store: true,
  },
  tokenize: "forward",
  context: true,
});

// Add indexes when initialized
for (const doc of searchIndex) {
  index.add(doc as DocumentIndex);
}

// Find matches for highlighting
function findMatches(
  text: string,
  query: string,
  offset = 0,
): [number, number][] {
  const matches: [number, number][] = [];
  const words = query.toLowerCase().split(/\s+/).filter(Boolean); // Remove empty strings
  const lowerText = text.toLowerCase();

  for (const word of words) {
    let pos = 0;
    let nextPos = lowerText.indexOf(word, pos);
    while (nextPos !== -1) {
      matches.push([nextPos + offset, nextPos + word.length + offset]);
      pos = nextPos + word.length; // Move past the current match
      nextPos = lowerText.indexOf(word, pos);
    }
  }

  return matches.sort((a, b) => a[0] - b[0]);
}

// Extract context with highlighting
function extractContext(
  text: string,
  query: string,
  contextLength = 100,
): { content: string; matches: [number, number][] } {
  const words = query.toLowerCase().split(/\s+/);

  // Find the first match
  let firstMatch = -1;
  for (const word of words) {
    const index = text.toLowerCase().indexOf(word);
    if (index !== -1 && (firstMatch === -1 || index < firstMatch)) {
      firstMatch = index;
    }
  }

  if (firstMatch === -1) {
    return { content: text.slice(0, contextLength), matches: [] };
  }

  // Calculate the context range
  const start = Math.max(0, firstMatch - contextLength / 2);
  const end = Math.min(text.length, firstMatch + contextLength / 2);
  let content = text.slice(start, end);

  // Don't add ellipsis if contextLength is equal to the original text length
  const isFullText = contextLength >= text.length;
  if (!isFullText) {
    // Add ellipsis
    if (start > 0) content = `...${content}`;
    if (end < text.length) content = `${content}...`;
  }

  // Find matches in the extracted context
  const matches = findMatches(content, query, 0);

  return { content, matches };
}

export async function searchDocumentation(
  query: string,
): Promise<SearchResult[]> {
  if (!query.trim()) {
    return [];
  }

  try {
    // Search both title and content
    const titleResults = await index.searchAsync(query, {
      index: "title",
      limit: 5,
      enrich: true,
    });
    const contentResults = await index.searchAsync(query, {
      index: "content",
      limit: 5,
      enrich: true,
    });

    // Merge results and remove duplicates
    const resultMap = new Map<number, SearchResult>();

    // Calculate scores and merge
    for (const result of [...titleResults, ...contentResults]) {
      if (result.result) {
        for (const docResult of result.result) {
          const doc = searchIndex.find((d) => d.id === Number(docResult.id));
          if (doc && !resultMap.has(doc.id)) {
            resultMap.set(doc.id, {
              title: doc.title,
              content: doc.content,
              url: doc.url,
              score: 0,
              highlight: {
                title: {
                  text: doc.title,
                  matches: [],
                },
                content: {
                  text: doc.content,
                  matches: [],
                },
              },
            });
          }
        }
      }
    }

    // Sort by score and return
    const results = Array.from(resultMap.values());
    for (const result of results) {
      // Find matches for title (without context extraction)
      result.highlight.title.text = result.title;
      result.highlight.title.matches = findMatches(result.title, query);

      // Extract highlight information for content
      const contentHighlight = extractContext(result.content, query);
      result.highlight.content.text = contentHighlight.content;
      result.highlight.content.matches = contentHighlight.matches;
    }
    return results.sort((a, b) => b.score - a.score);
  } catch (error) {
    console.error("Error during search:", error);
    return [];
  }
}
