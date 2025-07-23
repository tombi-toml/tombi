import type { TombiBin } from "@/bootstrap";
import { spawn } from "node:child_process";
import { text } from "node:stream/consumers";

export const LANGUAGE_SERVER_NAME = "tombi";

export class Server {
  private version?: string;

  constructor(public tombiBin: TombiBin) {}

  async showVersion(): Promise<string> {
    if (this.version === undefined) {
      let version: string;
      try {
        version = await text(
          spawn(this.tombiBin.path, ["--version"]).stdout.setEncoding("utf-8"),
        );

        const prefix = LANGUAGE_SERVER_NAME;
        version = version
          .slice(version.startsWith(prefix) ? prefix.length : 0)
          .trim();
      } catch {
        version = "<unknown>";
      }

      this.version = version;

      return version;
    }

    return this.version;
  }
}
