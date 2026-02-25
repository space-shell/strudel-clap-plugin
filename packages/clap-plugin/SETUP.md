# Strudel CLAP Plugin - Setup Guide

## Prerequisites

### Rust Toolchain

The plugin is written in Rust and requires a working Rust installation.

#### Option 1: Fresh Rustup Installation (Recommended)

```bash
# Install rustup (Rust toolchain manager)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source ~/.cargo/env

# Verify installation
cargo --version
rustc --version
```

#### Option 2: Using Nix (if you're on NixOS or use Nix)

```bash
# Enter a development shell with Rust
nix-shell -p cargo rustc

# Or create a shell.nix in this directory (see below)
```

Create `shell.nix`:
```nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    pkg-config
    alsa-lib
    libjack2
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
```

Then run:
```bash
nix-shell
cargo build
```

### System Dependencies (Linux)

For CLAP plugin development, you may need:

```bash
# Debian/Ubuntu
sudo apt install build-essential pkg-config libasound2-dev libjack-jackd2-dev

# Fedora
sudo dnf install gcc pkg-config alsa-lib-devel jack-audio-connection-kit-devel

# Arch Linux
sudo pacman -S base-devel alsa-lib jack2
```

### DAW for Testing

Install a CLAP-compatible host:
- **Carla** (lightweight, great for testing): `sudo apt install carla`
- **REAPER** (excellent CLAP support): https://www.reaper.fm/download.php
- **Bitwig Studio** (native CLAP support): https://www.bitwig.com/
- **Ardour** (open source): https://ardour.org/download.html

## Building the Plugin

### Development Build

```bash
cd packages/clap-plugin
cargo build
```

The plugin will be at: `target/debug/libstrudel_clap_plugin.so` (Linux)

### Release Build (Optimized)

```bash
cargo build --release
```

The plugin will be at: `target/release/libstrudel_clap_plugin.so` (Linux)

### Installing for Testing

To install the plugin where your DAW can find it:

```bash
# Install cargo-xtask for bundling
cargo install cargo-xtask

# Bundle and install the plugin
cargo xtask bundle strudel-clap-plugin --release

# Plugin installed to:
# Linux: ~/.clap/Strudel.clap
# macOS: ~/Library/Audio/Plug-Ins/CLAP/Strudel.clap
# Windows: %COMMONPROGRAMFILES%\CLAP\Strudel.clap
```

Note: `cargo xtask` is provided by nih-plug for packaging plugins properly.

## Testing in a DAW

### Using Carla (Easiest)

```bash
# Launch Carla
carla

# In Carla:
# 1. Click "Add Plugin"
# 2. Search for "Strudel"
# 3. Double-click to load
```

### Using REAPER

```bash
# Launch REAPER
reaper

# In REAPER:
# 1. Open Preferences (Ctrl+P)
# 2. Go to Plug-ins → CLAP
# 3. Add ~/.clap to plugin paths
# 4. Rescan plugins
# 5. Insert Strudel on a track (Right-click track → Insert virtual instrument → CLAP → Strudel)
```

## Current Status

As of the initial setup, the plugin:
- ✅ Compiles successfully (once Rust is set up)
- ✅ Exports CLAP and VST3 interfaces
- ✅ Has a master gain parameter (for testing)
- ✅ Can be loaded in a DAW
- ❌ Does not yet evaluate Strudel patterns (Phase 2)
- ❌ Does not yet output audio or MIDI (Phase 3)
- ❌ Does not yet have a GUI (Phase 4)

## Development Workflow

```bash
# 1. Make changes to Rust code
vim src/plugin.rs

# 2. Build
cargo build

# 3. Test in DAW
carla  # or your preferred host

# 4. Check for issues
cargo clippy  # Lint
cargo test    # Run tests (when we add them)
```

## Troubleshooting

### "cargo: command not found"

Rust is not in your PATH. Try:
```bash
source ~/.cargo/env
```

Or reinstall Rust with rustup (see above).

### "cannot find -lasound" or similar linker errors

Missing system dependencies. Install ALSA/JACK dev libraries (see System Dependencies above).

### Plugin doesn't appear in DAW

1. Check plugin was installed: `ls -la ~/.clap/`
2. Rescan plugins in your DAW
3. Check DAW plugin paths include `~/.clap`
4. Try loading in Carla first (simpler environment)

### nih-plug git dependency errors

If you see errors fetching nih-plug from git, you may need to:
```bash
# Update git to latest
sudo apt update && sudo apt install git

# Or use a specific nih-plug release (edit Cargo.toml)
# nih_plug = "0.0.7"  # Instead of git repo
```

## Next Steps

See [docs/clap-plugin-plan.md](../../docs/clap-plugin-plan.md) for the full implementation roadmap.

Current phase: **Phase 1 - Project Setup** (almost complete!)

Next phase: **Phase 2 - JavaScript Runtime Integration** (add Deno Core to evaluate Strudel patterns)
