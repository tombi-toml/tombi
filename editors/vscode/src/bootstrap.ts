import { exec } from "node:child_process";
import * as os from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import type * as extension from "@/extension";
import { log } from "@/logging";
import { LANGUAGE_SERVER_NAME } from "@/lsp/server";
import { getInterpreterDetails } from "@/python";

export type Env = {
  [name: string]: string;
};

export type TombiBin = {
  source:
    | "bundled"
    | "dev"
    | "editor settings"
    | "local"
    | "venv"
    | "node_modules";
  path: string;
};

/**
 * Bootstrap the Language Server binary.
 */
export async function bootstrap(
  context: vscode.ExtensionContext,
  settings: extension.Settings,
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
  settings: extension.Settings,
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

  for (const workspace of vscode.workspace.workspaceFolders ?? []) {
    // Check for tombi in Python virtual environment
    const venvBinPath = await findVenvTombiBin(binName, workspace.uri);
    if (venvBinPath) {
      return {
        source: "venv",
        path: venvBinPath,
      };
    }

    // Check for tombi in node_modules/.bin
    const nodeModulesBinPath = await findNodeModulesTombiBin(
      binName,
      workspace.uri,
    );
    if (nodeModulesBinPath) {
      return {
        source: "node_modules",
        path: nodeModulesBinPath,
      };
    }
  }

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

async function findVenvTombiBin(
  binName: string,
  workspaceUri: vscode.Uri,
): Promise<string | undefined> {
  const interpreterDetails = await getInterpreterDetails(workspaceUri);
  if (!interpreterDetails.pythonPath) {
    const venvBinPath = vscode.Uri.joinPath(
      workspaceUri,
      ".venv",
      process.platform === "win32" ? "Scripts" : "bin",
      binName,
    );

    if (await fileExists(venvBinPath)) {
      return venvBinPath.fsPath;
    }

    return undefined;
  }

  const pythonPath = interpreterDetails.pythonPath;
  const binDir = path.dirname(pythonPath);

  const tombiPath = path.join(binDir, binName);
  if (await fileExists(vscode.Uri.file(tombiPath))) {
    return tombiPath;
  }

  return undefined;
}

async function findNodeModulesTombiBin(
  _binName: string,
  workspaceUri: vscode.Uri,
): Promise<string | undefined> {
  const nodeModulesBinPath = vscode.Uri.joinPath(
    workspaceUri,
    "node_modules",
    ".bin",
    "tombi", // NOTE: npm installs the binary as "tombi"
  );

  if (await fileExists(nodeModulesBinPath)) {
    return nodeModulesBinPath.fsPath;
  }

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
