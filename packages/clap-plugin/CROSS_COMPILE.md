# Cross-Compiling Strudel CLAP Plugin for Windows

This guide explains how to build Windows binaries of the Strudel CLAP plugin from Linux.

## Quick Start

```bash
# Add Windows target to Rust
rustup target add x86_64-pc-windows-gnu

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Output: target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll
```

## Setup

### 1. Install Cross-Compilation Tools

Add to your `flake.nix` (or install system-wide):

```nix
buildInputs = with pkgs; [
  # ... existing packages ...

  # Windows cross-compilation
  pkgsCross.mingwW64.stdenv.cc
  wineWowPackages.stable  # Optional: for testing
];
```

Or manually:
```bash
# Debian/Ubuntu
sudo apt install mingw-w64

# Arch Linux
sudo pacman -S mingw-w64-gcc

# Fedora
sudo dnf install mingw64-gcc
```

### 2. Add Rust Windows Target

```bash
rustup target add x86_64-pc-windows-gnu
```

### 3. Configure Cargo for Cross-Compilation

Create or edit `~/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

## Building

### Development Build (with debug symbols)

```bash
cargo build --target x86_64-pc-windows-gnu
```

Output: `target/x86_64-pc-windows-gnu/debug/strudel_clap_plugin.dll`

### Release Build (optimized)

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

Output: `target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll`

### Bundle as CLAP Plugin

```bash
# Bundle for Windows (requires cargo-xtask)
cargo xtask bundle strudel-clap-plugin --release --target x86_64-pc-windows-gnu
```

Output: Windows CLAP bundle ready for installation

## Windows CLAP Plugin Format

On Windows, CLAP plugins are:
- **File extension:** `.clap`
- **Actually:** A directory/folder containing:
  - `Contents/x86_64-win/strudel_clap_plugin.dll` (64-bit)
  - Metadata files

The `cargo xtask bundle` command creates this structure automatically.

## Installation on Windows

Copy the `.clap` bundle to:
```
C:\Program Files\Common Files\CLAP\
```

Or user-specific location:
```
%LOCALAPPDATA%\Programs\Common\CLAP\
```

## Testing Without Windows

You can test Windows builds on Linux using Wine:

```bash
# Install Wine (if using Nix, add wineWowPackages.stable)
wine target/x86_64-pc-windows-gnu/release/strudel_clap_plugin.dll
```

Note: This only tests if the DLL loads, not full plugin functionality.

## Troubleshooting

### "linker not found" error

Make sure mingw-w64 is installed and the linker is in PATH:
```bash
which x86_64-w64-mingw32-gcc
```

### "undefined reference" errors

Some dependencies may not cross-compile easily. Check if all dependencies support Windows.

### Plugin doesn't load in Windows DAW

- Verify the .clap bundle structure is correct
- Check that all dependencies are included
- Ensure MSVC runtime is available (or use static linking)

## GNU vs MSVC Target

Two Windows targets are available:

1. **x86_64-pc-windows-gnu** (recommended for cross-compilation)
   - Uses MinGW toolchain
   - Easier to cross-compile from Linux
   - No MSVC runtime required

2. **x86_64-pc-windows-msvc** (native Windows builds)
   - Uses Microsoft Visual C++ toolchain
   - Requires Windows or complex setup
   - Better compatibility with some Windows DAWs

For cross-compilation from Linux, use `-gnu` target.

## Multi-Platform Build Script

Create `build-all.sh`:

```bash
#!/bin/bash
set -e

echo "Building for Linux..."
cargo build --release

echo "Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu

echo "Building for macOS..."
# cargo build --release --target x86_64-apple-darwin
# cargo build --release --target aarch64-apple-darwin

echo "All builds complete!"
ls -lh target/release/
ls -lh target/x86_64-pc-windows-gnu/release/
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Build Multi-Platform

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install cross-compilation tools
        run: |
          sudo apt-get update
          sudo apt-get install -y mingw-w64
          rustup target add x86_64-pc-windows-gnu

      - name: Build Linux
        run: cargo build --release

      - name: Build Windows
        run: cargo build --release --target x86_64-pc-windows-gnu

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: plugins
          path: |
            target/release/*.so
            target/x86_64-pc-windows-gnu/release/*.dll
```

## Known Limitations

1. **GUI testing**: Can't fully test the plugin GUI without Windows
2. **DAW compatibility**: Some DAWs may behave differently on Windows
3. **Audio drivers**: Different audio backend on Windows (WASAPI vs ALSA/JACK)
4. **Performance**: Cross-compiled binaries should perform identically, but driver behavior varies

## macOS Cross-Compilation

macOS is more complex to cross-compile to from Linux. Consider:
- Using GitHub Actions with macOS runners
- Using Zig cc as a cross-compiler (experimental)
- Building natively on macOS hardware

See [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) for experimental macOS cross-compilation.
