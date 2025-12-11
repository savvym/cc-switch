#!/bin/bash
set -e

echo "Building CLI binary for bundling..."

# Build CLI for current platform
cargo build --release -p cc-switch-cli --bin cc-switch

# Copy to resources directory
mkdir -p src-tauri/resources
cp target/release/cc-switch src-tauri/resources/

echo "✓ CLI binary bundled to src-tauri/resources/cc-switch"
