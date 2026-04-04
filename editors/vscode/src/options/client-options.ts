import * as vscode from "vscode";
import type * as languageclient from "vscode-languageclient";
import {
  SUPPORT_TOMBI_CONFIG_FILENAMES,
  SUPPORT_TOML_LANGUAGES,
} from "@/extension";

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
      // Notify the server about config file changes contained in the workspace.
      fileEvents: SUPPORT_TOMBI_CONFIG_FILENAMES.map((filename) =>
        vscode.workspace.createFileSystemWatcher(`**/${filename}`),
      ),
    },
  } as languageclient.LanguageClientOptions;

  return options;
}
