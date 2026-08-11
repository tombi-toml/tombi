import type {
  editor,
  IDisposable,
  IPosition,
  IRange,
  languages,
} from "monaco-editor";
import type { Monaco } from "~/utils/monaco-editor";
import type {
  LspDiagnostic,
  LspLocation,
  LspLocationLink,
  LspMarkupContent,
  LspPosition,
  LspRange,
  LspTextEdit,
  TombiLspClient,
} from "~/utils/tombi-lsp";

interface LspCompletionItem {
  label: string | { label: string; detail?: string; description?: string };
  kind?: number;
  detail?: string;
  documentation?: string | LspMarkupContent;
  filterText?: string;
  insertText?: string;
  insertTextFormat?: number;
  sortText?: string;
  textEdit?: LspTextEdit;
  additionalTextEdits?: LspTextEdit[];
}

interface LspCompletionList {
  isIncomplete?: boolean;
  items: LspCompletionItem[];
}

interface LspHover {
  contents: string | LspMarkupContent | Array<string | LspMarkupContent>;
  range?: LspRange;
}

interface LspDocumentLink {
  range: LspRange;
  target?: string;
  tooltip?: string;
}

interface LspFoldingRange {
  startLine: number;
  endLine: number;
  kind?: string;
}

interface LspInlayHint {
  position: LspPosition;
  label: string | Array<{ value: string; tooltip?: string | LspMarkupContent }>;
  kind?: number;
  paddingLeft?: boolean;
  paddingRight?: boolean;
  textEdits?: LspTextEdit[];
  tooltip?: string | LspMarkupContent;
}

interface LspDocumentSymbol {
  name: string;
  detail?: string;
  kind: number;
  range: LspRange;
  selectionRange: LspRange;
  children?: LspDocumentSymbol[];
}

interface LspCodeAction {
  title: string;
  kind?: string;
  isPreferred?: boolean;
  disabled?: { reason: string };
  diagnostics?: LspDiagnostic[];
  edit?: {
    changes?: Record<string, LspTextEdit[]>;
    documentChanges?: Array<{
      textDocument?: { uri: string; version?: number | null };
      edits?: LspTextEdit[];
    }>;
  };
}

interface LspSemanticTokens {
  resultId?: string;
  data: number[];
}

const position = (value: IPosition): LspPosition => ({
  character: value.column - 1,
  line: value.lineNumber - 1,
});

const range = (value: LspRange): IRange => ({
  endColumn: value.end.character + 1,
  endLineNumber: value.end.line + 1,
  startColumn: value.start.character + 1,
  startLineNumber: value.start.line + 1,
});

const lspRange = (value: IRange): LspRange => ({
  end: { character: value.endColumn - 1, line: value.endLineNumber - 1 },
  start: {
    character: value.startColumn - 1,
    line: value.startLineNumber - 1,
  },
});

const markdown = (value: string | LspMarkupContent) => ({
  value: typeof value === "string" ? value : value.value,
});

const textEdit = (value: LspTextEdit): languages.TextEdit => ({
  range: range(value.range),
  text: value.newText,
});

const completionKind = (monaco: Monaco, kind = 1) => {
  const values = monaco.languages.CompletionItemKind;
  return (
    [
      values.Text,
      values.Method,
      values.Function,
      values.Constructor,
      values.Field,
      values.Variable,
      values.Class,
      values.Interface,
      values.Module,
      values.Property,
      values.Unit,
      values.Value,
      values.Enum,
      values.Keyword,
      values.Snippet,
      values.Color,
      values.File,
      values.Reference,
      values.Folder,
      values.EnumMember,
      values.Constant,
      values.Struct,
      values.Event,
      values.Operator,
      values.TypeParameter,
    ][kind - 1] ?? values.Text
  );
};

const documentParams = (model: editor.ITextModel) => ({
  textDocument: { uri: model.uri.toString() },
});

const positionParams = (model: editor.ITextModel, value: IPosition) => ({
  ...documentParams(model),
  position: position(value),
});

function locations(
  monaco: Monaco,
  response:
    | LspLocation
    | LspLocationLink
    | Array<LspLocation | LspLocationLink>
    | null,
): languages.LocationLink[] {
  const values =
    response === null ? [] : Array.isArray(response) ? response : [response];
  return values.map((value) => {
    if ("targetUri" in value) {
      return {
        originSelectionRange: value.originSelectionRange
          ? range(value.originSelectionRange)
          : undefined,
        range: range(value.targetRange),
        targetSelectionRange: range(value.targetSelectionRange),
        uri: monaco.Uri.parse(value.targetUri),
      };
    }
    return {
      range: range(value.range),
      targetSelectionRange: range(value.range),
      uri: monaco.Uri.parse(value.uri),
    };
  });
}

function diagnostic(value: editor.IMarkerData): LspDiagnostic {
  return {
    code: typeof value.code === "string" ? value.code : value.code?.value,
    message: value.message,
    range: {
      end: { character: value.endColumn - 1, line: value.endLineNumber - 1 },
      start: {
        character: value.startColumn - 1,
        line: value.startLineNumber - 1,
      },
    },
    severity:
      value.severity === 8
        ? 1
        : value.severity === 4
          ? 2
          : value.severity === 2
            ? 3
            : 4,
    source: value.source,
  };
}

export function registerTombiLspProviders(
  monaco: Monaco,
  client: TombiLspClient,
): IDisposable {
  const selector = "toml";
  const disposables: IDisposable[] = [];

  disposables.push(
    monaco.languages.registerCompletionItemProvider(selector, {
      triggerCharacters: [".", ",", "=", ":", "[", "{", " ", '"', "'"],
      async provideCompletionItems(model, cursor, context) {
        const response = await client.request<
          LspCompletionList | LspCompletionItem[] | null
        >("textDocument/completion", {
          ...positionParams(model, cursor),
          context: {
            triggerCharacter: context.triggerCharacter,
            triggerKind: context.triggerKind + 1,
          },
        });
        const items = Array.isArray(response)
          ? response
          : (response?.items ?? []);
        return {
          incomplete: !Array.isArray(response) && response?.isIncomplete,
          suggestions: items.map((item) => {
            const label =
              typeof item.label === "string" ? item.label : item.label.label;
            const replacement = item.textEdit?.range
              ? range(item.textEdit.range)
              : model.getWordUntilPosition(cursor);
            return {
              additionalTextEdits: item.additionalTextEdits?.map((edit) => ({
                range: range(edit.range),
                text: edit.newText,
              })),
              detail: item.detail,
              documentation: item.documentation
                ? markdown(item.documentation)
                : undefined,
              filterText: item.filterText,
              insertText: item.textEdit?.newText ?? item.insertText ?? label,
              insertTextRules:
                item.insertTextFormat === 2
                  ? monaco.languages.CompletionItemInsertTextRule
                      .InsertAsSnippet
                  : undefined,
              kind: completionKind(monaco, item.kind),
              label: item.label,
              range:
                "startLineNumber" in replacement
                  ? replacement
                  : {
                      endColumn: replacement.endColumn,
                      endLineNumber: cursor.lineNumber,
                      startColumn: replacement.startColumn,
                      startLineNumber: cursor.lineNumber,
                    },
              sortText: item.sortText,
            };
          }),
        };
      },
    }),
    monaco.languages.registerHoverProvider(selector, {
      async provideHover(model, cursor) {
        const response = await client.request<LspHover | null>(
          "textDocument/hover",
          positionParams(model, cursor),
        );
        if (!response) return null;
        const contents = Array.isArray(response.contents)
          ? response.contents
          : [response.contents];
        return {
          contents: contents.map(markdown),
          range: response.range ? range(response.range) : undefined,
        };
      },
    }),
    monaco.languages.registerDocumentFormattingEditProvider(selector, {
      displayName: "Tombi",
      async provideDocumentFormattingEdits(model, options) {
        const response = await client.request<LspTextEdit[] | null>(
          "textDocument/formatting",
          {
            ...documentParams(model),
            options,
          },
        );
        return (response ?? []).map(textEdit);
      },
    }),
  );

  const registerLocation = (
    register: (provider: languages.DefinitionProvider) => IDisposable,
    method: string,
  ) =>
    register({
      async provideDefinition(model, cursor) {
        const response = await client.request<
          | LspLocation
          | LspLocationLink
          | Array<LspLocation | LspLocationLink>
          | null
        >(method, positionParams(model, cursor));
        return locations(monaco, response);
      },
    });

  disposables.push(
    registerLocation(
      (provider) =>
        monaco.languages.registerDefinitionProvider(selector, provider),
      "textDocument/definition",
    ),
    registerLocation(
      (provider) =>
        monaco.languages.registerDeclarationProvider(selector, {
          provideDeclaration: provider.provideDefinition,
        }),
      "textDocument/declaration",
    ),
    registerLocation(
      (provider) =>
        monaco.languages.registerTypeDefinitionProvider(selector, {
          provideTypeDefinition: provider.provideDefinition,
        }),
      "textDocument/typeDefinition",
    ),
    monaco.languages.registerReferenceProvider(selector, {
      async provideReferences(model, cursor, context) {
        const response = await client.request<LspLocation[] | null>(
          "textDocument/references",
          {
            ...positionParams(model, cursor),
            context: { includeDeclaration: context.includeDeclaration },
          },
        );
        return locations(monaco, response);
      },
    }),
    monaco.languages.registerLinkProvider(selector, {
      async provideLinks(model) {
        const response = await client.request<LspDocumentLink[] | null>(
          "textDocument/documentLink",
          documentParams(model),
        );
        return {
          links: (response ?? []).map((link) => ({
            range: range(link.range),
            tooltip: link.tooltip,
            url: link.target,
          })),
        };
      },
    }),
    monaco.languages.registerFoldingRangeProvider(selector, {
      async provideFoldingRanges(model) {
        const response = await client.request<LspFoldingRange[] | null>(
          "textDocument/foldingRange",
          documentParams(model),
        );
        return (response ?? []).map((item) => ({
          end: item.endLine + 1,
          kind: item.kind
            ? new monaco.languages.FoldingRangeKind(item.kind)
            : undefined,
          start: item.startLine + 1,
        }));
      },
    }),
    monaco.languages.registerInlayHintsProvider(selector, {
      async provideInlayHints(model, visibleRange) {
        const response = await client.request<LspInlayHint[] | null>(
          "textDocument/inlayHint",
          { ...documentParams(model), range: lspRange(visibleRange) },
        );
        return {
          dispose() {},
          hints: (response ?? []).map((hint) => ({
            kind: hint.kind,
            label:
              typeof hint.label === "string"
                ? hint.label
                : hint.label.map((part) => ({
                    label: part.value,
                    tooltip: part.tooltip ? markdown(part.tooltip) : undefined,
                  })),
            paddingLeft: hint.paddingLeft,
            paddingRight: hint.paddingRight,
            position: {
              column: hint.position.character + 1,
              lineNumber: hint.position.line + 1,
            },
            textEdits: hint.textEdits?.map(textEdit),
            tooltip: hint.tooltip ? markdown(hint.tooltip) : undefined,
          })),
        };
      },
    }),
    monaco.languages.registerDocumentSymbolProvider(selector, {
      async provideDocumentSymbols(model) {
        const response = await client.request<LspDocumentSymbol[] | null>(
          "textDocument/documentSymbol",
          documentParams(model),
        );
        const convert = (
          symbol: LspDocumentSymbol,
        ): languages.DocumentSymbol => ({
          children: symbol.children?.map(convert),
          detail: symbol.detail ?? "",
          kind: Math.max(0, symbol.kind - 1),
          name: symbol.name,
          range: range(symbol.range),
          selectionRange: range(symbol.selectionRange),
          tags: [],
        });
        return (response ?? []).map(convert);
      },
    }),
    monaco.languages.registerCodeActionProvider(selector, {
      async provideCodeActions(model, selectedRange, context) {
        const response = await client.request<LspCodeAction[] | null>(
          "textDocument/codeAction",
          {
            ...documentParams(model),
            context: {
              diagnostics: context.markers.map(diagnostic),
              only: context.only ? [context.only] : undefined,
              triggerKind: context.trigger,
            },
            range: lspRange(selectedRange),
          },
        );
        return {
          actions: (response ?? []).map((action) => ({
            disabled: action.disabled?.reason,
            edit: action.edit
              ? {
                  edits: [
                    ...Object.entries(action.edit.changes ?? {}).flatMap(
                      ([uri, edits]) =>
                        edits.map((edit) => ({
                          resource: monaco.Uri.parse(uri),
                          textEdit: textEdit(edit),
                          versionId: undefined,
                        })),
                    ),
                    ...(action.edit.documentChanges ?? []).flatMap((change) =>
                      change.textDocument
                        ? (change.edits ?? []).map((edit) => ({
                            resource: monaco.Uri.parse(
                              change.textDocument?.uri ?? "",
                            ),
                            textEdit: textEdit(edit),
                            versionId:
                              change.textDocument?.version ?? undefined,
                          }))
                        : [],
                    ),
                  ],
                }
              : undefined,
            isPreferred: action.isPreferred,
            kind: action.kind,
            title: action.title,
          })),
          dispose() {},
        };
      },
    }),
  );

  disposables.push(
    monaco.languages.registerDocumentSemanticTokensProvider(selector, {
      getLegend: () => ({
        tokenModifiers: [],
        tokenTypes: [
          "string",
          "number",
          "operator",
          "comment",
          "keyword",
          "table",
          "key",
          "boolean",
          "offsetDateTime",
          "localDateTime",
          "localDate",
          "localTime",
        ],
      }),
      async provideDocumentSemanticTokens(model) {
        const response = await client.request<LspSemanticTokens | null>(
          "textDocument/semanticTokens/full",
          documentParams(model),
        );
        return response
          ? {
              data: new Uint32Array(response.data),
              resultId: response.resultId,
            }
          : null;
      },
      releaseDocumentSemanticTokens() {},
    }),
  );

  return {
    dispose() {
      for (const disposable of disposables) disposable.dispose();
    },
  };
}
