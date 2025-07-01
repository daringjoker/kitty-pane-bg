#!/bin/bash

# Build script for kitty-pane-bg
# Builds optimized release binary with all optimizations enabled

set -e

echo "🔧 Building kitty-pane-bg..."

# Clean previous build
echo "🧹 Cleaning previous build artifacts..."
cargo clean

# Build with maximum optimizations
echo "🚀 Building optimized release binary..."
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Show build information
echo "✅ Build completed successfully!"
echo ""
echo "📁 Binary location: ./target/release/kitty-pane-bg"
echo "📊 Binary size: $(du -h target/release/kitty-pane-bg | cut -f1)"
echo ""
echo "🎯 To install to PATH:"
echo "   cp target/release/kitty-pane-bg ~/.local/bin/"
echo "   # or"
echo "   cargo install --path ."
echo ""
echo "🏃 To test:"
echo "   ./target/release/kitty-pane-bg check"
