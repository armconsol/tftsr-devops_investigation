import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";

describe("release workflow macOS packaging", () => {
  const workflowPath = path.resolve(
    process.cwd(),
    ".github/workflows/release.yml",
  );
  const workflowExists = fs.existsSync(workflowPath);

  // The release workflow lives only on the GitHub mirror. The authoritative
  // Gitea repo excludes .github/ from the two-way sync, so this asset is absent
  // there; skip the assertions when the workflow file is not present.
  it.skipIf(!workflowExists)(
    "ad-hoc signs the .app and builds the DMG with hdiutil",
    () => {
      const workflow = fs.readFileSync(workflowPath, "utf8");

      // Build the unpackaged .app, then sign and package it manually so the
      // DMG ships a proper ad-hoc signature with sealed resources. Building
      // straight to a DMG (`--bundles dmg`) yields a linker-signed app with
      // unsealed resources that Gatekeeper rejects as "damaged".
      expect(workflow).toContain(
        "CI=true npx tauri build --target aarch64-apple-darwin --bundles app",
      );
      expect(workflow).toContain("codesign --deep --force --sign -");
      expect(workflow).toContain("hdiutil create");
      expect(workflow).not.toContain(
        "CI=true npx tauri build --target aarch64-apple-darwin --bundles dmg",
      );
    },
  );
});
