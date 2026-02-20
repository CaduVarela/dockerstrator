#!/bin/bash

# Build release binaries for all platforms
# Requires: cargo-cross (install with: cargo install cross)

set -e

BUILD_DIR="./target/releases"
mkdir -p "$BUILD_DIR"

echo "Building Dockerstrator for all platforms..."
echo ""

# Install cross if not available
if ! command -v cross &> /dev/null; then
    echo "Installing cross..."
    cargo install cross
fi

TARGETS=(
    "x86_64-unknown-linux-gnu:dockerstrator-linux-x64"
    "aarch64-unknown-linux-gnu:dockerstrator-linux-arm64"
    "x86_64-apple-darwin:dockerstrator-macos-x64"
    "aarch64-apple-darwin:dockerstrator-macos-arm64"
)

for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target binary_name <<< "$target_info"

    echo "Building for $target..."

    # For macOS targets, use cargo instead of cross
    if [[ $target == *"darwin"* ]]; then
        cargo build --release --target "$target" 2>/dev/null || echo "Skipped (requires macOS)"
    else
        cross build --release --target "$target"
    fi

    # Copy binary if build succeeded
    if [ -f "target/$target/release/dockerstrator" ]; then
        cp "target/$target/release/dockerstrator" "$BUILD_DIR/$binary_name"
        echo "  -> $BUILD_DIR/$binary_name"
    fi
done

echo ""
echo "Build complete! Binaries available in: $BUILD_DIR/"
echo ""
ls -lh "$BUILD_DIR/"
