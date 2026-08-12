export * from "./tombi_wasm";

/** A diagnostic reported by Tombi. */
export interface Diagnostic {
  level: "ERROR" | "WARNING";
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

/** An error reported when formatting fails. */
export type FormatError = { error: string } | { diagnostics: Diagnostic[] };

/** An error reported when linting cannot be executed. */
export interface LintError {
  error: string;
}
