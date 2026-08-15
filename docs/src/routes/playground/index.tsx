import type { editor as MonacoEditor } from "monaco-editor";
import { FaSolidFeather } from "solid-icons/fa";
import {
  TbAlertTriangle,
  TbCheck,
  TbChevronDown,
  TbChevronRight,
  TbFile,
  TbFilePlus,
  TbFolder,
  TbFolderPlus,
  TbLayoutSidebarLeftCollapse,
  TbLayoutSidebarLeftExpand,
  TbLoader2,
} from "solid-icons/tb";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { PageHeading } from "~/components/PageHeading";
import { DEFAULT_URL } from "~/remark/page-heading";
import { loadMonacoEditor, type Monaco } from "~/utils/monaco-editor";
import { detectOperatingSystem } from "~/utils/platform";
import { registerTombiLspProviders } from "~/utils/tombi-monaco";
import {
  type LspDiagnostic,
  TombiLspClient,
  workspaceFileUri,
} from "~/utils/tombi-lsp";
import "~/styles/playground.css";

const DEFAULT_PATH = "tombi.toml";
const DEFAULT_SOURCE = `toml-version = "v1.1.0"

[format.rules]
line-width = 100

[lint.rules]
key-empty = "warn"
`;

interface WorkspaceEntry {
  kind: "file" | "directory";
  path: string;
  text: string;
  version: number;
}

interface TreeItem extends WorkspaceEntry {
  depth: number;
  name: string;
}

interface TreeContextMenu {
  path?: string;
  x: number;
  y: number;
}

type RunState = "loading" | "ready" | "linting" | "formatting" | "error";

function parentPath(path: string): string {
  const separator = path.lastIndexOf("/");
  return separator < 0 ? "" : path.slice(0, separator);
}

function baseName(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

function isPathWithin(path: string, parent: string): boolean {
  return path === parent || path.startsWith(`${parent}/`);
}

function replacePathPrefix(
  path: string,
  previousPath: string,
  nextPath: string,
): string {
  return path === previousPath
    ? nextPath
    : `${nextPath}${path.slice(previousPath.length)}`;
}

function normalizeWorkspacePath(input: string): string | undefined {
  const rawSegments = input
    .trim()
    .replaceAll("\\", "/")
    .split("/")
    .filter(Boolean);
  if (rawSegments[0] === "workspace") rawSegments.shift();
  const segments: string[] = [];
  for (const segment of rawSegments) {
    if (segment === ".") continue;
    if (segment === "..") return undefined;
    segments.push(segment);
  }
  return segments.length > 0 ? segments.join("/") : undefined;
}

function withParentDirectories(entries: WorkspaceEntry[]): WorkspaceEntry[] {
  const byPath = new Map(entries.map((entry) => [entry.path, entry]));
  for (const entry of entries) {
    let parent = parentPath(entry.path);
    while (parent) {
      if (!byPath.has(parent)) {
        byPath.set(parent, {
          kind: "directory",
          path: parent,
          text: "",
          version: 0,
        });
      }
      parent = parentPath(parent);
    }
  }
  return [...byPath.values()];
}

function markerSeverity(monaco: Monaco, diagnostic: LspDiagnostic): number {
  switch (diagnostic.severity) {
    case 2:
      return monaco.MarkerSeverity.Warning;
    case 3:
      return monaco.MarkerSeverity.Info;
    case 4:
      return monaco.MarkerSeverity.Hint;
    default:
      return monaco.MarkerSeverity.Error;
  }
}

export default function Playground() {
  const [entries, setEntries] = createSignal<WorkspaceEntry[]>([
    {
      kind: "file",
      path: DEFAULT_PATH,
      text: DEFAULT_SOURCE,
      version: 1,
    },
  ]);
  const [activePath, setActivePath] = createSignal(DEFAULT_PATH);
  const [expandedPaths, setExpandedPaths] = createSignal(new Set<string>());
  const [diagnostics, setDiagnostics] = createSignal<LspDiagnostic[]>([]);
  const [lsp, setLsp] = createSignal<TombiLspClient>();
  const [monaco, setMonaco] = createSignal<Monaco>();
  const [codeEditor, setCodeEditor] =
    createSignal<MonacoEditor.IStandaloneCodeEditor>();
  const [isMounted, setIsMounted] = createSignal(false);
  const [editorLoading, setEditorLoading] = createSignal(true);
  const [editorError, setEditorError] = createSignal<string>();
  const [formatShortcut, setFormatShortcut] = createSignal("Ctrl S");
  const [isExplorerOpen, setIsExplorerOpen] = createSignal(true);
  const [selectedDirectoryPath, setSelectedDirectoryPath] =
    createSignal<string>();
  const [creatingEntryKind, setCreatingEntryKind] = createSignal<
    "file" | "directory"
  >();
  const [renamingPath, setRenamingPath] = createSignal<string>();
  const [treeContextMenu, setTreeContextMenu] = createSignal<TreeContextMenu>();
  const [runState, setRunState] = createSignal<RunState>("loading");
  const [statusMessage, setStatusMessage] = createSignal(
    "Downloading tombi-wasm-lsp…",
  );
  const models = new Map<string, MonacoEditor.ITextModel>();
  let diagnosticSequence = 0;
  let editorContainer: HTMLDivElement | undefined;

  const activeEntry = () =>
    entries().find(
      (entry) => entry.kind === "file" && entry.path === activePath(),
    );

  const treeItems = createMemo<TreeItem[]>(() => {
    const children = new Map<string, WorkspaceEntry[]>();
    for (const entry of entries()) {
      const parent = parentPath(entry.path);
      children.set(parent, [...(children.get(parent) ?? []), entry]);
    }

    const flattened: TreeItem[] = [];
    const visit = (parent: string, depth: number) => {
      const siblings = [...(children.get(parent) ?? [])].sort((left, right) => {
        if (left.kind !== right.kind) return left.kind === "directory" ? -1 : 1;
        return left.path.localeCompare(right.path);
      });
      for (const entry of siblings) {
        flattened.push({ ...entry, depth, name: baseName(entry.path) });
        if (entry.kind === "directory" && expandedPaths().has(entry.path)) {
          visit(entry.path, depth + 1);
        }
      }
    };
    visit("", 0);
    return flattened;
  });

  const isPlaygroundReady = createMemo(() => isMounted() && !editorLoading());

  const setFailure = (message: string) => {
    setRunState("error");
    setStatusMessage(message);
  };

  const getFile = (path: string) =>
    entries().find((entry) => entry.kind === "file" && entry.path === path);

  const getOrCreateModel = (path: string) => {
    const monacoApi = monaco();
    const file = getFile(path);
    if (!monacoApi || !file) return;
    const existing = models.get(path);
    if (existing) return existing;
    const model = monacoApi.editor.createModel(
      file.text,
      path.endsWith(".toml") ? "toml" : "plaintext",
      monacoApi.Uri.parse(workspaceFileUri(path)),
    );
    models.set(path, model);
    return model;
  };

  const requestDiagnostics = async (
    client: TombiLspClient,
    path: string,
    successMessage = "No issues found.",
  ) => {
    const sequence = ++diagnosticSequence;
    setRunState("linting");
    setStatusMessage("Checking with Tombi LSP…");
    try {
      const nextDiagnostics = await client.diagnostics(path);
      if (sequence !== diagnosticSequence || path !== activePath()) return;
      setDiagnostics(nextDiagnostics);
      setRunState("ready");
      setStatusMessage(
        nextDiagnostics.length === 0
          ? successMessage
          : `${nextDiagnostics.length} issue${nextDiagnostics.length === 1 ? "" : "s"} found.`,
      );
    } catch (error) {
      if (sequence !== diagnosticSequence) return;
      setDiagnostics([]);
      setFailure(
        error instanceof Error ? error.message : "Tombi LSP check failed.",
      );
    }
  };

  const openFile = (path: string) => {
    const file = getFile(path);
    const editor = codeEditor();
    if (!file || !editor) return;
    setSelectedDirectoryPath();

    const previousPath = activePath();
    if (previousPath !== path) {
      setActivePath(path);
      setDiagnostics([]);
    }

    const model = getOrCreateModel(path);
    if (model) editor.setModel(model);
    editor.focus();
  };

  const updateActiveFile = (text: string) => {
    const path = activePath();
    const file = getFile(path);
    if (!file || file.text === text) return;
    const version = file.version + 1;
    lsp()?.changeDocument(path, text, version);
    setEntries((current) =>
      current.map((entry) =>
        entry.path === path ? { ...entry, text, version } : entry,
      ),
    );
  };

  const createEntry = (kind: "file" | "directory", input: string): boolean => {
    const label = kind === "file" ? "File" : "Folder";
    const path = normalizeWorkspacePath(input);
    if (!path) {
      setFailure(`Enter a valid ${label.toLowerCase()} path.`);
      return false;
    }
    if (entries().some((entry) => entry.path === path)) {
      setFailure(`“${path}” already exists.`);
      return false;
    }
    let ancestor = parentPath(path);
    while (ancestor) {
      if (
        entries().some(
          (entry) => entry.path === ancestor && entry.kind === "file",
        )
      ) {
        setFailure(`“${ancestor}” is a file, not a folder.`);
        return false;
      }
      ancestor = parentPath(ancestor);
    }

    const nextEntries = withParentDirectories([
      ...entries(),
      { kind, path, text: "", version: kind === "file" ? 1 : 0 },
    ]);
    setEntries(nextEntries);
    const client = lsp();
    if (client && kind === "file") client.openDocument(path, "", 1);

    const parents = new Set(expandedPaths());
    let parent = kind === "directory" ? path : parentPath(path);
    while (parent) {
      parents.add(parent);
      parent = parentPath(parent);
    }
    setExpandedPaths(parents);

    setRunState("ready");
    setStatusMessage(`${label} “${path}” created.`);
    setCreatingEntryKind();
    if (kind === "file") openFile(path);
    else setSelectedDirectoryPath(path);
    return true;
  };

  const beginCreatingEntry = (kind: "file" | "directory") => {
    setRenamingPath();
    setTreeContextMenu();
    const directoryPath = selectedDirectoryPath();
    if (directoryPath) {
      const next = new Set(expandedPaths());
      next.add(directoryPath);
      setExpandedPaths(next);
    }
    setCreatingEntryKind(kind);
    queueMicrotask(() => {
      document
        .querySelector<HTMLInputElement>(".playground-tree-new-input")
        ?.focus();
    });
  };

  const finishCreatingEntry = (input: HTMLInputElement) => {
    const kind = creatingEntryKind();
    if (!kind) return;
    if (!input.value.trim()) {
      setCreatingEntryKind();
      return;
    }
    const directoryPath = selectedDirectoryPath();
    const path = directoryPath
      ? `${directoryPath}/${input.value}`
      : input.value;
    if (createEntry(kind, path)) return;
    queueMicrotask(() => input.focus());
  };

  const renameEntry = (path: string, input: string): boolean => {
    const entry = entries().find((candidate) => candidate.path === path);
    const nextName = input.trim();
    if (!entry) {
      setRenamingPath();
      return true;
    }
    if (
      !nextName ||
      nextName === "." ||
      nextName === ".." ||
      nextName.includes("/") ||
      nextName.includes("\\")
    ) {
      setFailure("Enter a valid file or folder name.");
      return false;
    }

    const parent = parentPath(path);
    const nextPath = parent ? `${parent}/${nextName}` : nextName;
    if (nextPath === path) {
      setRenamingPath();
      return true;
    }

    const currentEntries = entries();
    const affectedEntries = currentEntries.filter((candidate) =>
      isPathWithin(candidate.path, path),
    );
    const unaffectedPaths = new Set(
      currentEntries
        .filter((candidate) => !isPathWithin(candidate.path, path))
        .map((candidate) => candidate.path),
    );
    if (
      affectedEntries.some((candidate) =>
        unaffectedPaths.has(replacePathPrefix(candidate.path, path, nextPath)),
      )
    ) {
      setFailure(`“${nextPath}” already exists.`);
      return false;
    }

    const client = lsp();
    const editor = codeEditor();
    const affectedFiles = affectedEntries.filter(
      (candidate) => candidate.kind === "file",
    );
    const modeledFiles = affectedFiles.filter((candidate) =>
      models.has(candidate.path),
    );
    const currentModel = editor?.getModel();
    if (
      currentModel &&
      modeledFiles.some(
        (candidate) => models.get(candidate.path) === currentModel,
      )
    ) {
      editor?.setModel(null);
    }
    for (const file of affectedFiles) {
      client?.closeDocument(file.path);
    }
    for (const file of modeledFiles) {
      models.get(file.path)?.dispose();
      models.delete(file.path);
    }

    const nextEntries = currentEntries.map((candidate) =>
      isPathWithin(candidate.path, path)
        ? {
            ...candidate,
            path: replacePathPrefix(candidate.path, path, nextPath),
          }
        : candidate,
    );
    setEntries(nextEntries);
    setExpandedPaths(
      new Set(
        [...expandedPaths()].map((expandedPath) =>
          isPathWithin(expandedPath, path)
            ? replacePathPrefix(expandedPath, path, nextPath)
            : expandedPath,
        ),
      ),
    );

    const selectedDirectory = selectedDirectoryPath();
    if (selectedDirectory && isPathWithin(selectedDirectory, path)) {
      setSelectedDirectoryPath(
        replacePathPrefix(selectedDirectory, path, nextPath),
      );
    }
    const previousActivePath = activePath();
    const nextActivePath = isPathWithin(previousActivePath, path)
      ? replacePathPrefix(previousActivePath, path, nextPath)
      : previousActivePath;
    if (nextActivePath !== previousActivePath) {
      setActivePath(nextActivePath);
      setDiagnostics([]);
    }

    for (const file of affectedFiles) {
      const renamedPath = replacePathPrefix(file.path, path, nextPath);
      client?.openDocument(renamedPath, file.text, file.version);
    }
    for (const file of modeledFiles) {
      getOrCreateModel(replacePathPrefix(file.path, path, nextPath));
    }
    if (nextActivePath !== previousActivePath) {
      const model = getOrCreateModel(nextActivePath);
      if (model) editor?.setModel(model);
    }

    setRenamingPath();
    setRunState("ready");
    setStatusMessage(`“${path}” renamed to “${nextPath}”.`);
    return true;
  };

  const finishRenamingEntry = (input: HTMLInputElement) => {
    const path = renamingPath();
    if (!path) return;
    if (!input.value.trim()) {
      setRenamingPath();
      return;
    }
    if (renameEntry(path, input.value)) return;
    queueMicrotask(() => input.focus());
  };

  const beginRenamingEntry = (path: string) => {
    setCreatingEntryKind();
    setTreeContextMenu();
    setRenamingPath(path);
    queueMicrotask(() => {
      const input = document.querySelector<HTMLInputElement>(
        ".playground-tree-rename-input",
      );
      input?.focus();
      input?.select();
    });
  };

  const deleteEntry = (path: string) => {
    const entry = entries().find((candidate) => candidate.path === path);
    if (!entry) return;

    const currentEntries = entries();
    const deletedEntries = currentEntries.filter((candidate) =>
      isPathWithin(candidate.path, path),
    );
    const remainingEntries = currentEntries.filter(
      (candidate) => !isPathWithin(candidate.path, path),
    );
    const deletedFiles = deletedEntries.filter(
      (candidate) => candidate.kind === "file",
    );
    const editor = codeEditor();
    const currentModel = editor?.getModel();
    if (
      currentModel &&
      deletedFiles.some((file) => models.get(file.path) === currentModel)
    ) {
      editor?.setModel(null);
    }
    for (const file of deletedFiles) {
      lsp()?.closeDocument(file.path);
      models.get(file.path)?.dispose();
      models.delete(file.path);
    }

    setEntries(remainingEntries);
    setExpandedPaths(
      new Set(
        [...expandedPaths()].filter(
          (expandedPath) => !isPathWithin(expandedPath, path),
        ),
      ),
    );
    const selectedDirectory = selectedDirectoryPath();
    if (selectedDirectory && isPathWithin(selectedDirectory, path)) {
      setSelectedDirectoryPath();
    }

    if (isPathWithin(activePath(), path)) {
      const nextFile = remainingEntries
        .filter((candidate) => candidate.kind === "file")
        .sort((left, right) => left.path.localeCompare(right.path))[0];
      setActivePath(nextFile?.path ?? "");
      setDiagnostics([]);
      const model = nextFile ? getOrCreateModel(nextFile.path) : undefined;
      editor?.setModel(model ?? null);
    }

    setRenamingPath();
    setTreeContextMenu();
    setRunState("ready");
    setStatusMessage(`“${path}” deleted.`);
  };

  const openTreeContextMenu = (event: MouseEvent, path?: string) => {
    event.preventDefault();
    event.stopPropagation();
    setCreatingEntryKind();
    setRenamingPath();
    const entry = entries().find((candidate) => candidate.path === path);
    setSelectedDirectoryPath(entry?.kind === "directory" ? path : undefined);
    setTreeContextMenu({
      path,
      x: Math.max(8, Math.min(event.clientX, window.innerWidth - 10.5 * 16)),
      y: Math.max(8, Math.min(event.clientY, window.innerHeight - 9.5 * 16)),
    });
    queueMicrotask(() => {
      document
        .querySelector<HTMLButtonElement>(
          ".playground-tree-context-menu button",
        )
        ?.focus();
    });
  };

  const toggleDirectory = (path: string) => {
    setSelectedDirectoryPath(path);
    const next = new Set(expandedPaths());
    if (next.has(path)) next.delete(path);
    else next.add(path);
    setExpandedPaths(next);
  };

  const formatDocument = async () => {
    const client = lsp();
    const editor = codeEditor();
    const path = activePath();
    if (!client || !editor || !activeEntry() || runState() === "formatting") {
      return;
    }

    setRunState("formatting");
    setStatusMessage("Formatting with Tombi LSP…");
    try {
      const edits = await client.format(path);
      if (path !== activePath()) return;
      if (edits.length > 0) {
        editor.pushUndoStop();
        editor.executeEdits(
          "tombi-lsp",
          edits.map((edit) => ({
            range: {
              startLineNumber: edit.range.start.line + 1,
              startColumn: edit.range.start.character + 1,
              endLineNumber: edit.range.end.line + 1,
              endColumn: edit.range.end.character + 1,
            },
            text: edit.newText,
          })),
        );
        editor.pushUndoStop();
      }
      client.saveDocument(path, editor.getValue());
      await requestDiagnostics(
        client,
        path,
        edits.length > 0 ? "Formatted successfully." : "Already formatted.",
      );
    } catch (error) {
      setFailure(
        error instanceof Error ? error.message : "Tombi LSP formatting failed.",
      );
    }
  };

  createEffect(() => {
    const client = lsp();
    const file = activeEntry();
    if (!client || !file) return;
    file.version;
    const timeout = window.setTimeout(() => {
      void requestDiagnostics(client, file.path);
    }, 250);
    onCleanup(() => window.clearTimeout(timeout));
  });

  createEffect(() => {
    const monacoApi = monaco();
    const client = lsp();
    if (!monacoApi || !client) return;
    const providers = registerTombiLspProviders(monacoApi, client);
    onCleanup(() => providers.dispose());
  });

  createEffect(() => {
    const monacoApi = monaco();
    const editor = codeEditor();
    const currentDiagnostics = diagnostics();
    const model = editor?.getModel();
    if (!monacoApi || !model) return;

    monacoApi.editor.setModelMarkers(
      model,
      "tombi",
      currentDiagnostics.map((diagnostic) => {
        const range = model.validateRange({
          startLineNumber: diagnostic.range.start.line + 1,
          startColumn: diagnostic.range.start.character + 1,
          endLineNumber: diagnostic.range.end.line + 1,
          endColumn: diagnostic.range.end.character + 1,
        });
        return {
          code:
            diagnostic.code === undefined ? undefined : String(diagnostic.code),
          message: diagnostic.message,
          severity: markerSeverity(monacoApi, diagnostic),
          source: diagnostic.source || "Tombi",
          ...range,
        };
      }),
    );
  });

  onMount(() => setIsMounted(true));

  onMount(() => {
    let disposed = false;
    let editor: MonacoEditor.IStandaloneCodeEditor | undefined;
    let themeObserver: MutationObserver | undefined;

    onCleanup(() => {
      disposed = true;
      themeObserver?.disconnect();
      editor?.dispose();
      for (const model of models.values()) model.dispose();
      models.clear();
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

        setMonaco(monacoApi);
        const model = getOrCreateModel(DEFAULT_PATH);
        const createdEditor = monacoApi.editor.create(editorContainer, {
          model,
          ariaLabel: "TOML editor",
          automaticLayout: true,
          fontFamily:
            '"Fira Code", "SFMono-Regular", Consolas, "Liberation Mono", monospace',
          fontSize: 14,
          folding: true,
          glyphMargin: false,
          hideCursorInOverviewRuler: true,
          lineDecorationsWidth: 8,
          lineNumbers: "on",
          lineNumbersMinChars: 4,
          minimap: { enabled: false },
          overviewRulerBorder: false,
          padding: { top: 16, bottom: 16 },
          renderValidationDecorations: "on",
          scrollBeyondLastLine: false,
          tabSize: 2,
        });
        editor = createdEditor;
        createdEditor.onDidChangeModelContent(() => {
          updateActiveFile(createdEditor.getValue());
        });

        updateTheme();
        themeObserver = new MutationObserver(updateTheme);
        themeObserver.observe(document.documentElement, {
          attributeFilter: ["class"],
          attributes: true,
        });

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

  createEffect(() => {
    if (editorLoading() || editorError()) return;

    let disposed = false;
    let client: TombiLspClient | undefined;
    onCleanup(() => {
      disposed = true;
      client?.dispose();
    });

    void (async () => {
      try {
        client = await TombiLspClient.create();
        if (disposed) {
          client.dispose();
          return;
        }
        for (const entry of entries()) {
          if (entry.kind === "file") {
            client.openDocument(entry.path, entry.text, entry.version);
          }
        }
        setLsp(client);
        setRunState("ready");
        setStatusMessage("Tombi WebAssembly language server is ready.");
      } catch (error) {
        if (disposed) return;
        setFailure(
          error instanceof Error
            ? error.message
            : "Unable to start the Tombi WebAssembly language server.",
        );
      }
    })();
  });

  onMount(() => {
    const operatingSystem = detectOperatingSystem();
    const isApplePlatform =
      operatingSystem === "mac" || operatingSystem === "ios";
    setFormatShortcut(isApplePlatform ? "⌘S" : "Ctrl S");

    const handleShortcut = (event: KeyboardEvent) => {
      if (
        (!event.metaKey && !event.ctrlKey) ||
        event.key.toLowerCase() !== "s"
      ) {
        return;
      }
      event.preventDefault();
      void formatDocument();
    };
    window.addEventListener("keydown", handleShortcut, { capture: true });
    onCleanup(() =>
      window.removeEventListener("keydown", handleShortcut, { capture: true }),
    );
  });

  onMount(() => {
    const closeContextMenu = () => setTreeContextMenu();
    const handleContextMenuKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") closeContextMenu();
    };
    window.addEventListener("pointerdown", closeContextMenu);
    window.addEventListener("resize", closeContextMenu);
    window.addEventListener("blur", closeContextMenu);
    window.addEventListener("keydown", handleContextMenuKey);
    onCleanup(() => {
      window.removeEventListener("pointerdown", closeContextMenu);
      window.removeEventListener("resize", closeContextMenu);
      window.removeEventListener("blur", closeContextMenu);
      window.removeEventListener("keydown", handleContextMenuKey);
    });
  });

  const goToDiagnostic = (diagnostic: LspDiagnostic) => {
    const editor = codeEditor();
    if (!editor) return;
    const position = {
      lineNumber: diagnostic.range.start.line + 1,
      column: diagnostic.range.start.character + 1,
    };
    editor.setPosition(position);
    editor.revealPositionInCenter(position);
    editor.focus();
  };

  const NewEntryInput = (props: { depth: number }) => (
    <Show when={creatingEntryKind()}>
      {(kind) => (
        <div
          class="playground-tree-new-entry"
          role="treeitem"
          tabIndex={-1}
          style={{ "padding-left": `${0.45 + props.depth * 0.9}rem` }}
        >
          <span class="playground-tree-chevron" />
          <Show
            when={kind() === "directory"}
            fallback={<TbFile aria-hidden="true" />}
          >
            <TbFolder aria-hidden="true" />
          </Show>
          <input
            type="text"
            name="workspace-entry-path"
            class="playground-tree-new-input"
            aria-label={kind() === "file" ? "New file path" : "New folder path"}
            autocomplete="off"
            spellcheck={false}
            onBlur={(event) => finishCreatingEntry(event.currentTarget)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                finishCreatingEntry(event.currentTarget);
              } else if (event.key === "Escape") {
                event.preventDefault();
                setCreatingEntryKind();
              }
            }}
          />
        </div>
      )}
    </Show>
  );

  const RenameEntryInput = (props: { entry: TreeItem }) => (
    <div
      class="playground-tree-new-entry"
      role="treeitem"
      tabIndex={-1}
      style={{ "padding-left": `${0.45 + props.entry.depth * 0.9}rem` }}
    >
      <span class="playground-tree-chevron">
        <Show when={props.entry.kind === "directory"}>
          <Show
            when={expandedPaths().has(props.entry.path)}
            fallback={<TbChevronRight aria-hidden="true" />}
          >
            <TbChevronDown aria-hidden="true" />
          </Show>
        </Show>
      </span>
      <Show
        when={props.entry.kind === "directory"}
        fallback={<TbFile aria-hidden="true" />}
      >
        <TbFolder aria-hidden="true" />
      </Show>
      <input
        type="text"
        class="playground-tree-new-input playground-tree-rename-input"
        aria-label={`Rename ${props.entry.name}`}
        value={props.entry.name}
        autocomplete="off"
        spellcheck={false}
        onBlur={(event) => finishRenamingEntry(event.currentTarget)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            finishRenamingEntry(event.currentTarget);
          } else if (event.key === "Escape") {
            event.preventDefault();
            setRenamingPath();
          }
        }}
      />
    </div>
  );

  return (
    <>
      <PageHeading
        title="TOML Playground | Tombi"
        description="Explore Tombi's WebAssembly language server in a browser workspace."
        og_url={`${DEFAULT_URL}playground`}
      />

      <div
        class="playground-shell"
        aria-busy={!isPlaygroundReady()}
        style={{ position: "relative", padding: "2.5rem 0 4rem" }}
      >
        <section
          class="playground-workspace"
          aria-label="Tombi WASM LSP playground"
          aria-hidden={!isPlaygroundReady()}
          style={{ display: isPlaygroundReady() ? undefined : "none" }}
        >
          <div class="playground-toolbar">
            <div class="playground-runtime-label">
              <button
                type="button"
                class="playground-explorer-toggle"
                title={isExplorerOpen() ? "Collapse sidebar" : "Expand sidebar"}
                aria-label={
                  isExplorerOpen() ? "Collapse sidebar" : "Expand sidebar"
                }
                aria-controls="playground-explorer"
                aria-expanded={isExplorerOpen()}
                onClick={() => setIsExplorerOpen((isOpen) => !isOpen)}
              >
                <Show
                  when={isExplorerOpen()}
                  fallback={<TbLayoutSidebarLeftExpand aria-hidden="true" />}
                >
                  <TbLayoutSidebarLeftCollapse aria-hidden="true" />
                </Show>
              </button>
              <div>
                <strong>WASM LSP</strong>
                <span>{activePath()}</span>
              </div>
            </div>

            <div class="playground-actions">
              <button
                type="button"
                class="playground-button playground-button-primary group"
                title={`Format (${formatShortcut()})`}
                aria-keyshortcuts={
                  formatShortcut() === "⌘S" ? "Meta+S" : "Control+S"
                }
                onClick={() => void formatDocument()}
                disabled={!lsp() || !activeEntry()}
                aria-busy={runState() === "formatting"}
              >
                <FaSolidFeather
                  class="group-hover:animate-shake"
                  aria-hidden="true"
                />
                Format
                <span class="playground-button-shortcut" aria-hidden="true">
                  {formatShortcut()}
                </span>
              </button>
            </div>
          </div>

          <div
            class="playground-main"
            classList={{ "is-explorer-collapsed": !isExplorerOpen() }}
          >
            <aside
              id="playground-explorer"
              class="playground-explorer"
              aria-label="Virtual filesystem"
            >
              <div class="playground-explorer-header">
                <span>Files</span>
                <div class="playground-explorer-actions">
                  <button
                    type="button"
                    title="New file"
                    aria-label="New file"
                    disabled={editorLoading() || !lsp()}
                    onClick={() => beginCreatingEntry("file")}
                  >
                    <TbFilePlus aria-hidden="true" />
                  </button>
                  <button
                    type="button"
                    title="New folder"
                    aria-label="New folder"
                    disabled={editorLoading() || !lsp()}
                    onClick={() => beginCreatingEntry("directory")}
                  >
                    <TbFolderPlus aria-hidden="true" />
                  </button>
                </div>
              </div>

              <div
                class="playground-tree"
                role="tree"
                aria-label="Workspace files"
                onContextMenu={(event) => openTreeContextMenu(event)}
              >
                <Show when={!selectedDirectoryPath()}>
                  <NewEntryInput depth={0} />
                </Show>
                <For each={treeItems()}>
                  {(entry) => (
                    <>
                      <Show when={renamingPath() === entry.path}>
                        <RenameEntryInput entry={entry} />
                      </Show>
                      <button
                        type="button"
                        role="treeitem"
                        aria-expanded={
                          entry.kind === "directory"
                            ? expandedPaths().has(entry.path)
                            : undefined
                        }
                        aria-selected={
                          entry.kind === "directory"
                            ? entry.path === selectedDirectoryPath()
                            : !selectedDirectoryPath() &&
                              entry.path === activePath()
                        }
                        class="playground-tree-item"
                        classList={{
                          "is-context-target":
                            treeContextMenu()?.path === entry.path,
                          "is-renaming": renamingPath() === entry.path,
                          "is-active":
                            (entry.kind === "directory" &&
                              entry.path === selectedDirectoryPath()) ||
                            (entry.kind === "file" &&
                              !selectedDirectoryPath() &&
                              entry.path === activePath()),
                        }}
                        style={{
                          "padding-left": `${0.45 + entry.depth * 0.9}rem`,
                        }}
                        onClick={() =>
                          entry.kind === "directory"
                            ? toggleDirectory(entry.path)
                            : openFile(entry.path)
                        }
                        onDblClick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          beginRenamingEntry(entry.path);
                        }}
                        onContextMenu={(event) =>
                          openTreeContextMenu(event, entry.path)
                        }
                      >
                        <span class="playground-tree-chevron">
                          <Show when={entry.kind === "directory"}>
                            <Show
                              when={expandedPaths().has(entry.path)}
                              fallback={<TbChevronRight aria-hidden="true" />}
                            >
                              <TbChevronDown aria-hidden="true" />
                            </Show>
                          </Show>
                        </span>
                        <Show
                          when={entry.kind === "directory"}
                          fallback={<TbFile aria-hidden="true" />}
                        >
                          <TbFolder aria-hidden="true" />
                        </Show>
                        <span>{entry.name}</span>
                      </button>
                      <Show
                        when={
                          entry.kind === "directory" &&
                          entry.path === selectedDirectoryPath()
                        }
                      >
                        <NewEntryInput depth={entry.depth + 1} />
                      </Show>
                    </>
                  )}
                </For>
              </div>
            </aside>

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
          </div>

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
                  {(diagnostic) => (
                    <button
                      type="button"
                      data-level={
                        diagnostic.severity === 2 ? "warning" : "error"
                      }
                      onClick={() => goToDiagnostic(diagnostic)}
                    >
                      <div class="playground-diagnostic-meta">
                        <span>
                          {diagnostic.severity === 2 ? "WARNING" : "ERROR"}
                        </span>
                        <code>{diagnostic.code ?? "syntax"}</code>
                        <span class="playground-diagnostic-location">
                          {activePath()}:{diagnostic.range.start.line + 1}:
                          {diagnostic.range.start.character + 1}
                        </span>
                      </div>
                      <p>{diagnostic.message}</p>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </section>
        </section>

        <Show when={!isPlaygroundReady()}>
          <output
            class="playground-workspace playground-initial-loading"
            aria-live="polite"
            style={{
              display: "grid",
              "min-height": "42rem",
              color: "var(--playground-muted, #667085)",
              "font-size": "0.9rem",
              "font-weight": 650,
              gap: "0.65rem",
              "place-content": "center",
              overflow: "hidden",
              border: "1px solid var(--playground-border, #d9dee9)",
              "border-radius": "1rem",
              background: "var(--playground-panel, #fff)",
              "box-shadow": "0 18px 45px rgb(15 23 42 / 8%)",
            }}
          >
            <TbLoader2
              class="playground-spinner animate-spin"
              aria-hidden="true"
              style={{
                width: "1.4rem",
                height: "1.4rem",
                "margin-inline": "auto",
              }}
            />
            <span>Loading playground…</span>
          </output>
        </Show>

        <Show when={treeContextMenu()}>
          {(menu) => (
            <div
              class="playground-tree-context-menu"
              role="menu"
              aria-label={
                menu().path
                  ? `Actions for ${baseName(menu().path ?? "")}`
                  : "Workspace actions"
              }
              style={{ left: `${menu().x}px`, top: `${menu().y}px` }}
              onPointerDown={(event) => event.stopPropagation()}
              onContextMenu={(event) => event.preventDefault()}
            >
              <Show
                when={
                  !menu().path ||
                  entries().find((entry) => entry.path === menu().path)
                    ?.kind === "directory"
                }
              >
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => beginCreatingEntry("file")}
                >
                  New File
                </button>
                <button
                  type="button"
                  role="menuitem"
                  onClick={() => beginCreatingEntry("directory")}
                >
                  New Folder
                </button>
                <Show when={menu().path}>
                  <hr class="playground-tree-context-separator" />
                </Show>
              </Show>
              <Show when={menu().path}>
                {(path) => (
                  <>
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => beginRenamingEntry(path())}
                    >
                      Rename
                    </button>
                    <button
                      type="button"
                      role="menuitem"
                      class="is-danger"
                      onClick={() => deleteEntry(path())}
                    >
                      Delete
                    </button>
                  </>
                )}
              </Show>
            </div>
          )}
        </Show>
      </div>
    </>
  );
}
