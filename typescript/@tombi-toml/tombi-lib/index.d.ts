export * from "./binding";

/** The `tombi.toml` configuration could not be loaded or parsed. */
export interface TombiConfigError extends Error {
  readonly name: "TombiConfigError";
}

/** A schema or schema catalog could not be resolved. */
export interface TombiSchemaError extends Error {
  readonly name: "TombiSchemaError";
}

/** An I/O operation failed. */
export interface TombiIOError extends Error {
  readonly name: "TombiIOError";
}

/**
 * Every error `format`/`lint` rejects with (other than a `TypeError` for
 * malformed `options`). Narrow it exhaustively by `name`.
 */
export type TombiError = TombiConfigError | TombiSchemaError | TombiIOError;
