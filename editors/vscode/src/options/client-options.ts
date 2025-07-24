import { SUPPORT_TOML_LANGUAGES } from "@/extension";
import * as vscode from "vscode";
import type * as languageclient from "vscode-languageclient";

export function clientOptions(
  workspaceFolder?: vscode.WorkspaceFolder,
): languageclient.LanguageClientOptions {
  const options = {
    documentSelector: SUPPORT_TOML_LANGUAGES.flatMap((language) => [
      { scheme: "file", language },
      { scheme: "untitled", language },
    ]),
    workspaceFolder,
    synchronize: {
      // Notify the server about file changes to tombi.toml and JSON files contained in the workspace
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/tombi.toml"),
        vscode.workspace.createFileSystemWatcher("**/pyproject.toml"),
      ],
    },
  } as languageclient.LanguageClientOptions;

  return options;
}
