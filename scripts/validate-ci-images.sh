#!/bin/sh
# validate-ci-images.sh — Asserts that Docker image tag references are consistent
# across all CI workflow files and Dockerfiles. Run locally or in CI to catch
# version drift when bumping the Rust toolchain.
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
AMD64_VER=$(grep '^FROM rust:' "$REPO_ROOT/.docker/Dockerfile.linux-amd64" |
    sed 's/FROM rust:\([0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$AMD64_VER" ] && fail "Could not extract Rust version from Dockerfile.linux-amd64 (expected 'FROM rust:X.Y-slim')"

WIN_VER=$(grep '^FROM rust:' "$REPO_ROOT/.docker/Dockerfile.windows-cross" |
    sed 's/FROM rust:\([0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$WIN_VER" ] && fail "Could not extract Rust version from Dockerfile.windows-cross (expected 'FROM rust:X.Y-slim')"

# arm64 uses rustup: --default-toolchain X.Y.Z  (strip patch → X.Y)
ARM64_FULL=$(grep 'default-toolchain' "$REPO_ROOT/.docker/Dockerfile.linux-arm64" |
    sed 's/.*--default-toolchain \([0-9]*\.[0-9]*\.[0-9]*\).*/\1/' | head -1)
[ -z "$ARM64_FULL" ] && fail "Could not extract Rust version from Dockerfile.linux-arm64 (expected '--default-toolchain X.Y.Z')"
ARM64_VER=$(echo "$ARM64_FULL" | sed 's/\.[0-9]*$//')

info "Dockerfile.linux-amd64   Rust: $AMD64_VER"
info "Dockerfile.windows-cross Rust: $WIN_VER"
info "Dockerfile.linux-arm64   Rust: $ARM64_VER (full: $ARM64_FULL)"

[ "$AMD64_VER" = "$WIN_VER" ] ||
    fail "amd64 ($AMD64_VER) and windows ($WIN_VER) Dockerfiles disagree on Rust version"
[ "$AMD64_VER" = "$ARM64_VER" ] ||
    fail "amd64 ($AMD64_VER) and arm64 ($ARM64_VER) Dockerfiles disagree on Rust version"

# Validate version format (must be X.Y, e.g. 1.89)
echo "$AMD64_VER" | grep -qE '^[0-9]+\.[0-9]+$' ||
    fail "Extracted Rust version '$AMD64_VER' does not match expected X.Y format"

EXPECTED_TAG="rust${AMD64_VER}-node22"
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
