import * as vscode from "vscode";
import type { Server } from "../lsp/server";

export async function showLanguageServerVersion(server: Server): Promise<void> {
  const version = await server.showVersion();

  vscode.window.showInformationMessage(
    `Tombi Language Server Version: ${version} (${server.tombiBin.source})`,
  );
}

export async function showTomlVersion(
  server: Server,
  uri: string,
): Promise<void> {
  const version = await server.showTomlVersion(uri);

  vscode.window.showInformationMessage(`TOML Version: ${version}`);
}
