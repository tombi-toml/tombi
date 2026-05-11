import { describe, expect, it, vi } from "vitest";

vi.mock("vscode", () => ({}));
vi.mock("vscode-languageclient/node", () => ({}));
vi.mock("@/lsp/client", () => ({
  getBuildInSchema: {},
}));

import { isLocalFilePath } from "@/command/open-tooltip-link";

describe("isLocalFilePath", () => {
  it("accepts POSIX absolute paths", () => {
    expect(isLocalFilePath("/Users/test/project/tombi.toml")).toBe(true);
  });

  it("accepts Windows absolute paths", () => {
    expect(isLocalFilePath("C:\\Users\\test\\project\\tombi.toml")).toBe(true);
  });

  it("rejects relative paths", () => {
    expect(isLocalFilePath("tombi/config.toml")).toBe(false);
  });
});
