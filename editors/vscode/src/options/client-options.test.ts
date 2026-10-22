import { once } from "node:events";
import { PassThrough } from "node:stream";
import { beforeEach, describe, expect, it, vi } from "vitest";
import * as vscode from "vscode";
import { clientOptions } from "./client-options";

const settings = vi.hoisted(() => ({ values: {} as Record<string, unknown> }));
const outputChannel = vi.hoisted(() => ({
  error: vi.fn(),
  warn: vi.fn(),
  info: vi.fn(),
  debug: vi.fn(),
  trace: vi.fn(),
}));

vi.mock("vscode", () => ({
  window: {
    createOutputChannel: vi.fn(() => outputChannel),
  },
  workspace: {
    getConfiguration: vi.fn(() => ({
      get: vi.fn((key: string, defaultValue: unknown) =>
        key in settings.values ? settings.values[key] : defaultValue,
      ),
    })),
    createFileSystemWatcher: vi.fn(() => ({})),
  },
}));

vi.mock("@/extension", () => ({
  SUPPORT_TOMBI_CONFIG_FILENAMES: ["tombi.toml"],
  SUPPORT_TOML_LANGUAGES: ["toml"],
}));

function initializationOptions(): unknown {
  const options = clientOptions().initializationOptions;
  if (typeof options !== "function") {
    throw new Error("Expected initializationOptions to be a function");
  }
  return options();
}

describe("clientOptions initializationOptions", () => {
  beforeEach(() => {
    settings.values = {};
    vi.clearAllMocks();
  });

  it("sends every semantic token by default", () => {
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enabled: true },
        comments: { enabled: true },
      },
    });
  });

  it("passes disabled settings", () => {
    settings.values = {
      "semanticTokens.strings.enabled": false,
      "semanticTokens.comments.enabled": false,
    };
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enabled: false },
        comments: { enabled: false },
      },
    });
  });

  it("passes partially specified settings", () => {
    settings.values = { "semanticTokens.strings.enabled": false };
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enabled: false },
        comments: { enabled: true },
      },
    });
  });

  it("reads the settings again on every call", () => {
    expect(initializationOptions()).toMatchObject({
      semanticTokens: { strings: { enabled: true } },
    });
    settings.values = { "semanticTokens.strings.enabled": false };
    expect(initializationOptions()).toMatchObject({
      semanticTokens: { strings: { enabled: false } },
    });
  });
});

describe("clientOptions stdio log handlers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  async function forwardLogs(
    streamName: "stdout" | "stderr",
    contents: string,
  ): Promise<void> {
    const stdioOptions = clientOptions().stdioOptions;
    if (!stdioOptions) {
      throw new Error("Expected stdioOptions to be configured");
    }

    const input = new PassThrough();
    const inputEnded = once(input, "end");
    const channel = vscode.window.createOutputChannel("Tombi", { log: true });
    stdioOptions[streamName](input, channel);
    input.end(contents);
    await inputEnded;
  }

  it("uses the prefixed level and removes the server timestamp", async () => {
    await forwardLogs(
      "stdout",
      [
        "2026-10-21 08:54:16.431 [ERROR] error message",
        "2026-10-21 08:54:17.431 [WARN] warning message",
        "2026-10-21 08:54:18.431 [INFO] handle_workspace_diagnostic",
        "2026-10-21 08:54:19.431 [DEBUG] debug message",
        "2026-10-21 08:54:20.431 [TRACE] trace message",
        "",
      ].join("\n"),
    );

    expect(outputChannel.error).toHaveBeenCalledExactlyOnceWith(
      "error message",
    );
    expect(outputChannel.warn).toHaveBeenCalledExactlyOnceWith(
      "warning message",
    );
    expect(outputChannel.info).toHaveBeenCalledExactlyOnceWith(
      "handle_workspace_diagnostic",
    );
    expect(outputChannel.debug).toHaveBeenCalledExactlyOnceWith(
      "debug message",
    );
    expect(outputChannel.trace).toHaveBeenCalledExactlyOnceWith(
      "trace message",
    );
  });

  it("keeps Tombi's prefixed level for continuation lines", async () => {
    await forwardLogs(
      "stdout",
      "   Info: handle_workspace_diagnostic\ncontinued detail\n",
    );

    expect(outputChannel.info).toHaveBeenNthCalledWith(
      1,
      "handle_workspace_diagnostic",
    );
    expect(outputChannel.info).toHaveBeenNthCalledWith(2, "continued detail");
  });

  it("uses the stream severity when a line has no level prefix", async () => {
    await forwardLogs("stdout", "plain stdout message\n");
    await forwardLogs("stderr", "plain stderr message\n");

    expect(outputChannel.info).toHaveBeenCalledExactlyOnceWith(
      "plain stdout message",
    );
    expect(outputChannel.error).toHaveBeenCalledExactlyOnceWith(
      "plain stderr message",
    );
  });
});
