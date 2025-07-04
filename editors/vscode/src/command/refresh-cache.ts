import { RequestType } from "vscode-languageclient";
import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";

export type RefreshCacheParams = Record<string, never>;
export const refreshCacheRequest = new RequestType<
  RefreshCacheParams,
  boolean,
  void
>("tombi/refreshCache");

export async function refreshCache(client: node.LanguageClient): Promise<void> {
  try {
    const result = await client.sendRequest(refreshCacheRequest, {});
    if (result) {
      vscode.window.showInformationMessage("Cache refreshed successfully");
    }
  } catch (error) {
    vscode.window.showErrorMessage(`Failed to refresh cache: ${error}`);
  }
}
