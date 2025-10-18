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

  const run = {
    command: serverPath,
    args,
    options: {
      env: {
        NO_COLOR: "1",
      },
    },
  };

  return {
    run,
    debug: run,
  };
}
