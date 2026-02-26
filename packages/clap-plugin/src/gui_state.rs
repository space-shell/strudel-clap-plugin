// GUI state shared between the audio thread and the GUI thread
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

/// State shared between the audio thread and the GUI thread.
///
/// Wrapped in `Arc<parking_lot::Mutex<GuiState>>`. The lock is held only briefly
/// in both threads: the audio thread uses `try_lock()` so it never blocks, and
/// the GUI thread locks briefly to read/write fields.
pub struct GuiState {
    /// Current text in the editor (what the user has typed).
    pub code: String,
    /// Empty string = last evaluation succeeded; non-empty = error from last eval.
    pub last_error: String,
    /// The GUI sets this to `true` when the user triggers Evaluate (button or
    /// Ctrl+Enter). `process()` consumes it (clears to `false`) and forwards the
    /// code to the eval thread.
    pub eval_requested: bool,
    /// Current BPM — written by `process()` every buffer so the GUI can display it.
    pub bpm: f64,
    /// Whether the DAW is currently playing — written by `process()` every buffer.
    pub is_playing: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            code: "s('bd sd hh sd')".to_string(),
            last_error: String::new(),
            eval_requested: false,
            bpm: 120.0,
            is_playing: false,
        }
    }
}
