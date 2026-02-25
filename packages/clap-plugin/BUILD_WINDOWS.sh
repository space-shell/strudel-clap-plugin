#!/bin/bash
# Quick script to build Strudel CLAP plugin for Windows

set -e

echo "🪟 Building Strudel CLAP Plugin for Windows..."
echo ""

# Check if Windows target is installed
if ! rustup target list | grep -q "x86_64-pc-windows-gnu (installed)"; then
    echo "📦 Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Build for Windows
echo "🔨 Building..."
cargo build --release --target x86_64-pc-windows-gnu

echo ""
echo "✅ Build complete!"
echo ""
echo "Output: target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll"
echo "Size: $(du -h target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll | cut -f1)"
echo ""
echo "To install on Windows:"
echo "  1. Rename to: Strudel.clap"
echo "  2. Copy to: C:\\Program Files\\Common Files\\CLAP\\"
echo ""
