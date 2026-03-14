import { describe, expect, it, vi } from "vitest";
import type { TombiBin } from "@/bootstrap";
import { serverOptions } from "./server-options";

type ExecutableRun = {
  command: string;
  args: string[];
};

vi.mock("vscode", () => ({
  workspace: {
    getConfiguration: vi.fn(() => ({
      get: vi.fn(() => undefined),
    })),
  },
}));

describe("serverOptions", () => {
  it("runs direct binaries without a runtime wrapper", () => {
    const tombiBin: TombiBin = {
      source: "bundled",
      binPath: "/tmp/tombi",
      command: "/tmp/tombi",
      args: [],
    };

    const options = serverOptions(tombiBin, {
      args: ["serve"],
      env: { FOO: "bar" },
    });
    expect("run" in options).toBe(true);
    if (!("run" in options)) {
      throw new Error("Expected language server options");
    }
    const run = options.run as ExecutableRun;

    expect(run.command).toBe("/tmp/tombi");
    expect(run.args).toEqual(["lsp", "serve"]);
  });

  it("runs node_modules installs through node", () => {
    const tombiBin: TombiBin = {
      source: "node_modules",
      binPath: "/tmp/node_modules/@tombi-toml/tombi/bin/tombi",
      command: process.execPath,
      args: ["/tmp/node_modules/@tombi-toml/tombi/bin/tombi"],
    };

    const options = serverOptions(tombiBin, {});
    expect("run" in options).toBe(true);
    if (!("run" in options)) {
      throw new Error("Expected language server options");
    }
    const run = options.run as ExecutableRun;

    expect(run.command).toBe(process.execPath);
    expect(run.args).toEqual([
      "/tmp/node_modules/@tombi-toml/tombi/bin/tombi",
      "lsp",
    ]);
  });

  it("runs node_modules shims directly when no node script is available", () => {
    const tombiBin: TombiBin = {
      source: "node_modules",
      binPath: "/tmp/node_modules/.bin/tombi",
      command: "/tmp/node_modules/.bin/tombi",
      args: [],
    };

    const options = serverOptions(tombiBin, {});
    expect("run" in options).toBe(true);
    if (!("run" in options)) {
      throw new Error("Expected language server options");
    }
    const run = options.run as ExecutableRun;

    expect(run.command).toBe("/tmp/node_modules/.bin/tombi");
    expect(run.args).toEqual(["lsp"]);
  });
});
