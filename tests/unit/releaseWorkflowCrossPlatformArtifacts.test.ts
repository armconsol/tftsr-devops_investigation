import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const autoTagWorkflowPath = path.resolve(
  process.cwd(),
  ".gitea/workflows/auto-tag.yml",
);

describe("auto-tag release cross-platform artifact handling", () => {
  it("overrides OpenSSL vendoring for windows-gnu cross builds", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("OPENSSL_NO_VENDOR: \"0\"");
    expect(workflow).toContain("OPENSSL_STATIC: \"1\"");
  });

  it("fails linux uploads when no artifacts are found", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("ERROR: No Linux amd64 artifacts were found to upload.");
    expect(workflow).toContain("ERROR: No Linux arm64 artifacts were found to upload.");
    expect(workflow).toContain("CI=true npx tauri build");
    expect(workflow).toContain("find src-tauri/target/aarch64-unknown-linux-gnu/release/bundle -type f");
    expect(workflow).toContain("CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc");
    expect(workflow).toContain("PKG_CONFIG_ALLOW_CROSS: \"1\"");
    expect(workflow).toContain("aarch64-unknown-linux-gnu");
  });

  it("fails windows uploads when no artifacts are found", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain(
      "ERROR: No Windows amd64 artifacts were found to upload.",
    );
  });

  it("replaces existing release assets before uploading reruns", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("Deleting existing asset id=$id name=$NAME before upload...");
    expect(workflow).toContain("-X DELETE \"$API/releases/$RELEASE_ID/assets/$id\"");
    expect(workflow).toContain("UPLOAD_NAME=\"linux-amd64-$NAME\"");
    expect(workflow).toContain("UPLOAD_NAME=\"linux-arm64-$NAME\"");
  });

  it("uses Ubuntu 22.04 with ports mirror for arm64 cross-compile", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("ubuntu:22.04");
    expect(workflow).toContain("ports.ubuntu.com/ubuntu-ports");
    expect(workflow).toContain("jammy");
  });
});
