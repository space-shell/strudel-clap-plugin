// Main plugin implementation — Phase 4: egui code editor GUI
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, resizable_window::ResizableWindow, EguiState};
use parking_lot::Mutex;
use std::sync::Arc;

use crate::event_buffer::DoubleEventBuffer;
use crate::eval_thread::EvalThread;
use crate::gui_state::GuiState;
use crate::voice::{VoiceManager, event_params_to_note};

// ─── Active MIDI note (tracks pending NoteOff) ────────────────────────────────

/// Tracks an in-flight MIDI note so we can send the NoteOff at the right time.
struct ActiveMidiNote {
    note: u8,
    channel: u8,
    /// Absolute sample position (from DAW timeline) at which NoteOff should fire.
    end_sample: i64,
}

// ─── Combined editor user-state (passed into create_egui_editor) ──────────────

/// Bundles the two Arcs the editor closure needs: the live GUI state and the
/// egui window state (required by `ResizableWindow`).
struct GuiEditorState {
    gui_state: Arc<Mutex<GuiState>>,
    egui_state: Arc<EguiState>,
}

// ─── Plugin struct ─────────────────────────────────────────────────────────────

pub struct StrudelPlugin {
    params: Arc<StrudelParams>,
    event_buffer: Arc<DoubleEventBuffer>,
    eval_thread: Option<EvalThread>,
    sample_rate: f32,
    /// Polyphonic sine-wave synthesizer.
    voice_manager: VoiceManager,
    /// MIDI notes waiting for their NoteOff event.
    active_midi_notes: Vec<ActiveMidiNote>,
    /// State shared with the GUI thread (code, error, bpm, play status).
    gui_state: Arc<Mutex<GuiState>>,
    /// egui window size and open/closed state — required by nih_plug_egui.
    egui_state: Arc<EguiState>,
    /// Previous muted state — used to detect the unmuted→muted transition so
    /// voices can be silenced immediately when the user hits Pause.
    was_muted: bool,
}

// ─── Parameters ───────────────────────────────────────────────────────────────

#[derive(Params)]
struct StrudelParams {
    #[id = "gain"]
    pub gain: FloatParam,

    /// Pattern code persisted in the DAW project.
    /// Updated on every Evaluate; restored on project load.
    #[persist = "editor_code"]
    pub editor_code: parking_lot::Mutex<String>,
}

impl Default for StrudelPlugin {
    fn default() -> Self {
        let event_buffer = Arc::new(DoubleEventBuffer::new(8));
        Self {
            params: Arc::new(StrudelParams::default()),
            event_buffer,
            eval_thread: None,
            sample_rate: 44100.0,
            voice_manager: VoiceManager::new(),
            active_midi_notes: Vec::new(),
            gui_state: Arc::new(Mutex::new(GuiState::default())),
            egui_state: EguiState::from_size(640, 440),
            was_muted: false,
        }
    }
}

impl Default for StrudelParams {
    fn default() -> Self {
        Self {
            editor_code: parking_lot::Mutex::new(String::new()),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

// ─── Plugin trait impl ────────────────────────────────────────────────────────

impl Plugin for StrudelPlugin {
    const NAME: &'static str = "Strudel";
    const VENDOR: &'static str = "Strudel";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "hello@strudel.cc";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(0),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let editor_state = GuiEditorState {
            gui_state: self.gui_state.clone(),
            egui_state: self.egui_state.clone(),
        };

        create_egui_editor(
            self.egui_state.clone(),
            editor_state,
            |_ctx, _state| {},
            |ctx, _setter, state| {
                let gui_state = &state.gui_state;
                let egui_state = &state.egui_state;

                // ── Keyboard shortcuts (single ctx.input call per frame) ────────
                let (ctrl_enter, zoom_in, zoom_out, zoom_reset, toggle_mute) = ctx.input(|i| {
                    let ctrl_enter =
                        i.key_pressed(egui::Key::Enter) && i.modifiers.command_only();
                    // Ctrl+= (no shift) or Ctrl+Plus (numpad / shifted = on some layouts)
                    let zoom_in = (i.key_pressed(egui::Key::Equals)
                        && i.modifiers.command_only())
                        || (i.key_pressed(egui::Key::Plus) && i.modifiers.ctrl);
                    let zoom_out =
                        i.key_pressed(egui::Key::Minus) && i.modifiers.command_only();
                    let zoom_reset =
                        i.key_pressed(egui::Key::Num0) && i.modifiers.command_only();
                    let toggle_mute =
                        i.key_pressed(egui::Key::Period) && i.modifiers.command_only();
                    (ctrl_enter, zoom_in, zoom_out, zoom_reset, toggle_mute)
                });

                {
                    let mut gs = gui_state.lock();
                    if ctrl_enter {
                        gs.eval_requested = true;
                    }
                    if toggle_mute {
                        gs.muted = !gs.muted;
                    }
                    if zoom_in {
                        gs.font_size = (gs.font_size + 1.0).min(36.0);
                    }
                    if zoom_out {
                        gs.font_size = (gs.font_size - 1.0).max(8.0);
                    }
                    if zoom_reset {
                        gs.font_size = GuiState::default().font_size;
                    }
                }

                // ── Apply editor font size ──────────────────────────────────────
                let font_size = gui_state.lock().font_size;
                ctx.style_mut(|style| {
                    style.text_styles.insert(
                        egui::TextStyle::Monospace,
                        egui::FontId::monospace(font_size),
                    );
                });

                // ── Header: title + transport info ─────────────────────────────
                egui::TopBottomPanel::top("strudel_header").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Strudel");
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let gs = gui_state.lock();
                                let play_icon = if gs.is_playing { "▶" } else { "⏹" };
                                ui.label(format!("{} {:.1} BPM", play_icon, gs.bpm));
                            },
                        );
                    });
                });

                // ── Footer: Evaluate + Pause/Resume buttons + status / error ───
                egui::TopBottomPanel::bottom("strudel_footer").show(ctx, |ui| {
                    let (last_error, muted) = {
                        let gs = gui_state.lock();
                        (gs.last_error.clone(), gs.muted)
                    };

                    ui.horizontal(|ui| {
                        if ui.button("Evaluate  Ctrl+Enter").clicked() {
                            gui_state.lock().eval_requested = true;
                        }

                        let pause_label = if muted {
                            "Resume  (restart pattern)"
                        } else {
                            "Pause  (silence output)"
                        };
                        if ui.button(pause_label).clicked() {
                            gui_state.lock().muted = !muted;
                        }

                        if last_error.is_empty() && !muted {
                            ui.colored_label(egui::Color32::GREEN, "● Ready");
                        } else if muted {
                            ui.colored_label(egui::Color32::YELLOW, "⏸ Paused");
                        }
                    });

                    if !last_error.is_empty() {
                        ui.colored_label(egui::Color32::RED, &last_error);
                    }
                });

                // ── Central panel: resizable code editor ────────────────────────
                ResizableWindow::new("strudel_resize")
                    .min_size([300.0, 150.0])
                    .show(ctx, egui_state, |ui| {
                        // Capture available size before the ScrollArea expands it to
                        // infinity, so the TextEdit fills the full panel height.
                        let available = ui.available_size();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut gs = gui_state.lock();
                            ui.add(
                                egui::TextEdit::multiline(&mut gs.code)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .min_size(available),
                            );
                        });
                    });
            },
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        // Prefer persisted code over the gui_state default
        let persisted = self.params.editor_code.lock().clone();
        let default_code = if !persisted.is_empty() {
            self.gui_state.lock().code = persisted.clone();
            persisted
        } else {
            self.gui_state.lock().code.clone()
        };

        let eval_thread = EvalThread::spawn(self.event_buffer.clone(), 8);

        if let Err(e) = eval_thread.set_code(default_code) {
            nih_error!("Failed to set default pattern: {}", e);
            return false;
        }
        if let Err(e) = eval_thread.set_tempo(120.0) {
            nih_error!("Failed to set tempo: {}", e);
            return false;
        }

        self.eval_thread = Some(eval_thread);
        nih_log!("Strudel plugin initialized (sample rate: {} Hz)", self.sample_rate);
        true
    }

    fn reset(&mut self) {
        // Called when transport stops or seeks — silence everything immediately.
        self.voice_manager.reset();
        self.active_midi_notes.clear();
        nih_log!("Strudel plugin reset");
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        // ── Update GUI state and consume eval requests (non-blocking) ─────────
        // Use try_lock() so the audio thread never blocks waiting for the GUI.
        let mut currently_muted = self.was_muted;
        if let Some(mut gs) = self.gui_state.try_lock() {
            gs.bpm = transport.tempo.unwrap_or(120.0);
            gs.is_playing = transport.playing;
            currently_muted = gs.muted;

            if gs.eval_requested {
                gs.eval_requested = false;
                let code = gs.code.clone();
                // Persist the code in params so the DAW project saves it
                *self.params.editor_code.lock() = code.clone();
                // Drop the lock before sending to avoid holding it during channel ops
                drop(gs);
                if let Some(ref eval_thread) = self.eval_thread {
                    let _ = eval_thread.set_code(code);
                }
            }
        }

        // ── Silence immediately on unmuted→muted transition ───────────────────
        if currently_muted && !self.was_muted {
            self.voice_manager.reset();
            self.active_midi_notes.clear();
        }
        self.was_muted = currently_muted;

        // ── Poll for eval results and update error display ─────────────────────
        if let Some(ref eval_thread) = self.eval_thread {
            while let Some(result) = eval_thread.try_recv_result() {
                if let Some(mut gs) = self.gui_state.try_lock() {
                    if result.success {
                        gs.last_error.clear();
                    } else {
                        gs.last_error =
                            result.error.unwrap_or_else(|| "Unknown error".to_string());
                    }
                }
            }
        }

        // ── Not playing: let voices decay, do not schedule new events ─────────
        if !transport.playing {
            if let Some(ref eval_thread) = self.eval_thread {
                let _ = eval_thread.update_transport(false, 0.0);
            }
            self.render_voices(buffer);
            return ProcessStatus::Normal;
        }

        // ── Muted: voices already silenced on transition; skip scheduling ─────
        if currently_muted {
            self.render_voices(buffer);
            return ProcessStatus::Normal;
        }

        // ── Timing calculations ───────────────────────────────────────────────
        let tempo = transport.tempo.unwrap_or(120.0);
        let samples_per_beat = self.sample_rate as f64 * 60.0 / tempo;
        let samples_per_cycle = samples_per_beat * 4.0; // 4 beats per cycle
        let buffer_start_sample = transport.pos_samples().unwrap_or(0);
        let buffer_size = buffer.samples();

        let current_cycle = buffer_start_sample as f64 / samples_per_cycle;
        let buffer_cycles = buffer_size as f64 / samples_per_cycle;
        let end_cycle = current_cycle + buffer_cycles;

        // ── Sync eval thread with DAW transport ───────────────────────────────
        if let Some(ref eval_thread) = self.eval_thread {
            // Tempo from transport overrides our default
            let _ = eval_thread.set_tempo(tempo);
            let _ = eval_thread.update_transport(true, current_cycle);
        }

        // ── Read events for this buffer window ────────────────────────────────
        let events = self.event_buffer.read_events_in_range(current_cycle, end_cycle);

        // ── Send MIDI NoteOff for notes that expire within this buffer ────────
        let buffer_end_sample = buffer_start_sample + buffer_size as i64;
        self.active_midi_notes.retain(|n| {
            if n.end_sample > buffer_start_sample && n.end_sample <= buffer_end_sample {
                let timing = ((n.end_sample - buffer_start_sample - 1).max(0) as u32)
                    .min(buffer_size as u32 - 1);
                context.send_event(NoteEvent::NoteOff {
                    timing,
                    voice_id: None,
                    channel: n.channel,
                    note: n.note,
                    velocity: 0.0,
                });
                false // consumed
            } else {
                n.end_sample > buffer_end_sample // keep only future notes
            }
        });

        // ── Schedule new events ───────────────────────────────────────────────
        for event in &events {
            let onset = event.onset.0 as f64 / event.onset.1 as f64;
            let duration_cycles = event.duration.0 as f64 / event.duration.1 as f64;

            // Sample offset within this buffer (clamped to [0, buffer_size - 1])
            let offset_f64 = (onset - current_cycle) * samples_per_cycle;
            let sample_offset = (offset_f64.max(0.0) as u32).min(buffer_size as u32 - 1);

            // Duration in samples (at least 1ms)
            let duration_samples = ((duration_cycles * samples_per_cycle) as u32)
                .max((self.sample_rate * 0.001) as u32);

            let (midi_note, freq, gain) = event_params_to_note(&event.params);
            let velocity = gain.clamp(0.0, 1.0);

            // MIDI NoteOn at the precise sample offset
            context.send_event(NoteEvent::NoteOn {
                timing: sample_offset,
                voice_id: None,
                channel: 0,
                note: midi_note,
                velocity,
            });

            // Schedule the corresponding NoteOff
            let end_sample = buffer_start_sample + sample_offset as i64 + duration_samples as i64;
            self.active_midi_notes.push(ActiveMidiNote {
                note: midi_note,
                channel: 0,
                end_sample,
            });

            // Trigger built-in synth voice
            self.voice_manager.note_on(
                midi_note,
                freq,
                gain,
                duration_samples,
                self.sample_rate,
            );
        }

        // ── Generate audio from synth voices ──────────────────────────────────
        self.render_voices(buffer);

        ProcessStatus::Normal
    }
}

impl StrudelPlugin {
    /// Mix all active synth voices into the output buffer (mono → all channels).
    fn render_voices(&mut self, buffer: &mut Buffer) {
        for channel_samples in buffer.iter_samples() {
            let synth = self.voice_manager.process_sample();
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample = synth * gain;
            }
        }
    }
}

// ─── CLAP / VST3 metadata ────────────────────────────────────────────────────

impl ClapPlugin for StrudelPlugin {
    const CLAP_ID: &'static str = "cc.strudel.clap-plugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Live coding pattern language for music");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for StrudelPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"StrudelClapPlugi";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}
