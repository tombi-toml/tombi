import * as v from "valibot";
import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";
import { isLocalFilePath } from "@/command/open-tooltip-link";
import { log } from "@/logging";
import type { AssociateSchemaParams } from "@/lsp/client";

const AssociateSchemaArgsSchema = v.object({
  path: v.pipe(
    v.string(),
    v.check(isLocalFilePath, "`path` must be an absolute file path"),
  ),
  fileMatch: v.union([v.string(), v.array(v.string())]),
  title: v.optional(v.string()),
  description: v.optional(v.string()),
  tomlVersion: v.optional(v.string()),
  force: v.optional(v.boolean()),
});
export type AssociateSchemaArgs = v.InferInput<
  typeof AssociateSchemaArgsSchema
>;

export function toAssociateSchemaParams(args: unknown): AssociateSchemaParams {
  const { path, fileMatch, ...rest } = v.parse(AssociateSchemaArgsSchema, args);
  return {
    ...rest,
    uri: vscode.Uri.file(path).toString(),
    fileMatch: typeof fileMatch === "string" ? [fileMatch] : fileMatch,
  };
}

/**
 * Associate a local JSON Schema file with TOML files matching `fileMatch`.
 *
 * Intended to be called programmatically, e.g. from another extension:
 *
 * ```ts
 * await vscode.commands.executeCommand("tombi.associateSchema", {
 *   path: "/abs/path/to/schema.json",
 *   fileMatch: ["**\/foo.toml"],
 * });
 * ```
 */
export async function associateSchema(
  client: node.LanguageClient,
  args: unknown,
): Promise<void> {
  let params: AssociateSchemaParams;
  try {
    params = toAssociateSchemaParams(args);
  } catch (error) {
    log.error(`Invalid arguments for associateSchema: ${error}`);
    throw error;
  }

  await client.sendNotification("tombi/associateSchema", params);
  log.info(
    `Schema associated: ${params.uri} -> ${params.fileMatch.join(", ")}`,
  );
}
