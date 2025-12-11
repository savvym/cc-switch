#!/bin/bash
set -e

echo "Building cc-switch CLI..."

# Build for current platform
cargo build --release -p cc-switch-cli --bin cc-switch

# Copy binary to project root
cp target/release/cc-switch ./cc-switch

echo "✓ Binary built: ./cc-switch"
echo ""
echo "Install with: sudo cp cc-switch /usr/local/bin/"
