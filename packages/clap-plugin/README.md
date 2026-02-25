# @strudel/clap-plugin

Strudel live coding pattern language as a CLAP/VST3 audio plugin for DAWs.

## Status

🚧 **In Development** - See [docs/clap-plugin-plan.md](../../docs/clap-plugin-plan.md) for the implementation plan.

## Features (Planned)

- 🎵 **Audio Synthesis** - Built-in synthesizer for instant sound
- 🎹 **MIDI Output** - Send notes to your DAW's instruments
- 🔄 **DAW Sync** - Tempo and transport synchronization
- 💾 **File Storage** - Save/load patterns as `.strudel` files
- ✨ **Live Coding** - Edit patterns in real-time while music plays

## Building

### Prerequisites

- Rust (latest stable)
- Cargo

### Development Build

```bash
cargo build
# Output: target/debug/libstrudel_clap_plugin.so (Linux)
```

### Release Build

```bash
cargo build --release
# Output: target/release/libstrudel_clap_plugin.so (Linux)
```

### Bundle for Installation

```bash
# Bundle the plugin (creates both CLAP and VST3)
cargo xtask bundle strudel-clap-plugin --release

# Outputs:
# - target/bundled/strudel-clap-plugin.clap (CLAP format)
# - target/bundled/strudel-clap-plugin.vst3/ (VST3 bundle)

# Install CLAP:
# Linux: cp target/bundled/strudel-clap-plugin.clap ~/.clap/
# macOS: cp target/bundled/strudel-clap-plugin.clap ~/Library/Audio/Plug-Ins/CLAP/
# Windows: copy target\bundled\strudel-clap-plugin.clap "%COMMONPROGRAMFILES%\CLAP\"

# Install VST3:
# Linux: cp -r target/bundled/strudel-clap-plugin.vst3 ~/.vst3/
# macOS: cp -r target/bundled/strudel-clap-plugin.vst3 ~/Library/Audio/Plug-Ins/VST3/
# Windows: xcopy target\bundled\strudel-clap-plugin.vst3 "%COMMONPROGRAMFILES%\VST3\" /E /I
```

## Testing in a DAW

The plugin can be tested in any CLAP or VST3-compatible DAW:

**CLAP Support:**
- **REAPER** - Excellent CLAP support
- **Bitwig Studio** - Native CLAP support
- **Ardour** - CLAP support in recent versions
- **Carla** - Lightweight host, great for testing

**VST3 Support:**
- **Ableton Live** - VST3 support
- **FL Studio** - VST3 support
- **Logic Pro** - VST3 support (macOS)
- **Cubase/Nuendo** - VST3 native format
- **Studio One** - VST3 support
- Most other modern DAWs

## Architecture

See [docs/clap-plugin-plan.md](../../docs/clap-plugin-plan.md) for detailed architecture documentation.

## Development Roadmap

- [x] Phase 0: Research & Planning
- [ ] Phase 1: Project Setup (In Progress)
- [ ] Phase 2: JavaScript Runtime Integration
- [ ] Phase 3: nih-plug Plugin Implementation
- [ ] Phase 4: Minimal GUI
- [ ] Phase 5: File Management
- [ ] Phase 6: Testing & Polish

## References

- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Rust plugin framework
- [pluguzu](https://codeberg.org/TristanCacqueray/pluguzu) - Similar project for TidalCycles
- [CLAP Specification](https://github.com/free-audio/clap)

## License

AGPL-3.0-or-later - Same as the main Strudel project.
