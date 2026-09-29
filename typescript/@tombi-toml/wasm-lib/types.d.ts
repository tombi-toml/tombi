export * from "./tombi_wasm";

/** A diagnostic reported by Tombi. */
export interface Diagnostic {
  level: "error" | "warning";
  code: string;
  message: string;
  range: Range;
  sourceFile: string | null;
  /** @deprecated Use {@link Diagnostic.sourceFile} instead. */
  source_file: string | null;
}

/** A zero-based position in a TOML document. */
export interface Position {
  line: number;
  column: number;
}

/** A range in a TOML document. */
export interface Range {
  start: Position;
  end: Position;
}

/** The result of formatting a TOML document. */
export interface FormatResult {
  formatted?: string;
  diagnostics: Diagnostic[];
}

/** The result of linting a TOML document. */
export interface LintResult {
  diagnostics: Diagnostic[];
}

/** The content of a `tombi.toml` config file at a given path. */
export interface ConfigFile {
  content: string;
  path: string;
}

/** @deprecated Use `string | ConfigFile` instead. */
export type Config = string | ConfigFile;

/** Options shared by the formatter and linter. */
export interface Options {
  /**
   * An in-memory `tombi.toml` configuration.
   * When a string is provided, it is treated as the content of a virtual
   * `tombi.toml`.
   */
  config?: string | ConfigFile;
}

/** Format a TOML document. */
export function format(
  source: string,
  sourcePath: string,
  options?: Options,
): Promise<FormatResult>;

/** Lint a TOML document. */
export function lint(
  source: string,
  sourcePath: string,
  options?: Options,
): Promise<LintResult>;

/**
 * The error `format`/`lint` reject with when the configuration, a schema, or an
 * I/O operation fails. Malformed `options` reject with a `TypeError` instead.
 */
export interface TombiError extends Error {
  readonly name: "TombiError";
}

/** @deprecated Use {@link TombiError} instead. */
export type TombiWasmError = TombiError;

/** One entry of the browser-backed virtual workspace. */
export interface WorkspaceEntry {
  uri: string;
  kind?: "file" | "directory";
  text?: string;
}

/** Update one file in the browser-backed virtual workspace. */
export function set_workspace_file(uri: string, text: string): void;

/** Remove one file from the browser-backed virtual workspace. */
export function remove_workspace_file(uri: string): void;

/** Replace the virtual workspace entries visible to `format`/`lint`. */
export function set_workspace_entries(entries: WorkspaceEntry[]): void;

/** Replace the virtual workspace files visible to `format`/`lint`. */
export function set_workspace_files(files: WorkspaceEntry[]): void;
