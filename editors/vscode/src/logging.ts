import { inspect } from "node:util";
import * as vscode from "vscode";

class Log {
  readonly output = vscode.window.createOutputChannel("Tombi", {
    log: true,
  });

  trace(...messages: [unknown, ...unknown[]]): void {
    this.output.trace(this.stringify(messages));
  }

  debug(...messages: [unknown, ...unknown[]]): void {
    this.output.debug(this.stringify(messages));
  }

  info(...messages: [unknown, ...unknown[]]): void {
    this.output.info(this.stringify(messages));
  }

  warn(...messages: [unknown, ...unknown[]]): void {
    this.output.warn(this.stringify(messages));
  }

  error(...messages: [unknown, ...unknown[]]): void {
    this.output.error(this.stringify(messages));
    this.output.show(true);
  }

  private stringify(messages: unknown[]): string {
    return messages
      .map((message) => {
        if (typeof message === "string") {
          return message;
        }
        if (message instanceof Error) {
          return message.stack || message.message;
        }
        return inspect(message, { depth: 6, colors: false });
      })
      .join(" ");
  }
}

export const log = new Log();
