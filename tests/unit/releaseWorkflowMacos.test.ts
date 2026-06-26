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
    "uses Tauri to build the DMG instead of creating it manually",
    () => {
      const workflow = fs.readFileSync(workflowPath, "utf8");

      expect(workflow).toContain(
        "CI=true npx tauri build --target aarch64-apple-darwin --bundles dmg",
      );
      expect(workflow).not.toContain("hdiutil create");
      expect(workflow).not.toContain("codesign --deep --force --sign -");
      expect(workflow).not.toContain("Could not find macOS app bundle");
    },
  );
});
