import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";
import yaml from "js-yaml";

const autoTagPath = path.resolve(process.cwd(), ".gitea/workflows/auto-tag.yml");
const releaseBetaPath = path.resolve(process.cwd(), ".gitea/workflows/release-beta.yml");
const tauriConfPath = path.resolve(process.cwd(), "src-tauri/tauri.conf.json");

describe("auto-tag.yml version embedding", () => {
  const workflow = fs.readFileSync(autoTagPath, "utf8");

  it("does not use the BusyBox-incompatible '0,/re/s//repl/' sed range form", () => {
    expect(workflow).not.toMatch(/sed -i "0,\//);
  });

  it("rewrites the version files via the shared node script", () => {
    expect(workflow).toContain('node scripts/update-version.mjs "${NEW_VERSION}"');
  });

  it("installs nodejs in the autotag job's alpine container", () => {
    expect(workflow).toMatch(/apk add --no-cache[^\n]*\bnodejs\b/);
  });

  it("verifies every version-bearing file was actually updated before committing", () => {
    expect(workflow).toContain("was not updated to");
  });

  it("every downstream job that builds exports RELEASE_TAG from the autotag job", () => {
    const parsed = yaml.load(workflow) as {
      jobs: Record<string, { steps?: { run?: string }[]; env?: Record<string, string> }>;
    };
    const buildJobs = Object.entries(parsed.jobs).filter(([name]) =>
      /build/i.test(name)
    );
    expect(buildJobs.length).toBeGreaterThan(0);
    for (const [name, job] of buildJobs) {
      const hasReleaseTag = JSON.stringify(job).includes("needs.autotag.outputs.release_tag");
      expect(hasReleaseTag, `job "${name}" should reference needs.autotag.outputs.release_tag`).toBe(
        true
      );
    }
  });
});

describe("release-beta.yml version embedding", () => {
  const workflow = fs.readFileSync(releaseBetaPath, "utf8");

  it("every build job references the autotag release_tag output", () => {
    const parsed = yaml.load(workflow) as {
      jobs: Record<string, { steps?: { run?: string }[] }>;
    };
    const buildJobs = Object.entries(parsed.jobs).filter(([name]) => /build/i.test(name));
    expect(buildJobs.length).toBeGreaterThan(0);
    for (const [name, job] of buildJobs) {
      const hasReleaseTag = JSON.stringify(job).includes("needs.autotag.outputs.release_tag");
      expect(hasReleaseTag, `job "${name}" should reference needs.autotag.outputs.release_tag`).toBe(
        true
      );
    }
  });
});

describe("tauri.conf.json build hook", () => {
  it("still runs version:update before building so RELEASE_TAG is embedded", () => {
    const conf = JSON.parse(fs.readFileSync(tauriConfPath, "utf8"));
    expect(conf.build.beforeBuildCommand).toContain("npm run version:update");
  });
});
