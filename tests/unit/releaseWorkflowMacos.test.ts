import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";

describe("release workflow macOS packaging", () => {
  it("uses Tauri to build the DMG instead of creating it manually", () => {
    const workflowPath = path.resolve(process.cwd(), ".github/workflows/release.yml");
    const workflow = fs.readFileSync(workflowPath, "utf8");

    expect(workflow).toContain(
      "CI=true npx tauri build --target aarch64-apple-darwin --bundles dmg",
    );
    expect(workflow).not.toContain("hdiutil create");
    expect(workflow).not.toContain("codesign --deep --force --sign -");
    expect(workflow).not.toContain("Could not find macOS app bundle");
  });
});
