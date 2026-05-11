import * as path from "node:path";
import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";
import { getBuiltInSchema } from "@/lsp/client";

export async function openTooltipLink(
  target: string,
  client: node.LanguageClient,
): Promise<void> {
  const uri = vscode.Uri.parse(target, true);

  if (uri.scheme === "file") {
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    return;
  }

  if (uri.scheme === "https" || uri.scheme === "http") {
    await vscode.env.openExternal(uri);
    return;
  }

  if (uri.scheme === "tombi") {
    const content = await client.sendRequest(getBuiltInSchema, {
      uri: target,
    });
    if (content == null) {
      vscode.window.showWarningMessage(`Schema not found: ${target}`);
      return;
    }

    const document = await vscode.workspace.openTextDocument({
      language: "json",
      content,
    });
    await vscode.window.showTextDocument(document, {
      preview: false,
    });
    return;
  }

  vscode.window.showWarningMessage(`Unsupported link: ${target}`);
}

export function isLocalFilePath(value: string): boolean {
  return path.isAbsolute(value) || path.win32.isAbsolute(value);
}
