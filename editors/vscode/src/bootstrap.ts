import * as vscode from "vscode";
import * as os from "node:os";
import type * as extention from "./extention";
import { log } from "@/logging";
import { LANGUAGE_SERVER_BIN_NAME } from "./lsp/server";

export type Env = {
  [name: string]: string;
};

export async function bootstrap(
  context: vscode.ExtensionContext,
  settings: extention.Settings,
): Promise<string> {
  const path = await getServerPath(context, settings);
  if (!path) {
    throw new Error("tombi Language Server is not available.");
  }

  log.info("Using Language Server binary at", path);

  return path;
}

export async function getServerPath(
  context: vscode.ExtensionContext,
  settings: extention.Settings,
): Promise<string | undefined> {
  const packageJson: {
    releaseTag: string | null;
  } = context.extension.packageJSON;

  const explicitPath =
    // biome-ignore lint/complexity/useLiteralKeys: <explanation>
    process.env["__TOML_LSP_SERVER_DEBUG"] ?? settings.server?.path;

  if (explicitPath) {
    if (explicitPath.startsWith("~/")) {
      return os.homedir() + explicitPath.slice("~".length);
    }
    return explicitPath;
  }

  if (packageJson.releaseTag === null) return LANGUAGE_SERVER_BIN_NAME;

  // finally, use the bundled one
  const ext = process.platform === "win32" ? ".exe" : "";
  const bundledUri = vscode.Uri.joinPath(
    context.extensionUri,
    "server",
    LANGUAGE_SERVER_BIN_NAME + ext,
  );
  if (await fileExists(bundledUri)) {
    return bundledUri.fsPath;
  }

  await vscode.window.showErrorMessage(
    "Unfortunately we don't ship binaries for your platform yet. ",
  );

  return undefined;
}


async function fileExists(uri: vscode.Uri) {
  return await vscode.workspace.fs.stat(uri).then(
    () => true,
    () => false,
  );
}