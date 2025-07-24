import * as vscode from "vscode";

import type { PythonExtension } from "@vscode/python-extension";

export interface IInterpreterDetails {
  pythonPath?: string;
  resource?: vscode.Uri;
}

async function activateExtension() {
  const extension = vscode.extensions.getExtension("ms-python.python");
  if (extension) {
    if (!extension.isActive) {
      await extension.activate();
    }
  }
  return extension;
}

let _api: PythonExtension | undefined;
async function getPythonExtensionAPI(): Promise<PythonExtension | undefined> {
  if (!_api) {
    const extension = await activateExtension();
    _api = extension?.exports;
  }
  return _api;
}

export async function getInterpreterDetails(
  resource?: vscode.Uri,
): Promise<IInterpreterDetails> {
  const api = await getPythonExtensionAPI();
  const environment = await api?.environments.resolveEnvironment(
    api?.environments.getActiveEnvironmentPath(resource),
  );
  if (environment?.executable.uri) {
    return {
      pythonPath: environment.executable.uri.fsPath,
      resource,
    } as IInterpreterDetails;
  }
  return { resource } as IInterpreterDetails;
}
