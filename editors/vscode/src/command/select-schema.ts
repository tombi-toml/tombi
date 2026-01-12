import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";
import { log } from "@/logging";
import { listSchemas, type SchemaInfo } from "@/lsp/client";

export async function selectSchema(client: node.LanguageClient): Promise<void> {
  try {
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

    // Get the workspace folder of the current file
    const workspaceFolder = vscode.workspace.getWorkspaceFolder(documentUri);
    let filePattern: string;

    if (workspaceFolder) {
      // Create a relative path pattern from the workspace root
      const relativePath = vscode.workspace.asRelativePath(documentUri, false);
      filePattern = `**/${relativePath.split("/").pop()}`; // Use the filename with ** prefix
    } else {
      // Use the filename if not in a workspace
      filePattern = `**/${documentUri.path.split("/").pop()}`;
    }

    // Send associateSchema notification to LSP
    await client.sendNotification("tombi/associateSchema", {
      uri: selected.schema.uri,
      fileMatch: [filePattern],
      tomlVersion: selected.schema.tomlVersion,
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
