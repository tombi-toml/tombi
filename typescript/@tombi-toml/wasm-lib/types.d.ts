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

/** An error reported when an operation cannot be executed. */
export interface TombiWasmError extends Error {
  readonly name: "TombiWasmError";
}
