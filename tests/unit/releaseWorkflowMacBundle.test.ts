import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const releaseWorkflowPath = path.resolve(
  process.cwd(),
  ".gitea/workflows/release.yml",
);

describe("release workflow macOS bundle path", () => {
  it("does not reference the legacy TFTSR.app bundle name", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).not.toContain("/bundle/macos/TFTSR.app");
  });

  it("resolves the macOS .app bundle dynamically", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).toContain("APP=$(find");
    expect(workflow).toContain("-name \"*.app\"");
  });
});
