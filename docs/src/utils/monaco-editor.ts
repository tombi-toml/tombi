export type Monaco = typeof import("monaco-editor/editor");

let monacoPromise: Promise<Monaco> | undefined;
let tomlRegistered = false;

export function loadMonacoEditor(): Promise<Monaco> {
  if (monacoPromise) return monacoPromise;

  monacoPromise = (async () => {
    const { default: EditorWorker } = await import(
      "monaco-editor/editor/editor.worker?worker&inline"
    );

    (
      globalThis as typeof globalThis & {
        MonacoEnvironment?: {
          getWorker: () => Worker;
        };
      }
    ).MonacoEnvironment = {
      getWorker: () => new EditorWorker(),
    };

    const monaco = await import("monaco-editor/editor");

    if (!tomlRegistered) {
      monaco.languages.register({ id: "toml" });
      monaco.languages.setLanguageConfiguration("toml", {
        comments: { lineComment: "#" },
        brackets: [
          ["[", "]"],
          ["{", "}"],
        ],
        autoClosingPairs: [
          { open: "[", close: "]" },
          { open: "{", close: "}" },
          { open: '"', close: '"' },
          { open: "'", close: "'" },
        ],
        surroundingPairs: [
          { open: "[", close: "]" },
          { open: "{", close: "}" },
          { open: '"', close: '"' },
          { open: "'", close: "'" },
        ],
      });
      monaco.languages.setMonarchTokensProvider("toml", {
        tokenizer: {
          root: [
            [/\s+/, "white"],
            [/#.*$/, "comment"],
            [/\[\[.*?\]\]|\[.*?\]/, "type.identifier"],
            [/"""/, { token: "string.quote", next: "@multilineBasic" }],
            [/'''/, { token: "string.quote", next: "@multilineLiteral" }],
            [/"([^"\\]|\\.)*"/, "string"],
            [/'[^']*'/, "string"],
            [/\b(?:true|false)\b/, "keyword"],
            [
              /\b\d{4}-\d{2}-\d{2}(?:[Tt ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:[Zz]|[+-]\d{2}:\d{2})?)?\b/,
              "number",
            ],
            [
              /\b(?:inf|nan|[+-]?(?:0x[0-9a-fA-F_]+|0o[0-7_]+|0b[01_]+|\d[\d_]*(?:\.\d[\d_]*)?(?:[eE][+-]?\d[\d_]*)?))\b/,
              "number",
            ],
            [/[A-Za-z0-9_-]+(?=\s*=)/, "variable.name"],
            [/[=,.]/, "delimiter"],
          ],
          multilineBasic: [
            [/"""/, { token: "string.quote", next: "@pop" }],
            [/\\./, "string.escape"],
            [/./, "string"],
          ],
          multilineLiteral: [
            [/'''/, { token: "string.quote", next: "@pop" }],
            [/./, "string"],
          ],
        },
      });
      tomlRegistered = true;
    }

    return monaco;
  })().catch((error) => {
    monacoPromise = undefined;
    throw error;
  });

  return monacoPromise;
}
