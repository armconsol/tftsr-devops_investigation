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
});
