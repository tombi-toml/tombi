export * from "./tombi_wasm";

/** A diagnostic reported by Tombi. */
export interface Diagnostic {
  level: "error" | "warning";
  code: string;
  message: string;
  range: Range;
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

/**
 * An in-memory `tombi.toml` configuration.
 * When a string is provided, it is treated as the content of a virtual
 * `tombi.toml`.
 */
export type Config = { content: string; path: string } | string;

/** Options shared by the formatter and linter. */
export interface Options {
  config?: Config;
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
 * Format a TOML document synchronously.
 *
 * Throws a `TombiError` when formatting needs asynchronous I/O, such as
 * fetching a remote schema. Use `format` in that case.
 */
export function formatSync(
  source: string,
  sourcePath: string,
  options?: Options,
): FormatResult;

/**
 * Lint a TOML document synchronously.
 *
 * Throws a `TombiError` when linting needs asynchronous I/O, such as fetching
 * a remote schema. Use `lint` in that case.
 */
export function lintSync(
  source: string,
  sourcePath: string,
  options?: Options,
): LintResult;

/**
 * The error `format`/`lint` reject with when the configuration, a schema, or an
 * I/O operation fails. Malformed `options` reject with a `TypeError` instead.
 */
export interface TombiError extends Error {
  readonly name: "TombiError";
}

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
