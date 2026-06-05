import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const autoTagWorkflowPath = path.resolve(
  process.cwd(),
  ".github/workflows/release.yml",
);

describe("auto-tag workflow release triggering", () => {
  it("creates tags via git push instead of API call", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("git push origin \"refs/tags/$NEXT\"");
    expect(workflow).not.toContain("POST \"$API/tags\"");
  });

  it("runs release build jobs after auto-tag succeeds", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("build-linux-amd64:");
    expect(workflow).toContain("build-windows-amd64:");
    expect(workflow).toContain("build-macos-arm64:");
    expect(workflow).toContain("build-linux-arm64:");
    expect(workflow).toContain("needs: autotag");
    expect(workflow).toContain("git tag --sort=-version:refname");
  });

  it("uses --clobber for artifact uploads to handle re-runs cleanly", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    const clobberCount = (workflow.match(/--clobber/g) ?? []).length;
    expect(clobberCount).toBeGreaterThanOrEqual(4);
  });
});
