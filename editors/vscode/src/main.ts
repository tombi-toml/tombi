import type * as vscode from "vscode";
import { Extension } from "@/extension";

let extension: Extension;

export async function activate(
  context: vscode.ExtensionContext,
): Promise<void> {
  if (!extension) {
    extension = await Extension.activate(context);
  }
}

export async function deactivate(): Promise<void> {
  extension?.deactivate();
}
