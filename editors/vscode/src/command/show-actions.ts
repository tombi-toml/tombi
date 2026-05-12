import * as vscode from "vscode";

type ActionItem = vscode.QuickPickItem & {
  command: string;
};

export async function showActions(commandPrefix: string): Promise<void> {
  const items: ActionItem[] = [
    {
      label: "Select Schema",
      iconPath: new vscode.ThemeIcon("json"),
      command: `${commandPrefix}.selectSchema`,
    },
    {
      label: "Refresh Cache",
      iconPath: new vscode.ThemeIcon("trash"),
      command: `${commandPrefix}.refreshCache`,
    },
    {
      label: "Restart Language Server",
      iconPath: new vscode.ThemeIcon("refresh"),
      command: `${commandPrefix}.restartLanguageServer`,
    },
    {
      label: "Open Language Server Logs",
      iconPath: new vscode.ThemeIcon("output"),
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
