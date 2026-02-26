// Main plugin implementation

use nih_plug::prelude::*;
use std::sync::Arc;

use crate::event_buffer::DoubleEventBuffer;
use crate::eval_thread::EvalThread;

/// The main Strudel plugin struct
pub struct StrudelPlugin {
    params: Arc<StrudelParams>,
    /// Event buffer for pre-rendered pattern events
    event_buffer: Arc<DoubleEventBuffer>,
    /// Evaluation thread for background pattern processing
    eval_thread: Option<EvalThread>,
    /// Sample rate in Hz
    sample_rate: f32,
}

/// Plugin parameters
#[derive(Params)]
struct StrudelParams {
    /// Master gain parameter (for testing)
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for StrudelPlugin {
    fn default() -> Self {
        // Create event buffer with 8 cycles of pre-rendering
        let event_buffer = Arc::new(DoubleEventBuffer::new(8));

        Self {
            params: Arc::new(StrudelParams::default()),
            event_buffer,
            eval_thread: None,
            sample_rate: 44100.0,
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

impl Plugin for StrudelPlugin {
    const NAME: &'static str = "Strudel";
    const VENDOR: &'static str = "Strudel";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "hello@strudel.cc";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // Stereo output, no input
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(0),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    // Enable MIDI output
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
        // Store sample rate
        self.sample_rate = buffer_config.sample_rate;

        // Spawn evaluation thread with 8 cycles of pre-rendering
        let eval_thread = EvalThread::spawn(self.event_buffer.clone(), 8);

        // Set default pattern (simple kick drum pattern)
        if let Err(e) = eval_thread.set_code("s('bd sd hh sd')".to_string()) {
            nih_error!("Failed to set default pattern: {}", e);
            return false;
        }

        // Set initial tempo from params (default 120 BPM)
        if let Err(e) = eval_thread.set_tempo(120.0) {
            nih_error!("Failed to set tempo: {}", e);
            return false;
        }

        self.eval_thread = Some(eval_thread);

        nih_log!("Strudel plugin initialized (sample rate: {} Hz)", self.sample_rate);
        true
    }

    fn reset(&mut self) {
        // Reset plugin state when transport stops or seeks
        nih_log!("Strudel plugin reset");
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let transport = context.transport();

        // Sync with transport state
        if let Some(ref eval_thread) = self.eval_thread {
            // Calculate current cycle from sample position and tempo
            // 1 cycle = 1 bar at current tempo
            let tempo = transport.tempo.unwrap_or(120.0);
            let samples_per_beat = self.sample_rate as f64 * 60.0 / tempo;
            let samples_per_cycle = samples_per_beat * 4.0; // 4 beats per cycle
            let current_cycle = transport.pos_samples().unwrap_or(0) as f64 / samples_per_cycle;

            // Update transport state in eval thread
            if let Err(e) = eval_thread.update_transport(transport.playing, current_cycle) {
                nih_error!("Failed to update transport: {}", e);
            }

            // Read events from buffer for this audio block
            let start_time = current_cycle;
            let end_time = current_cycle + (buffer.samples() as f64 / samples_per_cycle);
            let _events = self.event_buffer.read_events_in_range(start_time, end_time);

            // TODO Phase 3: Synthesize audio from events
            // For now, output silence
        }

        // Output silence (Phase 3 will generate audio)
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample = 0.0 * gain;
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // TODO: Implement GUI with egui or webview
        None
    }
}

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
