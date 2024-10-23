import { SUPPORT_LANGUAGES } from "@/extention";
import * as vscode from "vscode";
import type * as languageclient from "vscode-languageclient";

export function clientOptions(
  workspaceFolder?: vscode.WorkspaceFolder,
): languageclient.LanguageClientOptions {
  const options = {
    documentSelector: SUPPORT_LANGUAGES.map((language) => ({
      scheme: "file",
      language,
    })),
    workspaceFolder,
    synchronize: {
      // Notify the server about file changes to tombi.toml and JSON files contained in the workspace
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/tombi.toml"),
        vscode.workspace.createFileSystemWatcher("**/pyproject.toml"),
      ],
    },
    middleware: {
      async provideDocumentFormattingEdits(
        document,
      ): Promise<vscode.TextEdit[]> {
        return [
          vscode.TextEdit.insert(
            new vscode.Position(10, 0),
            "This is a formatting test",
          ),
        ];
      },
    },
  } as languageclient.LanguageClientOptions;

  return options;
}
