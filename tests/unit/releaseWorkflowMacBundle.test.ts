import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const autoTagWorkflowPath = path.resolve(
  process.cwd(),
  ".github/workflows/release.yml",
);

describe("auto-tag release macOS bundle path", () => {
  it("does not reference the legacy TFTSR.app bundle name", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).not.toContain("/bundle/macos/TFTSR.app");
  });

  it("resolves the macOS .app bundle dynamically", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("APP=$(find");
    expect(workflow).toContain("-name \"*.app\"");
  });
});
