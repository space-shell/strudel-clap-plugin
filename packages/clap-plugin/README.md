# @strudel/clap-plugin

Strudel live coding pattern language as a CLAP audio plugin for DAWs.

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
# Install cargo-xtask if not already installed
cargo install cargo-xtask

# Bundle the plugin
cargo xtask bundle strudel-clap-plugin --release

# Plugin installed to:
# Linux: ~/.clap/Strudel.clap
# macOS: ~/Library/Audio/Plug-Ins/CLAP/Strudel.clap
# Windows: %COMMONPROGRAMFILES%\CLAP\Strudel.clap
```

## Testing in a DAW

The plugin can be tested in any CLAP-compatible DAW:

- **REAPER** - Excellent CLAP support
- **Bitwig Studio** - Native CLAP support
- **Ardour** - CLAP support in recent versions
- **Carla** - Lightweight host, great for testing

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
