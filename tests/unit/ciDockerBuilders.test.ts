import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();

const readFile = (rel: string) => readFileSync(path.resolve(root, rel), "utf-8");

// ─── Dockerfiles ─────────────────────────────────────────────────────────────

describe("Dockerfile.linux-amd64", () => {
  const df = readFile(".docker/Dockerfile.linux-amd64");

  it("is based on the pinned Rust 1.88 slim image", () => {
    expect(df).toContain("FROM rust:1.88-slim");
  });

  it("installs webkit2gtk 4.1 dev package", () => {
    expect(df).toContain("libwebkit2gtk-4.1-dev");
  });

  it("installs Node.js 22 via NodeSource", () => {
    expect(df).toContain("nodesource.com/setup_22.x");
    expect(df).toContain("nodejs");
  });

  it("pre-adds the x86_64 Linux Rust target", () => {
    expect(df).toContain("rustup target add x86_64-unknown-linux-gnu");
  });

  it("cleans apt lists to keep image lean", () => {
    expect(df).toContain("rm -rf /var/lib/apt/lists/*");
  });
});

describe("Dockerfile.windows-cross", () => {
  const df = readFile(".docker/Dockerfile.windows-cross");

  it("is based on the pinned Rust 1.88 slim image", () => {
    expect(df).toContain("FROM rust:1.88-slim");
  });

  it("installs mingw-w64 cross-compiler", () => {
    expect(df).toContain("mingw-w64");
  });

  it("installs nsis for Windows installer bundling", () => {
    expect(df).toContain("nsis");
  });

  it("installs Node.js 22 via NodeSource", () => {
    expect(df).toContain("nodesource.com/setup_22.x");
  });

  it("pre-adds the Windows GNU Rust target", () => {
    expect(df).toContain("rustup target add x86_64-pc-windows-gnu");
  });

  it("cleans apt lists to keep image lean", () => {
    expect(df).toContain("rm -rf /var/lib/apt/lists/*");
  });
});

describe("Dockerfile.linux-arm64", () => {
  const df = readFile(".docker/Dockerfile.linux-arm64");

  it("is based on Ubuntu 22.04 (Jammy)", () => {
    expect(df).toContain("FROM ubuntu:22.04");
  });

  it("installs aarch64 cross-compiler", () => {
    expect(df).toContain("gcc-aarch64-linux-gnu");
    expect(df).toContain("g++-aarch64-linux-gnu");
  });

  it("sets up arm64 multiarch via ports.ubuntu.com", () => {
    expect(df).toContain("dpkg --add-architecture arm64");
    expect(df).toContain("ports.ubuntu.com/ubuntu-ports");
    expect(df).toContain("jammy");
  });

  it("installs arm64 webkit2gtk dev package", () => {
    expect(df).toContain("libwebkit2gtk-4.1-dev:arm64");
  });

  it("installs Rust 1.88 with arm64 cross-compilation target", () => {
    expect(df).toContain("--default-toolchain 1.88.0");
    expect(df).toContain("rustup target add aarch64-unknown-linux-gnu");
  });

  it("adds cargo to PATH via ENV", () => {
    expect(df).toContain('ENV PATH="/root/.cargo/bin:${PATH}"');
  });

  it("installs Node.js 22 via NodeSource", () => {
    expect(df).toContain("nodesource.com/setup_22.x");
  });
});

// ─── build-images.yml workflow ───────────────────────────────────────────────

describe("build-images.yml workflow", () => {
  const wf = readFile(".gitea/workflows/build-images.yml");

  it("triggers on changes to .docker/ files on master", () => {
    expect(wf).toContain("- master");
    expect(wf).toContain("- '.docker/**'");
  });

  it("supports manual workflow_dispatch trigger", () => {
    expect(wf).toContain("workflow_dispatch:");
  });

  it("does not explicitly mount the Docker socket (act_runner mounts it automatically)", () => {
    // act_runner already mounts /var/run/docker.sock; an explicit options: mount
    // causes a 'Duplicate mount point' error and must not be present.
    expect(wf).not.toContain("-v /var/run/docker.sock:/var/run/docker.sock");
  });

  it("authenticates to the local Gitea registry before pushing", () => {
    expect(wf).toContain("docker login");
    expect(wf).toContain("--password-stdin");
    expect(wf).toContain("172.0.0.29:3000");
  });

  it("builds and pushes all three platform images", () => {
    expect(wf).toContain("trcaa-linux-amd64:rust1.88-node22");
    expect(wf).toContain("trcaa-windows-cross:rust1.88-node22");
    expect(wf).toContain("trcaa-linux-arm64:rust1.88-node22");
  });

  it("uses alpine:latest with docker-cli (not docker:24-cli which triggers duplicate socket mount in act_runner)", () => {
    // act_runner v0.3.1 special-cases docker:* images and adds the socket bind;
    // combined with its global socket bind this causes a 'Duplicate mount point' error.
    expect(wf).toContain("alpine:latest");
    expect(wf).toContain("docker-cli");
    expect(wf).not.toContain("docker:24-cli");
  });

  it("runs all three build jobs on linux-amd64 runner", () => {
    const matches = wf.match(/runs-on: linux-amd64/g) ?? [];
    expect(matches.length).toBeGreaterThanOrEqual(3);
  });

  it("uses RELEASE_TOKEN secret for registry auth", () => {
    expect(wf).toContain("secrets.RELEASE_TOKEN");
  });
});
