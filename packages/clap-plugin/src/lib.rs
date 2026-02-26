// Strudel CLAP Plugin
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use nih_plug::prelude::*;

mod plugin;
mod js_runtime;
mod event_buffer;
mod eval_thread;
mod gui_state;
mod voice;

pub use plugin::StrudelPlugin;

// This exports the plugin entry point that the DAW will load
nih_export_clap!(StrudelPlugin);
nih_export_vst3!(StrudelPlugin);
