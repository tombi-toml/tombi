import { describe, expect, it, vi } from "vitest";

vi.mock("vscode", () => ({
  Uri: { file: (path: string) => ({ toString: () => `file://${path}` }) },
}));
vi.mock("vscode-languageclient/node", () => ({}));
vi.mock("@/lsp/client", () => ({
  getBuiltInSchema: {},
}));

import { toAssociateSchemaParams } from "@/command/associate-schema";

describe("toAssociateSchemaParams", () => {
  it("converts path to a file URI and normalizes fileMatch", () => {
    expect(
      toAssociateSchemaParams({
        path: "/tmp/schema.json",
        fileMatch: "**/foo.toml",
        force: true,
      }),
    ).toEqual({
      uri: "file:///tmp/schema.json",
      fileMatch: ["**/foo.toml"],
      force: true,
    });
  });

  it("rejects relative paths", () => {
    expect(() =>
      toAssociateSchemaParams({ path: "schema.json", fileMatch: ["*.toml"] }),
    ).toThrow();
  });

  it("rejects missing fileMatch", () => {
    expect(() =>
      toAssociateSchemaParams({ path: "/tmp/schema.json" }),
    ).toThrow();
  });
});
