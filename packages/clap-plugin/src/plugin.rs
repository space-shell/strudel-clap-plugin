// Main plugin implementation — Phase 3: MIDI output + built-in synthesizer
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::event_buffer::DoubleEventBuffer;
use crate::eval_thread::EvalThread;
use crate::voice::{VoiceManager, event_params_to_note, midi_to_freq};

// ─── Active MIDI note (tracks pending NoteOff) ────────────────────────────────

/// Tracks an in-flight MIDI note so we can send the NoteOff at the right time.
struct ActiveMidiNote {
    note: u8,
    channel: u8,
    /// Absolute sample position (from DAW timeline) at which NoteOff should fire.
    end_sample: i64,
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
}

// ─── Parameters ───────────────────────────────────────────────────────────────

#[derive(Params)]
struct StrudelParams {
    #[id = "gain"]
    pub gain: FloatParam,
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
        }
    }
}

impl Default for StrudelParams {
    fn default() -> Self {
        Self {
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

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        let eval_thread = EvalThread::spawn(self.event_buffer.clone(), 8);

        if let Err(e) = eval_thread.set_code("s('bd sd hh sd')".to_string()) {
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

        // ── Not playing: let voices decay, do not schedule new events ─────────
        if !transport.playing {
            if let Some(ref eval_thread) = self.eval_thread {
                let _ = eval_thread.update_transport(false, 0.0);
            }
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // TODO Phase 4: Implement GUI
        None
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
