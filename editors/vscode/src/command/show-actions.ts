import * as vscode from "vscode";

type ActionItem = vscode.QuickPickItem & {
  command: string;
};

export async function showActions(commandPrefix: string): Promise<void> {
  const items: ActionItem[] = [
    {
      label: "Select Schema",
      command: `${commandPrefix}.selectSchema`,
    },
    {
      label: "Refresh Cache",
      command: `${commandPrefix}.refreshCache`,
    },
    {
      label: "Restart Language Server",
      command: `${commandPrefix}.restartLanguageServer`,
    },
    {
      label: "Show Language Server Version",
      command: `${commandPrefix}.showLanguageServerVersion`,
    },
    {
      label: "Open Server Logs",
      command: `${commandPrefix}.openServerLogs`,
    },
  ];

  const selected = await vscode.window.showQuickPick(items, {
    placeHolder: "Select a Tombi command",
  });

  if (!selected) {
    return;
  }

  await vscode.commands.executeCommand(selected.command);
}
