import * as vscode from "vscode";
import { EXTENSION_ID } from "@/extension";

type ActionItem = vscode.QuickPickItem & {
  command: string;
};

export async function showActions(): Promise<void> {
  const items: ActionItem[] = [
    {
      label: "Select Schema",
      command: "selectSchema",
    },
    {
      label: "Refresh Cache",
      command: "refreshCache",
    },
    {
      label: "Restart Language Server",
      command: "restartLanguageServer",
    },
    {
      label: "Show Language Server Version",
      command: "showLanguageServerVersion",
    },
    {
      label: "Open Server Logs",
      command: "openServerLogs",
    },
  ];

  const selected = await vscode.window.showQuickPick(items, {
    placeHolder: "Select a Tombi command",
  });

  if (!selected) {
    return;
  }

  await vscode.commands.executeCommand(`${EXTENSION_ID}.${selected.command}`);
}
