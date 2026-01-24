import * as vscode from "vscode";
import type * as node from "vscode-languageclient/node";
import type { Settings } from "@/extension";

export function serverOptions(
  serverPath: string,
  settings: Settings,
): node.ServerOptions {
  const args: string[] = settings.args ?? [];
  if (args[0] !== "lsp") {
    args.unshift("lsp");
  }

  const proxyEnv = getProxyEnv();

  const run = {
    command: serverPath,
    args,
    options: {
      env: {
        ...process.env,
        NO_COLOR: "1",
        ...proxyEnv,
        ...settings.env,
      },
    },
  };

  return {
    run,
    debug: run,
  };
}

function getProxyEnv(): Record<string, string> {
  const httpConfig = vscode.workspace.getConfiguration("http");
  const proxyEnv: Record<string, string> = {};

  const proxy = httpConfig.get<string>("proxy");
  if (proxy) {
    // biome-ignore lint/complexity/useLiteralKeys: process.env properties require bracket notation
    proxyEnv["HTTPS_PROXY"] = proxy;
  }

  return proxyEnv;
}
