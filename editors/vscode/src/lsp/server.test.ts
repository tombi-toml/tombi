import { describe, expect, it } from "vitest";
import { parseTombiVersionOutput } from "./server";

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
