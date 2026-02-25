// Strudel CLAP Plugin
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use nih_plug::prelude::*;
use std::sync::Arc;

mod plugin;

pub use plugin::StrudelPlugin;

// This exports the plugin entry point that the DAW will load
nih_export_clap!(StrudelPlugin);
nih_export_vst3!(StrudelPlugin);
