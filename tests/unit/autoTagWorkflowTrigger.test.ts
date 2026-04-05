import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const autoTagWorkflowPath = path.resolve(
  process.cwd(),
  ".gitea/workflows/auto-tag.yml",
);

describe("auto-tag workflow release triggering", () => {
  it("creates tags via git push instead of Gitea tag API", () => {
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
    expect(workflow).toContain("needs: auto-tag");
    expect(workflow).toContain("needs.auto-tag.outputs.tag_created == 'true'");
    expect(workflow).toContain("TAG=\"${{ needs.auto-tag.outputs.release_tag }}\"");
  });
});
