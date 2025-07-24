import * as vscode from "vscode";
import * as os from "node:os";
import type * as extention from "./extention";
import { log } from "@/logging";
import { LANGUAGE_SERVER_NAME } from "./lsp/server";
import { exec } from "node:child_process";

export type Env = {
  [name: string]: string;
};

export type TombiBin = {
  source: "bundled" | "dev" | "editor settings" | "local";
  path: string;
};

/**
 * Bootstrap the Language Server binary.
 */
export async function bootstrap(
  context: vscode.ExtensionContext,
  settings: extention.Settings,
): Promise<TombiBin> {
  const tombiBin = await getTombiBin(context, settings);
  if (!tombiBin) {
    throw new Error("tombi Language Server is not available.");
  }

  log.info("Using Language Server binary at", tombiBin.path);

  return tombiBin;
}

async function getTombiBin(
  context: vscode.ExtensionContext,
  settings: extention.Settings,
): Promise<TombiBin | undefined> {
  let settingsPath = settings.path;
  if (settingsPath) {
    if (settingsPath.startsWith("~/")) {
      settingsPath = os.homedir() + settingsPath.slice("~".length);
    }
    return {
      source: "editor settings",
      path: settingsPath,
    };
  }

  // biome-ignore lint/complexity/useLiteralKeys: process.env properties require bracket notation
  const developPath = process.env["__TOMBI_LANGUAGE_SERVER_DEBUG"];
  if (developPath) {
    return {
      source: "dev",
      path: developPath,
    };
  }

  const ext = process.platform === "win32" ? ".exe" : "";
  const binName = LANGUAGE_SERVER_NAME + ext;

  // use local tombi binary
  const localBinPath = await findLocalTombiBin(binName);
  if (localBinPath) {
    return {
      source: "local",
      path: localBinPath,
    };
  }

  // finally, use the bundled one
  const bundledUri = vscode.Uri.joinPath(
    context.extensionUri,
    "server",
    binName,
  );

  if (await fileExists(bundledUri)) {
    return {
      source: "bundled",
      path: bundledUri.fsPath,
    };
  }

  await vscode.window.showErrorMessage(
    "Unfortunately we don't ship binaries for your platform yet. ",
  );

  return undefined;
}

async function findLocalTombiBin(binName: string): Promise<string | undefined> {
  try {
    // Check if tombi is available in PATH
    const command =
      process.platform === "win32" ? `where ${binName}` : `which ${binName}`;
    const result = await new Promise<{ stdout: string; stderr: string }>(
      (resolve, reject) => {
        exec(command, (error, stdout, stderr) => {
          if (error) {
            reject(error);
          } else {
            resolve({ stdout, stderr });
          }
        });
      },
    );

    const path = result.stdout.trim();
    if (path) {
      return path;
    }
  } catch (_error: unknown) {
    // tombi not found in PATH, continue to bundled binary
  }

  return undefined;
}

async function fileExists(uri: vscode.Uri) {
  return await vscode.workspace.fs.stat(uri).then(
    () => true,
    () => false,
  );
}
