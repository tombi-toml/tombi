import { gte } from "semver";
import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";
import { TOMBI_DEV_VERSION } from "@/extension";
import { log } from "@/logging";
import { listSchemas, type SchemaInfo } from "@/lsp/client";

const MIN_VERSION = "0.7.19";

export async function selectSchema(
  client: node.LanguageClient,
  lspVersion?: string,
): Promise<void> {
  try {
    // Check LSP version
    if (
      lspVersion &&
      lspVersion !== TOMBI_DEV_VERSION &&
      !gte(lspVersion, MIN_VERSION)
    ) {
      const message = `Select Schema requires Tombi v${MIN_VERSION} or later. Current version: ${lspVersion}. Please update Tombi.`;
      vscode.window.showErrorMessage(message);
      log.warn(message);
      return;
    }

    // Get the active editor
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showWarningMessage("No active editor");
      return;
    }

    // Check if the active file is a TOML file
    if (editor.document.languageId !== "toml") {
      vscode.window.showWarningMessage(
        "Current file is not a TOML file. Please open a TOML file first.",
      );
      return;
    }

    // Get the current file URI
    const documentUri = editor.document.uri;
    if (documentUri.scheme === "untitled") {
      vscode.window.showWarningMessage(
        "Untitled files are not supported. Please save the file first.",
      );
      return;
    }

    // Fetch schemas from LSP
    const response = await client.sendRequest(listSchemas, {});
    const schemas = response.schemas;

    if (schemas.length === 0) {
      vscode.window.showInformationMessage("No schemas available");
      return;
    }

    // Create QuickPick items
    const items: Array<vscode.QuickPickItem & { schema: SchemaInfo }> =
      schemas.map((schema) => {
        const item: vscode.QuickPickItem & { schema: SchemaInfo } = {
          label: schema.title || schema.uri,
          description: schema.uri,
          schema,
        };
        if (schema.description) {
          item.detail = schema.description;
        }
        return item;
      });

    // Show QuickPick to select a schema
    const selected = await vscode.window.showQuickPick(items, {
      placeHolder: "Select a schema to apply to the current TOML file",
      matchOnDescription: true,
      matchOnDetail: true,
    });

    if (!selected) {
      // User cancelled the selection
      return;
    }

    const filePattern = documentUri.path;

    // Send associateSchema notification to LSP
    await client.sendNotification("tombi/associateSchema", {
      title: selected.schema.title,
      description: selected.schema.description,
      uri: selected.schema.uri,
      fileMatch: [documentUri.path],
      tomlVersion: selected.schema.tomlVersion,
      force: true, // Force user-selected schema to take precedence over catalog schemas
    });

    log.info(`Schema associated: ${selected.schema.uri} -> ${filePattern}`);

    vscode.window.showInformationMessage(
      `Schema "${selected.label}" applied successfully`,
    );
  } catch (error) {
    log.error(`Failed to select schema: ${error}`);
    vscode.window.showErrorMessage(`Failed to select schema: ${error}`);
  }
}
