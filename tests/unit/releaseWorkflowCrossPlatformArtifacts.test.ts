import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const releaseWorkflowPath = path.resolve(
  process.cwd(),
  ".gitea/workflows/release.yml",
);

describe("release workflow cross-platform artifact handling", () => {
  it("overrides OpenSSL vendoring for windows-gnu cross builds", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).toContain("OPENSSL_NO_VENDOR: \"0\"");
    expect(workflow).toContain("OPENSSL_STATIC: \"1\"");
  });

  it("fails linux uploads when no artifacts are found", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).toContain("ERROR: No Linux amd64 artifacts were found to upload.");
    expect(workflow).toContain("ERROR: No Linux arm64 artifacts were found to upload.");
  });

  it("fails windows uploads when no artifacts are found", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).toContain(
      "ERROR: No Windows amd64 artifacts were found to upload.",
    );
  });

  it("replaces existing release assets before uploading reruns", () => {
    const workflow = readFileSync(releaseWorkflowPath, "utf-8");

    expect(workflow).toContain("Deleting existing asset id=$id name=$NAME before upload...");
    expect(workflow).toContain("-X DELETE \"$API/releases/$RELEASE_ID/assets/$id\"");
  });
});
