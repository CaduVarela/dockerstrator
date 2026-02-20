#!/bin/bash

# Build script for Dockerstrator
# Compiles the Rust project in release mode

set -e

echo "Building Dockerstrator..."
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed"
    echo "Install from: https://rustup.rs/"
    exit 1
fi

# Build
cargo build --release

echo ""
echo "Build complete!"
echo ""
echo "Executable location: ./target/release/dockerstrator"
echo ""
echo "To install globally:"
echo "  cp target/release/dockerstrator ~/.local/bin/"
echo ""
echo "To run:"
echo "  ./target/release/dockerstrator"
