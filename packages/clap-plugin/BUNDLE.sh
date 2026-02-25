#!/bin/bash
# Bundle Strudel CLAP/VST3 Plugin
# This script builds and bundles the plugin for distribution

set -e

echo "🎵 Building Strudel CLAP/VST3 Plugin..."
echo ""

# Build and bundle
cargo xtask bundle strudel-clap-plugin --release

echo ""
echo "✅ Plugin bundled successfully!"
echo ""
echo "Output:"
echo "  CLAP: target/bundled/strudel-clap-plugin.clap"
echo "  VST3: target/bundled/strudel-clap-plugin.vst3/"
echo ""
echo "To install:"
echo "  CLAP: cp target/bundled/strudel-clap-plugin.clap ~/.clap/"
echo "  VST3: cp -r target/bundled/strudel-clap-plugin.vst3 ~/.vst3/"
echo ""
