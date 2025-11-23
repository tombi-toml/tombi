import { A } from "@solidjs/router";
import { For, Show } from "solid-js";
import type { SearchResult } from "~/utils/search";

interface SearchResultsProps {
  results: SearchResult[];
}

function HighlightedText(props: { text: string; matches: [number, number][] }) {
  const segments: { text: string; isHighlight: boolean }[] = [];
  let lastIndex = 0;

  // Sort matches and resolve overlaps
  const sortedMatches = [...props.matches].sort((a, b) => a[0] - b[0]);
  const mergedMatches: [number, number][] = [];

  for (const match of sortedMatches) {
    if (
      mergedMatches.length === 0 ||
      match[0] > mergedMatches[mergedMatches.length - 1][1]
    ) {
      mergedMatches.push(match);
    } else {
      mergedMatches[mergedMatches.length - 1][1] = Math.max(
        mergedMatches[mergedMatches.length - 1][1],
        match[1],
      );
    }
  }

  // Create segments
  for (const [start, end] of mergedMatches) {
    if (start > lastIndex) {
      segments.push({
        text: props.text.slice(lastIndex, start),
        isHighlight: false,
      });
    }
    segments.push({
      text: props.text.slice(start, end),
      isHighlight: true,
    });
    lastIndex = end;
  }

  if (lastIndex < props.text.length) {
    segments.push({
      text: props.text.slice(lastIndex),
      isHighlight: false,
    });
  }

  return (
    <span>
      <For each={segments}>
        {(segment) => (
          <span
            class={
              segment.isHighlight
                ? "bg-yellow-200 dark:bg-yellow-800 rounded px-1"
                : ""
            }
          >
            {segment.text}
          </span>
        )}
      </For>
    </span>
  );
}

export function SearchResults(props: SearchResultsProps) {
  return (
    <Show when={props.results.length > 0}>
      <div
        class="fixed left-0 right-0 mt-2 mx-auto max-w-150 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl max-h-[80vh] overflow-y-auto z-[60]"
        style={{ top: "calc(5rem + 4px)" }}
      >
        <div class="p-4">
          <For each={props.results}>
            {(result) => (
              <A
                href={result.url}
                class="block p-4 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg mb-2 transition-colors border border-transparent dark:border-gray-800"
              >
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">
                  <HighlightedText
                    text={result.highlight.title.text}
                    matches={result.highlight.title.matches}
                  />
                </h3>
                <p class="text-gray-600 dark:text-gray-300">
                  <HighlightedText
                    text={result.highlight.content.text}
                    matches={result.highlight.content.matches}
                  />
                </p>
              </A>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
