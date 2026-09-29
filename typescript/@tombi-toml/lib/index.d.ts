export * from "./binding";

/**
 * The error `format`/`lint` reject with (and `formatSync`/`lintSync` throw) when
 * the configuration, a schema, or an I/O operation fails. Malformed `options`
 * reject (or throw) with a `TypeError` instead.
 */
export interface TombiError extends Error {
  readonly name: "TombiError";
}
