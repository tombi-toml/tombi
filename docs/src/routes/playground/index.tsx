import type { editor as MonacoEditor } from "monaco-editor";
import { FaSolidFeather } from "solid-icons/fa";
import {
  TbAlertTriangle,
  TbCheck,
  TbFileCode,
  TbLoader2,
} from "solid-icons/tb";
import {
  createEffect,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { PageHeading } from "~/components/PageHeading";
import { DEFAULT_URL } from "~/remark/page-heading";
import { loadMonacoEditor, type Monaco } from "~/utils/monaco-editor";
import {
  loadTombiWasm,
  normalizeWasmError,
  type TombiDiagnostic,
  type TombiWasm,
} from "~/utils/tombi-wasm";
import "~/styles/playground.css";

const SAMPLE_SOURCE = `toml-version = "v1.1.0"

[format.rules]
line-width = 100

[lint.rules]
key-empty = "warn"
`;

type RunState = "loading" | "ready" | "linting" | "formatting" | "error";

function diagnosticKey(diagnostic: TombiDiagnostic): string {
  const start = diagnostic.range?.start;
  return [
    diagnostic.level,
    diagnostic.code,
    diagnostic.message,
    start?.line,
    start?.column,
  ].join(":");
}

function uniqueDiagnostics(diagnostics: TombiDiagnostic[]): TombiDiagnostic[] {
  const seen = new Set<string>();
  return diagnostics.filter((diagnostic) => {
    const key = diagnosticKey(diagnostic);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function toMonacoColumn(
  content: string,
  lineNumber: number,
  graphemeColumn: number,
): number {
  const line = content.split(/\r?\n/)[lineNumber] ?? "";
  const segmenter = new Intl.Segmenter(undefined, { granularity: "grapheme" });
  const prefix = Array.from(segmenter.segment(line))
    .slice(0, graphemeColumn)
    .map(({ segment }) => segment)
    .join("");
  return prefix.length + 1;
}

export default function Playground() {
  const [source, setSource] = createSignal(SAMPLE_SOURCE);
  const [fileName, setFileName] = createSignal("tombi.toml");
  const [tomlVersion, setTomlVersion] = createSignal("v1.1.0");
  const [diagnostics, setDiagnostics] = createSignal<TombiDiagnostic[]>([]);
  const [wasm, setWasm] = createSignal<TombiWasm>();
  const [monaco, setMonaco] = createSignal<Monaco>();
  const [codeEditor, setCodeEditor] =
    createSignal<MonacoEditor.IStandaloneCodeEditor>();
  const [editorLoading, setEditorLoading] = createSignal(true);
  const [editorError, setEditorError] = createSignal<string>();
  const [formatShortcut, setFormatShortcut] = createSignal("Ctrl+S");
  const [runState, setRunState] = createSignal<RunState>("loading");
  const [statusMessage, setStatusMessage] = createSignal(
    "Loading the Tombi WebAssembly runtime…",
  );
  let lintSequence = 0;
  let lastLintKey = "";
  let editorContainer: HTMLDivElement | undefined;

  const lintKey = (content: string, filePath: string, version: string) =>
    JSON.stringify([content, filePath, version]);

  const lintSource = async (
    runtime: TombiWasm,
    content: string,
    filePath: string,
    version: string,
    successMessage = "No issues found.",
  ) => {
    const sequence = ++lintSequence;
    const key = lintKey(content, filePath, version);
    let nextDiagnostics: TombiDiagnostic[] = [];
    let fatalError: string | undefined;

    setRunState("linting");
    setStatusMessage("Linting…");

    try {
      await runtime.lint(content, filePath || undefined, version);
    } catch (error) {
      const failure = normalizeWasmError(error);
      nextDiagnostics = uniqueDiagnostics(failure.diagnostics);
      fatalError = failure.error;
    }

    if (sequence !== lintSequence) return;

    lastLintKey = key;
    setDiagnostics(nextDiagnostics);

    if (fatalError) {
      setRunState("error");
      setStatusMessage(fatalError);
      return;
    }

    setRunState("ready");
    setStatusMessage(
      nextDiagnostics.length === 0
        ? successMessage
        : `${nextDiagnostics.length} issue${nextDiagnostics.length === 1 ? "" : "s"} found.`,
    );
  };

  const formatAndLint = async (runtime = wasm()) => {
    if (!runtime || runState() === "formatting") return;

    const sequence = ++lintSequence;
    const currentSource = source();
    const currentFileName = fileName().trim();
    const currentVersion = tomlVersion();
    let formattedSource: string;

    setRunState("formatting");
    setStatusMessage("Formatting…");

    try {
      formattedSource = await runtime.format(
        currentSource,
        currentFileName || undefined,
        currentVersion,
      );
    } catch (error) {
      if (sequence !== lintSequence) return;

      const failure = normalizeWasmError(error);
      const nextDiagnostics = uniqueDiagnostics(failure.diagnostics);
      lastLintKey = lintKey(currentSource, currentFileName, currentVersion);
      setDiagnostics(nextDiagnostics);
      setRunState(failure.error ? "error" : "ready");
      setStatusMessage(
        failure.error ||
          `${nextDiagnostics.length} issue${nextDiagnostics.length === 1 ? "" : "s"} found.`,
      );
      return;
    }

    if (sequence !== lintSequence) return;

    await lintSource(
      runtime,
      formattedSource,
      currentFileName,
      currentVersion,
      "Formatted successfully. No issues found.",
    );

    if (
      lastLintKey === lintKey(formattedSource, currentFileName, currentVersion)
    ) {
      setSource(formattedSource);
    }
  };

  createEffect(() => {
    const runtime = wasm();
    const currentSource = source();
    const currentFileName = fileName().trim();
    const currentVersion = tomlVersion();
    const key = lintKey(currentSource, currentFileName, currentVersion);

    if (!runtime || key === lastLintKey) return;

    const timeout = window.setTimeout(() => {
      if (key !== lastLintKey) {
        void lintSource(
          runtime,
          currentSource,
          currentFileName,
          currentVersion,
        );
      }
    }, 250);

    onCleanup(() => window.clearTimeout(timeout));
  });

  createEffect(() => {
    const editor = codeEditor();
    const currentSource = source();
    const model = editor?.getModel();
    if (!model || model.getValue() === currentSource) return;

    model.pushEditOperations(
      [],
      [{ range: model.getFullModelRange(), text: currentSource }],
      () => null,
    );
  });

  createEffect(() => {
    const monacoApi = monaco();
    const editor = codeEditor();
    const currentDiagnostics = diagnostics();
    const currentSource = source();
    const model = editor?.getModel();
    if (!monacoApi || !editor || !model) return;

    const markers = currentDiagnostics.map((diagnostic) => {
      const start = diagnostic.range?.start ?? { line: 0, column: 0 };
      const end = diagnostic.range?.end ?? start;
      const startLineNumber = start.line + 1;
      const endLineNumber = end.line + 1;
      const startColumn = toMonacoColumn(
        currentSource,
        start.line,
        start.column,
      );
      let endColumn = toMonacoColumn(currentSource, end.line, end.column);

      if (startLineNumber === endLineNumber && endColumn <= startColumn) {
        endColumn = startColumn + 1;
      }

      const range = model.validateRange({
        startLineNumber,
        startColumn,
        endLineNumber,
        endColumn,
      });

      return {
        severity:
          diagnostic.level.toUpperCase() === "WARNING"
            ? monacoApi.MarkerSeverity.Warning
            : monacoApi.MarkerSeverity.Error,
        code: diagnostic.code || undefined,
        message: diagnostic.message,
        source: "Tombi",
        startLineNumber: range.startLineNumber,
        startColumn: range.startColumn,
        endLineNumber: range.endLineNumber,
        endColumn: range.endColumn,
      };
    });

    monacoApi.editor.setModelMarkers(model, "tombi", markers);
  });

  onMount(() => {
    let disposed = false;
    let editor: MonacoEditor.IStandaloneCodeEditor | undefined;
    let model: MonacoEditor.ITextModel | undefined;
    let themeObserver: MutationObserver | undefined;

    onCleanup(() => {
      disposed = true;
      themeObserver?.disconnect();
      editor?.dispose();
      model?.dispose();
    });

    void (async () => {
      try {
        const monacoApi = await loadMonacoEditor();
        if (disposed || !editorContainer) return;

        const updateTheme = () => {
          monacoApi.editor.setTheme(
            document.documentElement.classList.contains("dark")
              ? "vs-dark"
              : "vs",
          );
        };

        model = monacoApi.editor.createModel(source(), "toml");
        const createdEditor = monacoApi.editor.create(editorContainer, {
          model,
          ariaLabel: "TOML editor",
          automaticLayout: true,
          fontFamily:
            '"Fira Code", "SFMono-Regular", Consolas, "Liberation Mono", monospace',
          fontSize: 14,
          folding: false,
          glyphMargin: false,
          hideCursorInOverviewRuler: true,
          lineDecorationsWidth: 8,
          lineNumbers: "on",
          lineNumbersMinChars: 5,
          minimap: { enabled: false },
          overviewRulerBorder: false,
          padding: { top: 16, bottom: 16 },
          renderValidationDecorations: "on",
          scrollBeyondLastLine: false,
          tabSize: 2,
        });
        editor = createdEditor;
        createdEditor.onDidChangeModelContent(() => {
          updateSource(createdEditor.getValue());
        });

        updateTheme();
        themeObserver = new MutationObserver(updateTheme);
        themeObserver.observe(document.documentElement, {
          attributeFilter: ["class"],
          attributes: true,
        });

        setMonaco(monacoApi);
        setCodeEditor(createdEditor);
        setEditorLoading(false);
      } catch (error) {
        if (disposed) return;
        setEditorLoading(false);
        setEditorError(
          error instanceof Error ? error.message : "Unable to load the editor.",
        );
      }
    })();
  });

  onMount(async () => {
    try {
      const runtime = await loadTombiWasm();
      setWasm(runtime);
      await lintSource(runtime, source(), fileName().trim(), tomlVersion());
    } catch (error) {
      const failure = normalizeWasmError(error);
      setRunState("error");
      setStatusMessage(
        failure.error || "Unable to load the Tombi WebAssembly runtime.",
      );
    }
  });

  onMount(() => {
    const isApplePlatform = /Mac|iPhone|iPad|iPod/.test(navigator.platform);
    setFormatShortcut(isApplePlatform ? "⌘S" : "Ctrl+S");

    const handleShortcut = (event: KeyboardEvent) => {
      if (!event.metaKey && !event.ctrlKey) return;

      if (event.key.toLowerCase() === "s") {
        event.preventDefault();
        void formatAndLint();
      }
    };

    window.addEventListener("keydown", handleShortcut, { capture: true });
    onCleanup(() =>
      window.removeEventListener("keydown", handleShortcut, { capture: true }),
    );
  });

  const updateSource = (value: string) => {
    lintSequence += 1;
    setSource(value);
  };

  const goToDiagnostic = (diagnostic: TombiDiagnostic) => {
    const editor = codeEditor();
    const start = diagnostic.range?.start;
    if (!editor || !start) return;

    const position = {
      lineNumber: start.line + 1,
      column: toMonacoColumn(source(), start.line, start.column),
    };
    editor.setPosition(position);
    editor.revealPositionInCenter(position);
    editor.focus();
  };

  return (
    <>
      <PageHeading
        title="TOML Playground | Tombi"
        description="Format and lint TOML instantly with Tombi running locally in your browser."
        og_url={`${DEFAULT_URL}playground`}
      />

      <div class="playground-shell">
        <section class="playground-workspace" aria-label="TOML playground">
          <div class="playground-toolbar">
            <label class="playground-field">
              <span>File name</span>
              <div class="playground-input-wrap">
                <TbFileCode aria-hidden="true" />
                <input
                  type="text"
                  value={fileName()}
                  placeholder="tombi.toml"
                  spellcheck={false}
                  onInput={(event) => {
                    lintSequence += 1;
                    setFileName(event.currentTarget.value);
                  }}
                />
              </div>
            </label>

            <label class="playground-field playground-version-field">
              <span>TOML version</span>
              <select
                value={tomlVersion()}
                onChange={(event) => {
                  lintSequence += 1;
                  setTomlVersion(event.currentTarget.value);
                }}
              >
                <option value="v1.1.0">TOML v1.1.0</option>
                <option value="v1.0.0">TOML v1.0.0</option>
                <option value="v1.1.0-preview">TOML v1.1 preview</option>
              </select>
            </label>

            <div class="playground-actions">
              <button
                type="button"
                class="playground-button playground-button-primary"
                title={`Format (${formatShortcut()})`}
                aria-keyshortcuts={
                  formatShortcut() === "⌘S" ? "Meta+S" : "Control+S"
                }
                onClick={() => void formatAndLint()}
                disabled={!wasm() || runState() === "formatting"}
              >
                <Show
                  when={runState() !== "formatting"}
                  fallback={
                    <TbLoader2 class="playground-spinner" aria-hidden="true" />
                  }
                >
                  <FaSolidFeather aria-hidden="true" />
                </Show>
                {runState() === "formatting" ? "Formatting…" : "Format"}
              </button>
            </div>
          </div>

          <section class="playground-editor-card">
            <div
              class="playground-editor"
              classList={{
                "is-loading": editorLoading() || Boolean(editorError()),
              }}
              ref={editorContainer}
            >
              <Show when={editorLoading()}>
                <div class="playground-editor-loading">
                  <TbLoader2 class="playground-spinner" aria-hidden="true" />
                  Loading editor…
                </div>
              </Show>
              <Show when={editorError()}>
                {(message) => (
                  <div class="playground-editor-error" role="alert">
                    <TbAlertTriangle aria-hidden="true" />
                    {message()}
                  </div>
                )}
              </Show>
            </div>
          </section>

          <section class="playground-results" aria-live="polite">
            <div
              class="playground-status"
              data-state={runState()}
              data-has-diagnostics={diagnostics().length > 0}
            >
              <Show
                when={runState() === "ready" || runState() === "error"}
                fallback={
                  <TbLoader2 class="playground-spinner" aria-hidden="true" />
                }
              >
                <Show
                  when={runState() !== "error" && diagnostics().length === 0}
                  fallback={<TbAlertTriangle aria-hidden="true" />}
                >
                  <TbCheck aria-hidden="true" />
                </Show>
              </Show>
              <span>{statusMessage()}</span>
            </div>

            <Show when={diagnostics().length > 0}>
              <div class="playground-diagnostics">
                <For each={diagnostics()}>
                  {(diagnostic) => {
                    const line = (diagnostic.range?.start.line ?? 0) + 1;
                    const column = (diagnostic.range?.start.column ?? 0) + 1;
                    return (
                      <button
                        type="button"
                        data-level={diagnostic.level?.toLowerCase()}
                        onClick={() => goToDiagnostic(diagnostic)}
                      >
                        <div class="playground-diagnostic-meta">
                          <span>{diagnostic.level || "ERROR"}</span>
                          <code>{diagnostic.code || "syntax"}</code>
                          <span>
                            Line {line}, column {column}
                          </span>
                        </div>
                        <p>{diagnostic.message}</p>
                      </button>
                    );
                  }}
                </For>
              </div>
            </Show>
          </section>
        </section>
      </div>
    </>
  );
}
