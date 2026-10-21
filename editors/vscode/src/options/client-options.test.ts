import { beforeEach, describe, expect, it, vi } from "vitest";
import { clientOptions } from "./client-options";

const settings = vi.hoisted(() => ({ values: {} as Record<string, unknown> }));

vi.mock("vscode", () => ({
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
  });

  it("sends every semantic token by default", () => {
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enable: true },
        comments: { enable: true },
      },
    });
  });

  it("passes disabled settings", () => {
    settings.values = {
      "semanticTokens.strings.enable": false,
      "semanticTokens.comments.enable": false,
    };
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enable: false },
        comments: { enable: false },
      },
    });
  });

  it("passes partially specified settings", () => {
    settings.values = { "semanticTokens.strings.enable": false };
    expect(initializationOptions()).toEqual({
      semanticTokens: {
        strings: { enable: false },
        comments: { enable: true },
      },
    });
  });

  it("reads the settings again on every call", () => {
    expect(initializationOptions()).toMatchObject({
      semanticTokens: { strings: { enable: true } },
    });
    settings.values = { "semanticTokens.strings.enable": false };
    expect(initializationOptions()).toMatchObject({
      semanticTokens: { strings: { enable: false } },
    });
  });
});
