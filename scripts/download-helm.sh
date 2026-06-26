#!/bin/bash
set -e

HELM_VERSION="v3.17.0"
BINARIES_DIR="src-tauri/binaries"

echo "Downloading helm binaries version ${HELM_VERSION}..."

mkdir -p "$BINARIES_DIR"

# Helm tarballs extract to {os}-{arch}/helm (or helm.exe on Windows)

echo "Downloading helm for Linux x86_64..."
TMPDIR=$(mktemp -d)
curl -L -o "$TMPDIR/helm-linux-amd64.tar.gz" \
  "https://get.helm.sh/helm-${HELM_VERSION}-linux-amd64.tar.gz"
tar -xzf "$TMPDIR/helm-linux-amd64.tar.gz" -C "$TMPDIR"
cp "$TMPDIR/linux-amd64/helm" "$BINARIES_DIR/helm-x86_64-unknown-linux-gnu"
rm -rf "$TMPDIR"

echo "Downloading helm for Linux aarch64..."
TMPDIR=$(mktemp -d)
curl -L -o "$TMPDIR/helm-linux-arm64.tar.gz" \
  "https://get.helm.sh/helm-${HELM_VERSION}-linux-arm64.tar.gz"
tar -xzf "$TMPDIR/helm-linux-arm64.tar.gz" -C "$TMPDIR"
cp "$TMPDIR/linux-arm64/helm" "$BINARIES_DIR/helm-aarch64-unknown-linux-gnu"
rm -rf "$TMPDIR"

echo "Downloading helm for macOS x86_64..."
TMPDIR=$(mktemp -d)
curl -L -o "$TMPDIR/helm-darwin-amd64.tar.gz" \
  "https://get.helm.sh/helm-${HELM_VERSION}-darwin-amd64.tar.gz"
tar -xzf "$TMPDIR/helm-darwin-amd64.tar.gz" -C "$TMPDIR"
cp "$TMPDIR/darwin-amd64/helm" "$BINARIES_DIR/helm-x86_64-apple-darwin"
rm -rf "$TMPDIR"

echo "Downloading helm for macOS aarch64..."
TMPDIR=$(mktemp -d)
curl -L -o "$TMPDIR/helm-darwin-arm64.tar.gz" \
  "https://get.helm.sh/helm-${HELM_VERSION}-darwin-arm64.tar.gz"
tar -xzf "$TMPDIR/helm-darwin-arm64.tar.gz" -C "$TMPDIR"
cp "$TMPDIR/darwin-arm64/helm" "$BINARIES_DIR/helm-aarch64-apple-darwin"
rm -rf "$TMPDIR"

echo "Downloading helm for Windows x86_64..."
TMPDIR=$(mktemp -d)
curl -L -o "$TMPDIR/helm-windows-amd64.zip" \
  "https://get.helm.sh/helm-${HELM_VERSION}-windows-amd64.zip"
unzip -q "$TMPDIR/helm-windows-amd64.zip" -d "$TMPDIR"
cp "$TMPDIR/windows-amd64/helm.exe" "$BINARIES_DIR/helm-x86_64-pc-windows-msvc.exe"
rm -rf "$TMPDIR"

# Make binaries executable
chmod +x "$BINARIES_DIR"/helm-*-linux-* "$BINARIES_DIR"/helm-*-darwin

echo "helm binaries downloaded successfully to $BINARIES_DIR"
echo "Total size:"
du -sh "$BINARIES_DIR"
