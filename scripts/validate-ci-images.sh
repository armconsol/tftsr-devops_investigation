#!/bin/sh
# validate-ci-images.sh — Asserts that Docker image tag references are consistent
# across all CI workflow files and Dockerfiles. Run locally or in CI to catch
# version drift when bumping the Rust toolchain or Node.js version.
#
# Exit 0 on success, 1 on first failure (with descriptive error).
set -eu

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
info() { printf '  %s\n' "$*"; }

echo "=== Validating CI image tag consistency ==="

# --- Extract Rust version from each Dockerfile ---

# amd64 and Windows use: FROM rust:X.Y-slim
AMD64_RUST=$(grep '^FROM rust:' "$REPO_ROOT/.docker/Dockerfile.linux-amd64" |
    sed 's/FROM rust:\([0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$AMD64_RUST" ] && fail "Could not extract Rust version from Dockerfile.linux-amd64 (expected 'FROM rust:X.Y-slim')"

WIN_RUST=$(grep '^FROM rust:' "$REPO_ROOT/.docker/Dockerfile.windows-cross" |
    sed 's/FROM rust:\([0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$WIN_RUST" ] && fail "Could not extract Rust version from Dockerfile.windows-cross (expected 'FROM rust:X.Y-slim')"

# arm64 uses rustup: --default-toolchain X.Y.Z  (strip patch → X.Y)
ARM64_RUST_FULL=$(grep 'default-toolchain' "$REPO_ROOT/.docker/Dockerfile.linux-arm64" |
    sed 's/.*--default-toolchain \([0-9]*\.[0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$ARM64_RUST_FULL" ] && fail "Could not extract Rust version from Dockerfile.linux-arm64 (expected '--default-toolchain X.Y.Z')"
ARM64_RUST=$(echo "$ARM64_RUST_FULL" | sed 's/\.[0-9]*$//')

info "Dockerfile.linux-amd64   Rust: $AMD64_RUST"
info "Dockerfile.windows-cross Rust: $WIN_RUST"
info "Dockerfile.linux-arm64   Rust: $ARM64_RUST (full: $ARM64_RUST_FULL)"

[ "$AMD64_RUST" = "$WIN_RUST" ] ||
    fail "amd64 ($AMD64_RUST) and windows ($WIN_RUST) Dockerfiles disagree on Rust version"
[ "$AMD64_RUST" = "$ARM64_RUST" ] ||
    fail "amd64 ($AMD64_RUST) and arm64 ($ARM64_RUST) Dockerfiles disagree on Rust version"

echo "$AMD64_RUST" | grep -qE '^[0-9]+\.[0-9]+$' ||
    fail "Extracted Rust version '$AMD64_RUST' does not match expected X.Y format"

# --- Extract Node.js major version from each Dockerfile ---
# All three install Node via: curl .../setup_XX.x | bash -

AMD64_NODE=$(grep -oE 'setup_[0-9]+\.x' "$REPO_ROOT/.docker/Dockerfile.linux-amd64" |
    grep -oE '[0-9]+' | head -1)
[ -z "$AMD64_NODE" ] && fail "Could not extract Node.js version from Dockerfile.linux-amd64 (expected 'setup_XX.x')"

WIN_NODE=$(grep -oE 'setup_[0-9]+\.x' "$REPO_ROOT/.docker/Dockerfile.windows-cross" |
    grep -oE '[0-9]+' | head -1)
[ -z "$WIN_NODE" ] && fail "Could not extract Node.js version from Dockerfile.windows-cross (expected 'setup_XX.x')"

ARM64_NODE=$(grep -oE 'setup_[0-9]+\.x' "$REPO_ROOT/.docker/Dockerfile.linux-arm64" |
    grep -oE '[0-9]+' | head -1)
[ -z "$ARM64_NODE" ] && fail "Could not extract Node.js version from Dockerfile.linux-arm64 (expected 'setup_XX.x')"

info "Dockerfile.linux-amd64   Node: $AMD64_NODE"
info "Dockerfile.windows-cross Node: $WIN_NODE"
info "Dockerfile.linux-arm64   Node: $ARM64_NODE"

[ "$AMD64_NODE" = "$WIN_NODE" ] ||
    fail "amd64 (node$AMD64_NODE) and windows (node$WIN_NODE) Dockerfiles disagree on Node.js version"
[ "$AMD64_NODE" = "$ARM64_NODE" ] ||
    fail "amd64 (node$AMD64_NODE) and arm64 (node$ARM64_NODE) Dockerfiles disagree on Node.js version"

EXPECTED_TAG="rust${AMD64_RUST}-node${AMD64_NODE}"
info "Expected image tag: $EXPECTED_TAG"
echo ""

# --- Assert all workflow files reference the expected tag ---

grep -q "$EXPECTED_TAG" "$REPO_ROOT/.gitea/workflows/build-images.yml" ||
    fail "build-images.yml does not reference expected tag '$EXPECTED_TAG' — update docker build -t lines"
info "build-images.yml  ✓ references $EXPECTED_TAG"

grep -q "$EXPECTED_TAG" "$REPO_ROOT/.gitea/workflows/release-beta.yml" ||
    fail "release-beta.yml does not reference expected tag '$EXPECTED_TAG' — update container image: lines"
info "release-beta.yml  ✓ references $EXPECTED_TAG"

grep -q "$EXPECTED_TAG" "$REPO_ROOT/.gitea/workflows/auto-tag.yml" ||
    fail "auto-tag.yml does not reference expected tag '$EXPECTED_TAG' — update container image: lines"
info "auto-tag.yml      ✓ references $EXPECTED_TAG"

echo ""
echo "=== All checks passed ==="
