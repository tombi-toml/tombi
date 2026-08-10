export interface TombiPosition {
  line: number;
  column: number;
}

export interface TombiRange {
  start: TombiPosition;
  end: TombiPosition;
}

export interface TombiDiagnostic {
  level: "ERROR" | "WARNING" | string;
  code: string;
  message: string;
  range?: TombiRange;
  source_file?: string | null;
}

export interface TombiWasm {
  format(
    source: string,
    filePath?: string,
    tomlVersion?: string,
  ): Promise<string>;
  lint(source: string, filePath?: string, tomlVersion?: string): Promise<void>;
}

interface TombiWasmModule extends TombiWasm {
  default(): Promise<unknown>;
}

export interface TombiWasmFailure {
  error?: string;
  diagnostics: TombiDiagnostic[];
}

let wasmPromise: Promise<TombiWasm> | undefined;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function normalizeWasmError(error: unknown): TombiWasmFailure {
  if (isRecord(error)) {
    const diagnostics = Array.isArray(error.diagnostics)
      ? error.diagnostics.filter(isRecord).map((diagnostic) => ({
          level:
            typeof diagnostic.level === "string" ? diagnostic.level : "ERROR",
          code: typeof diagnostic.code === "string" ? diagnostic.code : "",
          message:
            typeof diagnostic.message === "string"
              ? diagnostic.message
              : "Unknown Tombi diagnostic",
          range: isRecord(diagnostic.range)
            ? (diagnostic.range as unknown as TombiRange)
            : undefined,
          source_file:
            typeof diagnostic.source_file === "string"
              ? diagnostic.source_file
              : null,
        }))
      : [];

    return {
      error: typeof error.error === "string" ? error.error : undefined,
      diagnostics,
    };
  }

  return {
    error: error instanceof Error ? error.message : String(error),
    diagnostics: [],
  };
}

export function loadTombiWasm(): Promise<TombiWasm> {
  if (wasmPromise) return wasmPromise;

  wasmPromise = (async () => {
    const baseUrl = import.meta.env.BASE_URL.endsWith("/")
      ? import.meta.env.BASE_URL
      : `${import.meta.env.BASE_URL}/`;
    const moduleUrl = new URL(
      `${baseUrl}wasm/tombi_wasm.js`,
      window.location.origin,
    ).href;
    const module = (await import(
      /* @vite-ignore */ moduleUrl
    )) as TombiWasmModule;
    await module.default();
    return module;
  })().catch((error) => {
    wasmPromise = undefined;
    throw error;
  });

  return wasmPromise;
}
