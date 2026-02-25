#!/bin/bash
# Build Strudel CLAP plugin for all platforms

set -e

echo "🌍 Building Strudel CLAP Plugin for all platforms..."
echo ""

# Linux
echo "🐧 Building for Linux..."
cargo build --release
echo "✅ Linux: target/release/libstrudel_clap_plugin.so"
echo ""

# Windows
echo "🪟 Building for Windows..."
if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    rustup target add x86_64-pc-windows-gnu
fi
cargo build --release --target x86_64-pc-windows-gnu
echo "✅ Windows: target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll"
echo ""

# macOS would go here (requires macOS or complex setup)
# echo "🍎 Building for macOS..."
# cargo build --release --target x86_64-apple-darwin
# cargo build --release --target aarch64-apple-darwin

echo "🎉 All builds complete!"
echo ""
echo "Summary:"
ls -lh target/release/libstrudel_clap_plugin.so 2>/dev/null || true
ls -lh target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll 2>/dev/null || true
