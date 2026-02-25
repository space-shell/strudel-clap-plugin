# Strudel CLAP Plugin - Implementation Plan

**Status:** 🚧 In Development
**Started:** 2026-02-25
**Last Updated:** 2026-02-25
**License:** AGPL-3.0-or-later

---

## Overview

This document tracks the development of a CLAP audio plugin that integrates Strudel's pattern engine into DAWs. The plugin will support both audio synthesis and MIDI output, with DAW transport synchronization.

### Design Goals

- ✅ **Speed over quality** - Start with simple built-in synth, iterate later
- ✅ **Minimal GUI** - Code editor first, fancy features later
- ✅ **Separate file storage** - Patterns stored as `.strudel` files
- ✅ **AGPL-3.0** - Keep consistent with Strudel's licensing

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                  DAW (Host)                         │
│  ┌───────────────────────────────────────────────┐ │
│  │         Strudel CLAP Plugin                   │ │
│  │  ┌─────────────────────────────────────────┐  │ │
│  │  │  GUI Layer (Tauri WebView)             │  │ │
│  │  │  - CodeMirror editor                    │  │ │
│  │  │  - Audio/MIDI mode toggle               │  │ │
│  │  └─────────────────────────────────────────┘  │ │
│  │              ↕ IPC Messages                    │ │
│  │  ┌─────────────────────────────────────────┐  │ │
│  │  │  Rust Plugin Core (nih-plug)           │  │ │
│  │  │  - Transport sync (tempo, play/stop)    │  │ │
│  │  │  - Audio/MIDI output                    │  │ │
│  │  │  - Event scheduling                     │  │ │
│  │  │  - Simple built-in synthesizer          │  │ │
│  │  └─────────────────────────────────────────┘  │ │
│  │              ↕ JS/Rust Bridge                  │ │
│  │  ┌─────────────────────────────────────────┐  │ │
│  │  │  JavaScript Runtime (Deno Core/V8)     │  │ │
│  │  │  - Strudel pattern engine               │  │ │
│  │  │  - Pattern evaluation                   │  │ │
│  │  │  - Event generation                     │  │ │
│  │  └─────────────────────────────────────────┘  │ │
│  └───────────────────────────────────────────────┘ │
│              ↕ Audio/MIDI/Transport                │
└─────────────────────────────────────────────────────┘
```

### Key Architectural Decisions

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Plugin Framework** | nih-plug | Best Rust CLAP/VST3 support, proven by pluguzu |
| **JavaScript Runtime** | Deno Core (V8) | Good Rust bindings, ES modules, production-ready |
| **GUI** | Tauri WebView (nih_plug_webview) | Reuse Strudel web UI components |
| **Audio Synthesis** | Simple built-in synth | Faster development, iterate on quality later |
| **Event Model** | Fixed-cycle pre-rendering | Better seek performance (proven by pluguzu) |
| **MIDI Output** | nih-plug MIDI API | Standard, works across hosts |
| **Transport Sync** | Tempo + Play/Stop | Core features for MVP |
| **Pattern Storage** | Separate `.strudel` files | Shareable, version-controllable |

---

## Implementation Phases

### ✅ Phase 0: Research & Planning
**Status:** Complete
**Duration:** 1 day

- [x] Research pluguzu architecture
- [x] Analyze existing Tauri setup
- [x] Create implementation plan
- [x] Get user approval

### 🚧 Phase 1: Project Setup & Proof of Concept
**Status:** In Progress
**Duration:** 1-2 weeks
**Started:** 2026-02-25

#### Tasks
- [x] Create `packages/clap-plugin/` directory structure
- [x] Set up Cargo.toml with nih-plug dependencies
- [x] Set up package.json for JS bundling
- [x] Create minimal Rust plugin stub (hello world)
- [x] Create README, LICENSE, and SETUP guide
- [ ] **Next: Verify Rust toolchain and build plugin**
- [ ] Test loading in CLAP host (Carla/REAPER)
- [ ] Choose and integrate JavaScript runtime (Deno Core recommended)
- [ ] Bundle Strudel for embedding (~1MB JS bundle)
- [ ] Verify JS execution from Rust

**Deliverable:** Plugin loads in DAW, can execute simple JS code

#### Progress Notes

**2026-02-25:**
- ✅ Created package structure with all boilerplate files
- ✅ Implemented minimal nih-plug plugin with:
  - CLAP and VST3 exports
  - Stereo audio output configuration
  - MIDI output enabled
  - Master gain parameter for testing
  - Stub process() function
- ⚠️ **Action Required:** Need to verify Rust installation before continuing
  - Detected Nix-based Rust installation that may need nix-shell
  - See SETUP.md for installation options

#### Directory Structure
```
packages/clap-plugin/
├── Cargo.toml              # Rust dependencies
├── build.rs                # Build script
├── package.json            # JS dependencies & scripts
├── README.md
├── LICENSE                 # AGPL-3.0
├── src/
│   ├── lib.rs             # nih-plug entry point
│   ├── plugin.rs          # Main plugin implementation
│   ├── audio_engine.rs    # Simple synthesizer
│   ├── midi_handler.rs    # MIDI output
│   ├── js_runtime.rs      # JavaScript runtime bridge
│   ├── event_buffer.rs    # Double-buffered events
│   ├── transport.rs       # DAW transport sync
│   └── ui/
│       ├── mod.rs         # GUI module
│       └── webview.rs     # Tauri webview integration
├── webui/                 # Minimal GUI source
│   ├── index.html
│   ├── editor.js          # CodeMirror setup
│   └── styles.css
└── bundled/               # Compiled Strudel bundle
    └── strudel.js         # Generated by build
```

### ⏳ Phase 2: JavaScript Runtime Integration
**Status:** Not Started
**Duration:** 2-3 weeks

#### Tasks
- [ ] Initialize Deno runtime with Strudel bundle
- [ ] Implement Rust→JS bridge functions:
  - `set_code(pattern_code)` - Update pattern
  - `get_events(start_time, end_time)` - Query events
  - `set_tempo(bpm)` - Sync tempo
  - `get_errors()` - Return compilation errors
- [ ] Implement event structure (onset, duration, note, velocity, params)
- [ ] Implement fixed-cycle rendering (4-8 bars pre-rendered)
- [ ] Implement double buffering for event updates
- [ ] Create custom Strudel audio backend (outputs events, not audio)
- [ ] Test pattern evaluation and event generation

**Deliverable:** Pattern code → Event stream working

### ⏳ Phase 3: nih-plug Plugin Implementation
**Status:** Not Started
**Duration:** 2-3 weeks

#### Tasks
- [ ] Implement `Plugin` trait for `StrudelPlugin`
- [ ] Configure audio I/O (stereo output)
- [ ] Configure MIDI output
- [ ] Implement transport synchronization:
  - [ ] Tempo (BPM) sync
  - [ ] Play/Stop sync
- [ ] Implement `process()` audio callback:
  - [ ] Read transport state
  - [ ] Query events for current buffer
  - [ ] Send MIDI events via context
  - [ ] Synthesize audio for audio events
- [ ] Implement simple built-in synthesizer:
  - [ ] Basic oscillators (sine, saw, square)
  - [ ] Simple ADSR envelope
  - [ ] Basic lowpass filter
- [ ] Test in multiple DAWs (REAPER, Carla, Ardour)

**Deliverable:** Plugin generates audio/MIDI from patterns, syncs with DAW

### ⏳ Phase 4: Minimal GUI
**Status:** Not Started
**Duration:** 2-3 weeks

#### Tasks
- [ ] Integrate nih_plug_webview (or fallback to egui)
- [ ] Create minimal HTML/CSS/JS for editor
- [ ] Integrate CodeMirror with Strudel syntax
- [ ] Implement IPC messages:
  - GUI→Rust: `set_code`, `set_audio_mode`, `set_midi_mode`
  - Rust→GUI: `state_update`, `compilation_error`
- [ ] Add mode toggles (audio/MIDI/both)
- [ ] Add file load/save buttons
- [ ] Embed webview in plugin
- [ ] Test GUI in DAW environment

**Deliverable:** Functional code editor embedded in plugin

### ⏳ Phase 5: File Management
**Status:** Not Started
**Duration:** 1 week

#### Tasks
- [ ] Define `.strudel` file format (JSON with metadata)
- [ ] Implement file load/save dialog (Tauri or native)
- [ ] Add recent files list
- [ ] Handle file associations (optional)
- [ ] Test file persistence across DAW sessions

**Deliverable:** Patterns can be saved/loaded as separate files

### ⏳ Phase 6: Testing & Polish
**Status:** Not Started
**Duration:** 1-2 weeks

#### Tasks
- [ ] Write Rust unit tests (event buffer, timing, MIDI generation)
- [ ] Write integration tests with test hosts
- [ ] Test on multiple platforms (Linux, macOS, Windows)
- [ ] Test in multiple DAWs (REAPER, Bitwig, Ardour, Carla)
- [ ] Performance profiling (latency, CPU usage)
- [ ] Fix bugs and edge cases
- [ ] Write user documentation

**Deliverable:** Stable, tested plugin ready for release

---

## Phase 7+: Future Enhancements
**Status:** Backlog

### Audio Quality
- [ ] Port full superdough/supradough engine to Rust
- [ ] Sample playback support
- [ ] Advanced effects (reverb, delay, filters)

### GUI Features
- [ ] Pattern visualization
- [ ] Settings panel
- [ ] Multiple pattern slots/layers
- [ ] Pattern preset browser
- [ ] Syntax highlighting improvements

### DAW Integration
- [ ] Timeline position sync (bars/beats)
- [ ] Parameter automation lanes
- [ ] Tempo changes/curves support
- [ ] Project state save/restore

### Advanced Features
- [ ] VST3 support (nih-plug supports both)
- [ ] MIDI input (trigger patterns with notes)
- [ ] External editor support (WebSocket sync)
- [ ] Pattern sharing with strudel.cc
- [ ] Hydra visuals output

---

## Technical Stack

### Core Dependencies

**Rust (Plugin)**
- `nih-plug` v0.0.7 - Plugin framework
- `nih_plug_webview` - WebView GUI integration
- `deno_core` v0.278 - JavaScript runtime
- `serde` / `serde_json` - Serialization
- `crossbeam-channel` - Thread-safe messaging
- `ringbuf` - Lock-free audio buffers
- `cpal` - Audio I/O (standalone mode)
- `midir` - MIDI output (standalone mode)

**JavaScript (Pattern Engine)**
- `@strudel/core` - Pattern engine
- `@strudel/mini` - Mini notation parser
- `@strudel/tonal` - Musical utilities
- `@strudel/transpiler` - Code transpilation
- Custom audio backend (event output)

**GUI**
- CodeMirror 6 - Code editor
- Minimal HTML/CSS - UI framework
- Tauri IPC - Rust↔JS communication

---

## Event Data Structure

```rust
struct StrudelEvent {
    // Timing
    onset: f64,              // Time in beats from cycle start
    duration: f64,           // Event duration in beats

    // Output routing
    event_type: EventType,   // Audio, MIDI, or Both

    // Audio parameters (if event_type includes Audio)
    sound: Option<String>,   // Sound name (e.g., "bd", "sd")
    note: Option<f64>,       // Musical note (MIDI note number, supports microtones)
    velocity: f64,           // 0.0-1.0

    // MIDI parameters (if event_type includes MIDI)
    midi_channel: u8,        // 0-15
    midi_note: u8,           // 0-127
    midi_velocity: u8,       // 0-127

    // Effects/modulation (applies to both audio and MIDI)
    params: HashMap<String, f64>,  // gain, pan, lpf, hpf, delay, reverb, etc.
}

enum EventType {
    Audio,    // Generate audio internally
    Midi,     // Send MIDI to DAW
    Both,     // Do both
}
```

---

## Pattern File Format

`.strudel` files are JSON with metadata:

```json
{
  "version": "1.0",
  "metadata": {
    "title": "My Pattern",
    "author": "Username",
    "created": "2026-02-25T12:00:00Z",
    "modified": "2026-02-25T14:30:00Z",
    "tempo": 120,
    "tags": ["techno", "drums"]
  },
  "code": "s(\"bd sd hh sd\").bank('RolandTR909')",
  "settings": {
    "audioMode": true,
    "midiMode": false,
    "midiChannel": 0
  }
}
```

---

## Build & Development

### Development Build
```bash
cd packages/clap-plugin
cargo build
# Plugin: target/debug/libstrudel_plugin.so (Linux)
```

### Release Build
```bash
cargo build --release
cargo xtask bundle strudel-plugin --release
# Output: ~/.clap/Strudel.clap (Linux)
#         ~/Library/Audio/Plug-Ins/CLAP/Strudel.clap (macOS)
```

### Testing in DAW
```bash
# Launch Carla (lightweight host for testing)
carla

# Or REAPER
reaper
```

---

## Known Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| nih_plug_webview is experimental | High | Have egui fallback ready |
| JS runtime overhead adds latency | Medium | Profile early, optimize hot paths |
| Strudel bundle size (~1-2MB) | Low | Acceptable for plugin |
| Web Audio → Native audio mismatch | Medium | Start simple, iterate on features |
| Cross-platform GUI differences | Medium | Test on all platforms regularly |

---

## Success Metrics

### MVP Success Criteria
- [ ] Plugin loads in REAPER, Carla, Ardour
- [ ] Pattern code compiles and generates events
- [ ] Audio output works with simple synth
- [ ] MIDI output works (notes sent to DAW)
- [ ] Tempo syncs with DAW BPM
- [ ] Play/Stop syncs with transport
- [ ] Code editor works in plugin GUI
- [ ] Patterns can be saved/loaded
- [ ] Latency < 10ms
- [ ] No audio dropouts during pattern changes

---

## References

- [pluguzu](https://codeberg.org/TristanCacqueray/pluguzu) - TidalCycles CLAP plugin (inspiration)
- [nih-plug](https://github.com/robbert-vdh/nih-plug) - Plugin framework
- [nih-plug docs](https://nih-plug.robbertvanderhelm.nl/nih_plug/)
- [Deno Core](https://github.com/denoland/deno_core) - JS runtime
- [CLAP Specification](https://github.com/free-audio/clap)

---

## Changelog

### 2026-02-25
- Initial plan created
- Phase 1 started
- Architecture and technical stack defined
