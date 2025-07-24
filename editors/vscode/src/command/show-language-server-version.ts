import * as vscode from "vscode";
import type { Server } from "../lsp/server";

export async function showLanguageServerVersion(server: Server): Promise<void> {
  const version = await server.showVersion();

  let message = `Tombi Language Server Version: ${version}(${server.tombiBin.source})`;

  if (
    server.tombiBin.source === "local" ||
    server.tombiBin.source === "editor settings"
  ) {
    message += `\t${server.tombiBin.path}`;
  }

  vscode.window.showInformationMessage(message);
}
