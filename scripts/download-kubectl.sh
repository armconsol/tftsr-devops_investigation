#!/bin/bash
set -e

KUBECTL_VERSION="v1.30.0"
BINARIES_DIR="src-tauri/binaries"

echo "Downloading kubectl binaries version ${KUBECTL_VERSION}..."

mkdir -p "$BINARIES_DIR"

# Download for all platforms
# Tauri uses this structure: binaries/kubectl-{target-triple} or kubectl-{target-triple}.exe
echo "Downloading kubectl for Linux x86_64..."
curl -L -o "$BINARIES_DIR/kubectl-x86_64-unknown-linux-gnu" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/amd64/kubectl"

echo "Downloading kubectl for Linux aarch64..."
curl -L -o "$BINARIES_DIR/kubectl-aarch64-unknown-linux-gnu" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/linux/arm64/kubectl"

echo "Downloading kubectl for macOS x86_64..."
curl -L -o "$BINARIES_DIR/kubectl-x86_64-apple-darwin" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/amd64/kubectl"

echo "Downloading kubectl for macOS aarch64..."
curl -L -o "$BINARIES_DIR/kubectl-aarch64-apple-darwin" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/darwin/arm64/kubectl"

echo "Downloading kubectl for Windows x86_64..."
curl -L -o "$BINARIES_DIR/kubectl-x86_64-pc-windows-gnu.exe" \
  "https://dl.k8s.io/release/$KUBECTL_VERSION/bin/windows/amd64/kubectl.exe"

# Make binaries executable (not needed for Windows .exe)
chmod +x "$BINARIES_DIR"/kubectl-*-linux-* "$BINARIES_DIR"/kubectl-*-darwin

echo "kubectl binaries downloaded successfully to $BINARIES_DIR"
echo "Total size:"
du -sh "$BINARIES_DIR"
