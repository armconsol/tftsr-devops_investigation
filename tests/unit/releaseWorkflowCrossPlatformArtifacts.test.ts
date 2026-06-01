import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const autoTagWorkflowPath = path.resolve(
  process.cwd(),
  ".github/workflows/release.yml",
);

describe("auto-tag release cross-platform artifact handling", () => {
  it("overrides OpenSSL vendoring for windows-gnu cross builds", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("OPENSSL_NO_VENDOR: \"0\"");
    expect(workflow).toContain("OPENSSL_STATIC: \"1\"");
  });

  it("fails linux uploads when no artifacts are found", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("ERROR: No Linux amd64 artifacts found.");
    expect(workflow).toContain("ERROR: No Linux arm64 artifacts found.");
    expect(workflow).toContain("CI=true npx tauri build");
    expect(workflow).toContain("find src-tauri/target/aarch64-unknown-linux-gnu/release/bundle -type f");
    expect(workflow).toContain("CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc");
    expect(workflow).toContain("PKG_CONFIG_ALLOW_CROSS: \"1\"");
    expect(workflow).toContain("aarch64-unknown-linux-gnu");
  });

  it("fails windows uploads when no artifacts are found", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("ERROR: No Windows amd64 artifacts found.");
  });

  it("replaces existing release assets before uploading reruns", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    expect(workflow).toContain("gh release delete-asset");
    expect(workflow).toContain("gh release upload");
    expect(workflow).toContain("linux-amd64-$(basename");
    expect(workflow).toContain("linux-arm64-$(basename");
  });

  it("uses pre-baked Ubuntu 22.04 cross-compiler image for arm64", () => {
    const workflow = readFileSync(autoTagWorkflowPath, "utf-8");

    // Multiarch ubuntu:22.04 + ports mirror setup moved to pre-baked image;
    // verify workflow references the correct image and cross-compile env vars.
    expect(workflow).toContain("trcaa-linux-arm64:rust1.88-node22");
    expect(workflow).toContain("CC_aarch64_unknown_linux_gnu: aarch64-linux-gnu-gcc");
    expect(workflow).toContain("aarch64-unknown-linux-gnu");
  });
});
