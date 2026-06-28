import { spawn } from "node:child_process";
import { text } from "node:stream/consumers";
import type { TombiBin } from "@/bootstrap";

export const LANGUAGE_SERVER_NAME = "tombi";

export class Server {
  private version?: string;

  constructor(public tombiBin: TombiBin) {}

  async showVersion(): Promise<string> {
    if (this.version === undefined) {
      let version: string;
      try {
        version = await text(
          spawn(this.tombiBin.command, [
            ...this.tombiBin.args,
            "--version",
          ]).stdout.setEncoding("utf-8"),
        );

        version = parseTombiVersionOutput(version);
      } catch {
        version = "<unknown>";
      }

      this.version = version;

      return version;
    }

    return this.version;
  }
}

export function parseTombiVersionOutput(output: string): string {
  const version = output
    .trim()
    .match(/^tombi\s+v?([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/)?.[1];

  return version ?? output.trim();
}
