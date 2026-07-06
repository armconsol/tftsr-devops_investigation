import { describe, it, expect } from "vitest";
import { isValidVersion, stripV, resolveVersion } from "../../scripts/update-version.mjs";

describe("isValidVersion", () => {
  it("accepts plain semver", () => {
    expect(isValidVersion("3.1.0")).toBe(true);
  });

  it("accepts prerelease semver", () => {
    expect(isValidVersion("3.1.0-beta.9")).toBe(true);
  });

  it("rejects malformed versions", () => {
    expect(isValidVersion("3.1")).toBe(false);
    expect(isValidVersion("v3.1.0")).toBe(false);
    expect(isValidVersion("")).toBe(false);
    expect(isValidVersion("not-a-version")).toBe(false);
  });
});

describe("stripV", () => {
  it("strips a leading v", () => {
    expect(stripV("v3.1.0")).toBe("3.1.0");
  });

  it("leaves a version without a leading v unchanged", () => {
    expect(stripV("3.1.0")).toBe("3.1.0");
  });
});

describe("resolveVersion precedence", () => {
  it("prefers an explicit argument over everything else", () => {
    const version = resolveVersion({
      argVersion: "v3.2.0",
      releaseTagEnv: "v3.1.0",
      gitDescribe: () => "v3.0.0",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.2.0");
  });

  it("falls back to RELEASE_TAG when no argument is given", () => {
    const version = resolveVersion({
      argVersion: undefined,
      releaseTagEnv: "v3.1.0-beta.4",
      gitDescribe: () => "v3.0.0",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.1.0-beta.4");
  });

  it("falls back to git describe when argument and RELEASE_TAG are absent", () => {
    const version = resolveVersion({
      argVersion: undefined,
      releaseTagEnv: undefined,
      gitDescribe: () => "v3.0.0",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.0.0");
  });

  it("falls back to package.json when git describe throws", () => {
    const version = resolveVersion({
      argVersion: undefined,
      releaseTagEnv: undefined,
      gitDescribe: () => {
        throw new Error("no tags");
      },
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("2.0.0");
  });

  it("skips an invalid explicit argument and falls through", () => {
    const version = resolveVersion({
      argVersion: "not-a-version",
      releaseTagEnv: "v3.1.0",
      gitDescribe: () => "v3.0.0",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.1.0");
  });

  it("skips an invalid RELEASE_TAG and falls through to git describe", () => {
    const version = resolveVersion({
      argVersion: undefined,
      releaseTagEnv: "garbage",
      gitDescribe: () => "v3.0.0",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.0.0");
  });

  it("accepts a prerelease tag from git describe", () => {
    const version = resolveVersion({
      argVersion: undefined,
      releaseTagEnv: undefined,
      gitDescribe: () => "v3.1.0-beta.2",
      packageJsonFallback: () => "2.0.0",
    });
    expect(version).toBe("3.1.0-beta.2");
  });
});
