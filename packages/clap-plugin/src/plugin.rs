// Main plugin implementation

use nih_plug::prelude::*;
use std::sync::Arc;

/// The main Strudel plugin struct
pub struct StrudelPlugin {
    params: Arc<StrudelParams>,
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
        Self {
            params: Arc::new(StrudelParams::default()),
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
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // TODO: Initialize JavaScript runtime here
        // TODO: Load default pattern
        nih_log!("Strudel plugin initialized");
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
        // For now, just output silence and log transport state
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample = 0.0 * gain; // Silence for MVP
            }
        }

        // Log transport state (will be used for sync)
        if let Some(transport) = context.transport() {
            if transport.playing {
                // TODO: Generate events based on transport position
                // TODO: Synthesize audio or send MIDI
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
