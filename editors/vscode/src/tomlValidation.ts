import * as v from "valibot";
import * as vscode from "vscode";
import type { BaseLanguageClient } from "vscode-languageclient";
import type { AssociateSchemaParams } from "./lsp/client";

const TomlValidationSchema = v.optional(
  v.array(
    v.object({
      url: v.string(),
      fileMatch: v.union([v.string(), v.array(v.string())]),
    }),
  ),
);
type TomlValidation = v.InferOutput<typeof TomlValidationSchema>;

export function registerExtensionSchemas(client: BaseLanguageClient) {
  for (const ext of vscode.extensions.all) {
    let tomlValidations: TomlValidation;
    try {
      tomlValidations = v.parse(
        TomlValidationSchema,
        ext.packageJSON?.contributes?.tomlValidation,
      );
    } catch (e) {
      console.error(
        `Failed to parse tomlValidation for extension ${ext.id}: ${e}`,
        ext.packageJSON?.contributes?.tomlValidation,
      );
      continue;
    }

    if (tomlValidations === undefined) {
      continue;
    }
    for (const tomlValidation of tomlValidations) {
      if (typeof tomlValidation.fileMatch === "string") {
        tomlValidation.fileMatch = [tomlValidation.fileMatch];
      }
      const params: AssociateSchemaParams = {
        uri: tomlValidation.url,
        fileMatch: tomlValidation.fileMatch,
      };
      client.sendNotification("tombi/associateSchema", params);
    }
  }
}
