import { describe, expect, it } from "vitest";
import { parseTombiVersionOutput, Server } from "./server";

describe("parseTombiVersionOutput", () => {
  it("keeps the plain semver from legacy version output", () => {
    expect(parseTombiVersionOutput("tombi 1.1.5\n")).toBe("1.1.5");
  });

  it("drops target information from version output before semver comparisons", () => {
    expect(
      parseTombiVersionOutput("tombi 1.1.5 (aarch64-apple-darwin)\n"),
    ).toBe("1.1.5");
  });

  it("keeps prerelease suffixes such as the development version", () => {
    expect(
      parseTombiVersionOutput("tombi 0.0.0-dev (aarch64-apple-darwin)\n"),
    ).toBe("0.0.0-dev");
  });
});

describe("Server.update", () => {
  it("switches the binary and drops the cached version", async () => {
    const server = new Server({
      source: "bundled",
      command: "node",
      args: ["-p", "'tombi 1.0.0'", "--"],
      binPath: "a",
    } as never);
    expect(await server.showVersion()).toBe("1.0.0");

    server.update({
      source: "venv",
      command: "node",
      args: ["-p", "'tombi 2.0.0'", "--"],
      binPath: "b",
    } as never);
    expect(server.tombiBin.source).toBe("venv");
    expect(await server.showVersion()).toBe("2.0.0");
  });
});
