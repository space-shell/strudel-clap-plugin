#!/usr/bin/env bash
# Bundle Strudel runtime for CLAP plugin
# Copyright (C) 2026 Strudel Contributors
# Licensed under AGPL-3.0-or-later

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(dirname "$SCRIPT_DIR")"
STRUDEL_ROOT="$(dirname "$(dirname "$PLUGIN_DIR")")"

echo "📦 Bundling Strudel runtime for plugin..."
echo "Plugin dir: $PLUGIN_DIR"
echo "Strudel root: $STRUDEL_ROOT"

# Create output directory
mkdir -p "$PLUGIN_DIR/bundled"

# Bundle the runtime with esbuild
echo "🔨 Building JavaScript bundle..."
cd "$PLUGIN_DIR"

# Bundle everything into a single file
# Use IIFE format to avoid ES module issues with import.meta
pnpm exec esbuild "$SCRIPT_DIR/runtime-entry.mjs" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --outfile="$PLUGIN_DIR/bundled/strudel-runtime.js" \
  --external:node:* \
  --main-fields=main,module \
  --define:import.meta.url="'file://strudel-runtime.js'" \
  --log-level=warning

# Check if bundle was created successfully
if [ -f "$PLUGIN_DIR/bundled/strudel-runtime.js" ]; then
    BUNDLE_SIZE=$(du -h "$PLUGIN_DIR/bundled/strudel-runtime.js" | cut -f1)
    echo "✅ Bundle created successfully: $BUNDLE_SIZE"
    echo "   Location: $PLUGIN_DIR/bundled/strudel-runtime.js"
else
    echo "❌ Bundle creation failed"
    exit 1
fi

echo "✨ Done!"
