import * as vscode from "vscode";
import type * as languageclient from "vscode-languageclient";
import {
  SUPPORT_TOMBI_CONFIG_FILENAMES,
  SUPPORT_TOML_LANGUAGES,
} from "@/extension";

/**
 * `initializationOptions` of the language server.
 *
 * The shape is the contract with the language server:
 * `{ semanticTokens: { strings: { enable }, comments: { enable } } }`.
 */
function initializationOptions() {
  const configuration = vscode.workspace.getConfiguration("tombi");
  return {
    semanticTokens: {
      strings: {
        enable: configuration.get<boolean>(
          "semanticTokens.strings.enable",
          true,
        ),
      },
      comments: {
        enable: configuration.get<boolean>(
          "semanticTokens.comments.enable",
          true,
        ),
      },
    },
  };
}

export function clientOptions(
  workspaceFolder?: vscode.WorkspaceFolder,
): languageclient.LanguageClientOptions {
  const options = {
    diagnosticCollectionName: "tombi",
    documentSelector: SUPPORT_TOML_LANGUAGES.flatMap((language) => [
      { scheme: "file", language },
      { scheme: "untitled", language },
    ]),
    workspaceFolder,
    // NOTE: A function is evaluated on every (re)start of the language server,
    //       so that `Restart Language Server` picks up the changed settings.
    initializationOptions,
    synchronize: {
      // Notify the server about config file changes contained in the workspace.
      fileEvents: SUPPORT_TOMBI_CONFIG_FILENAMES.map((filename) =>
        vscode.workspace.createFileSystemWatcher(`**/${filename}`),
      ),
    },
  } as languageclient.LanguageClientOptions;

  return options;
}
