export interface LspPosition {
  line: number;
  character: number;
}

export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

export interface LspDiagnostic {
  range: LspRange;
  severity?: number;
  code?: string | number;
  source?: string;
  message: string;
}

export interface LspTextEdit {
  range: LspRange;
  newText: string;
}

export interface LspMarkupContent {
  kind: "markdown" | "plaintext";
  value: string;
}

export interface LspLocation {
  uri: string;
  range: LspRange;
}

export interface LspLocationLink {
  originSelectionRange?: LspRange;
  targetUri: string;
  targetRange: LspRange;
  targetSelectionRange: LspRange;
}

interface JsonRpcMessage {
  type?: "ready" | "error";
  message?: string;
  jsonrpc?: string;
  id?: number;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: { code?: number; message?: string };
}

interface PendingRequest {
  resolve(value: unknown): void;
  reject(reason: Error): void;
  timeout: number;
}

const WORKSPACE_URI = "file:///workspace";
const TOMBI_CONFIG_FILENAMES = new Set([
  ".tombi.toml",
  "pyproject.toml",
  "tombi.toml",
]);

function baseUrl(): string {
  return import.meta.env.SERVER_BASE_URL.endsWith("/")
    ? import.meta.env.SERVER_BASE_URL
    : `${import.meta.env.SERVER_BASE_URL}/`;
}

function encodePath(path: string): string {
  return path
    .split("/")
    .map((segment) => encodeURIComponent(segment))
    .join("/");
}

export function workspaceFileUri(path: string): string {
  return `${WORKSPACE_URI}/${encodePath(path)}`;
}

export class TombiLspClient {
  readonly #worker: Worker;
  readonly #pending = new Map<number, PendingRequest>();
  readonly #ready: Promise<void>;
  readonly #resolveReady: () => void;
  readonly #rejectReady: (reason: Error) => void;
  #nextId = 1;

  private constructor(worker: Worker) {
    this.#worker = worker;
    let resolveReady!: () => void;
    let rejectReady!: (reason: Error) => void;
    this.#ready = new Promise<void>((resolve, reject) => {
      resolveReady = resolve;
      rejectReady = reject;
    });
    this.#resolveReady = resolveReady;
    this.#rejectReady = rejectReady;
    worker.addEventListener(
      "message",
      (event: MessageEvent<JsonRpcMessage>) => {
        this.#handleMessage(event.data);
      },
    );
    worker.addEventListener("error", (event) => {
      const error = new Error(event.message || "Tombi LSP worker failed.");
      this.#rejectReady(error);
      this.#failPending(error);
    });
  }

  static async create(): Promise<TombiLspClient> {
    const workerUrl = new URL(
      `${baseUrl()}wasm/worker.js`,
      window.location.origin,
    );
    const client = new TombiLspClient(
      new Worker(workerUrl, { name: "tombi-lsp", type: "module" }),
    );

    await client.#ready;
    await client.request("initialize", {
      capabilities: {
        general: { positionEncodings: ["utf-16"] },
        textDocument: {
          codeAction: { dynamicRegistration: false },
          completion: {
            completionItem: {
              documentationFormat: ["markdown", "plaintext"],
              labelDetailsSupport: true,
              snippetSupport: true,
            },
            dynamicRegistration: false,
          },
          declaration: { dynamicRegistration: false, linkSupport: true },
          definition: { dynamicRegistration: false, linkSupport: true },
          diagnostic: { dynamicRegistration: true },
          documentLink: { dynamicRegistration: false },
          documentSymbol: { dynamicRegistration: false },
          foldingRange: { dynamicRegistration: false },
          formatting: { dynamicRegistration: false },
          hover: {
            contentFormat: ["markdown", "plaintext"],
            dynamicRegistration: false,
          },
          inlayHint: { dynamicRegistration: false },
          references: { dynamicRegistration: false },
          semanticTokens: {
            dynamicRegistration: false,
            formats: ["relative"],
            requests: { full: true },
            tokenModifiers: [],
            tokenTypes: [
              "namespace",
              "type",
              "class",
              "enum",
              "interface",
              "struct",
              "typeParameter",
              "parameter",
              "variable",
              "property",
              "enumMember",
              "event",
              "function",
              "method",
              "macro",
              "keyword",
              "modifier",
              "comment",
              "string",
              "number",
              "regexp",
              "operator",
            ],
          },
          synchronization: { dynamicRegistration: false },
          typeDefinition: { dynamicRegistration: false, linkSupport: true },
        },
        workspace: {
          configuration: true,
          diagnostic: { refreshSupport: false },
          workspaceFolders: true,
        },
      },
      clientInfo: { name: "Tombi Playground" },
      processId: null,
      rootUri: WORKSPACE_URI,
      workspaceFolders: [{ name: "workspace", uri: WORKSPACE_URI }],
    });
    client.#notify("initialized", {});
    return client;
  }

  openDocument(path: string, text: string, version: number): void {
    this.#notify("textDocument/didOpen", {
      textDocument: {
        languageId: path.endsWith(".toml") ? "toml" : "plaintext",
        text,
        uri: workspaceFileUri(path),
        version,
      },
    });
  }

  changeDocument(path: string, text: string, version: number): void {
    this.#notify("textDocument/didChange", {
      contentChanges: [{ text }],
      textDocument: { uri: workspaceFileUri(path), version },
    });
  }

  closeDocument(path: string): void {
    this.#notify("textDocument/didClose", {
      textDocument: { uri: workspaceFileUri(path) },
    });
  }

  saveDocument(path: string, text: string): void {
    this.#notify("textDocument/didSave", {
      text,
      textDocument: { uri: workspaceFileUri(path) },
    });
  }

  async diagnostics(path: string): Promise<LspDiagnostic[]> {
    const report = await this.request<{ items?: LspDiagnostic[] }>(
      "textDocument/diagnostic",
      { textDocument: { uri: workspaceFileUri(path) } },
    );
    return report?.items ?? [];
  }

  async format(path: string): Promise<LspTextEdit[]> {
    const filename = path.slice(path.lastIndexOf("/") + 1);
    if (TOMBI_CONFIG_FILENAMES.has(filename)) {
      await this.request<boolean>("tombi/updateConfig", {
        uri: workspaceFileUri(path),
      });
    }

    return (
      (await this.request<LspTextEdit[] | null>("textDocument/formatting", {
        options: { insertSpaces: true, tabSize: 2 },
        textDocument: { uri: workspaceFileUri(path) },
      })) ?? []
    );
  }

  request<T>(method: string, params: unknown): Promise<T> {
    return this.#request<T>(method, params);
  }

  dispose(): void {
    this.#failPending(new Error("Tombi LSP client disposed."));
    this.#worker.terminate();
  }

  #request<T>(method: string, params: unknown): Promise<T> {
    const id = this.#nextId++;
    return new Promise<T>((resolve, reject) => {
      const timeout = window.setTimeout(() => {
        this.#pending.delete(id);
        reject(new Error(`Tombi LSP request timed out: ${method}`));
      }, 15_000);
      this.#pending.set(id, {
        reject,
        resolve: (value) => resolve(value as T),
        timeout,
      });
      this.#worker.postMessage({ jsonrpc: "2.0", id, method, params });
    });
  }

  #notify(method: string, params: unknown): void {
    this.#worker.postMessage({ jsonrpc: "2.0", method, params });
  }

  #handleMessage(message: JsonRpcMessage): void {
    if (message.type === "ready") {
      this.#resolveReady();
      return;
    }
    if (message.type === "error") {
      const error = new Error(message.message || "Tombi LSP worker failed.");
      this.#rejectReady(error);
      this.#failPending(error);
      return;
    }
    if (typeof message.id === "number" && message.method) {
      this.#handleServerRequest(message);
      return;
    }
    if (typeof message.id !== "number") return;

    const pending = this.#pending.get(message.id);
    if (!pending) return;
    this.#pending.delete(message.id);
    window.clearTimeout(pending.timeout);
    if (message.error) {
      pending.reject(
        new Error(message.error.message || "Tombi LSP request failed."),
      );
    } else {
      pending.resolve(message.result);
    }
  }

  #handleServerRequest(message: JsonRpcMessage): void {
    let result: unknown = null;
    if (message.method === "workspace/configuration") {
      const items = (message.params as { items?: unknown[] } | undefined)
        ?.items;
      result = items?.map(() => null) ?? [];
    } else if (message.method === "workspace/workspaceFolders") {
      result = [{ name: "workspace", uri: WORKSPACE_URI }];
    }
    this.#worker.postMessage({ jsonrpc: "2.0", id: message.id, result });
  }

  #failPending(error: Error): void {
    for (const pending of this.#pending.values()) {
      window.clearTimeout(pending.timeout);
      pending.reject(error);
    }
    this.#pending.clear();
  }
}
