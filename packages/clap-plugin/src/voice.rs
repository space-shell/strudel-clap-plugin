// Polyphonic synthesizer voice management
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;
const MAX_VOICES: usize = 16;

/// A single synthesis voice: sine oscillator with a simple AR envelope.
#[derive(Clone)]
struct Voice {
    active: bool,
    /// MIDI note number being played (for voice stealing logic)
    note: u8,
    /// Current oscillator phase in [0, 2π)
    phase: f32,
    /// Phase increment per sample (= frequency * 2π / sample_rate)
    phase_increment: f32,
    gain: f32,
    /// Samples elapsed since voice was triggered
    elapsed: u32,
    /// Total duration in samples (note off after this)
    duration_samples: u32,
    /// Attack ramp duration in samples
    attack_samples: u32,
    /// Release ramp duration in samples
    release_samples: u32,
}

impl Voice {
    fn new() -> Self {
        Self {
            active: false,
            note: 0,
            phase: 0.0,
            phase_increment: 0.0,
            gain: 1.0,
            elapsed: 0,
            duration_samples: 0,
            attack_samples: 0,
            release_samples: 0,
        }
    }

    fn trigger(&mut self, note: u8, frequency: f32, gain: f32, duration_samples: u32, sample_rate: f32) {
        self.active = true;
        self.note = note;
        self.phase = 0.0;
        self.phase_increment = frequency * TWO_PI / sample_rate;
        self.gain = gain;
        self.elapsed = 0;
        self.duration_samples = duration_samples.max(64);

        // 5ms attack
        self.attack_samples = (0.005 * sample_rate) as u32;
        // Release: the smaller of 50ms or 20% of total duration
        self.release_samples = ((0.05 * sample_rate) as u32)
            .min(self.duration_samples / 5)
            .max(1);
    }

    /// Generate and return the next sample, advancing the voice state.
    fn next_sample(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Envelope
        let env = if self.elapsed < self.attack_samples {
            // Linear attack
            self.elapsed as f32 / self.attack_samples.max(1) as f32
        } else {
            let release_start = self.duration_samples.saturating_sub(self.release_samples);
            if self.elapsed >= release_start {
                // Linear release
                let rel = self.elapsed - release_start;
                1.0 - (rel as f32 / self.release_samples as f32)
            } else {
                // Sustain
                1.0
            }
        };

        // Sine oscillator
        let osc = self.phase.sin();
        self.phase += self.phase_increment;
        if self.phase >= TWO_PI {
            self.phase -= TWO_PI;
        }

        self.elapsed += 1;
        if self.elapsed >= self.duration_samples {
            self.active = false;
        }

        // Scale down so multiple voices don't clip
        osc * env * self.gain * 0.15
    }

    fn stop(&mut self) {
        self.active = false;
    }
}

/// Polyphonic voice manager. Up to MAX_VOICES simultaneous notes.
pub struct VoiceManager {
    voices: Vec<Voice>,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            voices: (0..MAX_VOICES).map(|_| Voice::new()).collect(),
        }
    }

    /// Trigger a new note. Steals the oldest voice if all slots are occupied.
    pub fn note_on(&mut self, note: u8, frequency: f32, gain: f32, duration_samples: u32, sample_rate: f32) {
        // Prefer a free voice
        let idx = self.voices.iter().position(|v| !v.active)
            .unwrap_or_else(|| {
                // All busy: steal voice 0 (simple round-robin could be nicer, but this works)
                0
            });
        self.voices[idx].trigger(note, frequency, gain, duration_samples, sample_rate);
    }

    /// Mix all active voices and return the next mono sample.
    pub fn process_sample(&mut self) -> f32 {
        self.voices.iter_mut()
            .filter(|v| v.active)
            .map(|v| v.next_sample())
            .sum()
    }

    /// Silence all voices immediately (called on transport reset/stop).
    pub fn reset(&mut self) {
        for v in &mut self.voices {
            v.stop();
        }
    }
}

/// Convert a MIDI note number to a frequency in Hz.
pub fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

/// Map a Strudel sound name to a General MIDI percussion note (ch 9).
pub fn sound_to_gm_note(s: &str) -> u8 {
    match s.split(':').next().unwrap_or(s) {
        "bd" | "kick" | "bassdrum" => 36,
        "sd" | "sn" | "snare" => 38,
        "hh" | "ch" => 42,
        "oh" => 46,
        "cp" | "clap" => 39,
        "rim" | "rs" => 37,
        "mt" => 47,
        "ht" => 50,
        "lt" => 45,
        "cowbell" | "cb" => 56,
        "clave" | "cl" => 75,
        "cymbal" | "cy" => 49,
        "ride" | "rc" => 51,
        _ => 60, // default: C4
    }
}

/// Map a Strudel sound name to a characteristic frequency for the built-in synth.
pub fn sound_to_freq(s: &str) -> f32 {
    match s.split(':').next().unwrap_or(s) {
        "bd" | "kick" | "bassdrum" => 55.0,   // A1 sub-bass thump
        "sd" | "sn" | "snare" => 185.0,        // F#3 midrange crack
        "hh" | "ch" => 8000.0,                 // high metallic transient
        "oh" => 6000.0,                         // open hi-hat shimmer
        "cp" | "clap" => 900.0,                // clap attack frequency
        "rim" | "rs" => 400.0,                  // rimshot ping
        "mt" => 220.0,                          // mid tom
        "ht" => 330.0,                          // high tom
        "lt" => 147.0,                          // low tom
        "cowbell" | "cb" => 562.0,
        _ => 440.0, // A4 default
    }
}

/// Extract MIDI note, synth frequency, and gain from a Strudel event's params.
///
/// Priority:
///   1. `note` field (absolute MIDI note)
///   2. `n` field when no `s` field (semitone offset from C4 = 60)
///   3. `s` field (drum sound → GM percussion note + characteristic freq)
///   4. Default: C4 / 440 Hz
pub fn event_params_to_note(params: &serde_json::Value) -> (u8, f32, f32) {
    let gain = params["gain"].as_f64().unwrap_or(0.8) as f32;

    // Explicit note field (melodic)
    if let Some(n) = params["note"].as_f64() {
        let midi = n.clamp(0.0, 127.0) as u8;
        return (midi, midi_to_freq(midi), gain);
    }

    // n field without s field: treat as MIDI note (or offset from C4)
    let has_s = params["s"].is_string();
    if !has_s {
        if let Some(n) = params["n"].as_f64() {
            // Strudel n() uses absolute note numbers when not combined with s()
            let midi = n.clamp(0.0, 127.0) as u8;
            return (midi, midi_to_freq(midi), gain);
        }
    }

    // Sound name (drums / sample banks)
    if let Some(s) = params["s"].as_str() {
        let midi = sound_to_gm_note(s);
        let freq = sound_to_freq(s);
        return (midi, freq, gain);
    }

    // Default: C4
    (60, midi_to_freq(60), gain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_to_freq() {
        let f = midi_to_freq(69);
        assert!((f - 440.0).abs() < 0.01, "A4 should be 440 Hz, got {}", f);

        let f = midi_to_freq(60);
        assert!((f - 261.63).abs() < 0.1, "C4 should be ~261.63 Hz, got {}", f);
    }

    #[test]
    fn test_voice_manager_basic() {
        let mut vm = VoiceManager::new();
        vm.note_on(60, 261.63, 0.8, 4410, 44100.0);
        // Should produce non-silent output
        let s = vm.process_sample();
        // First sample is in attack phase (near 0 due to zero-phase start)
        // Just check we don't panic and it returns a finite value
        assert!(s.is_finite());
    }

    #[test]
    fn test_event_params_to_note_drum() {
        let params = serde_json::json!({"s": "bd", "gain": 0.9});
        let (note, freq, gain) = event_params_to_note(&params);
        assert_eq!(note, 36); // GM bass drum
        assert!((gain - 0.9).abs() < 0.001);
        assert!(freq > 0.0);
    }

    #[test]
    fn test_event_params_to_note_melodic() {
        let params = serde_json::json!({"note": 64, "gain": 0.7});
        let (note, _freq, gain) = event_params_to_note(&params);
        assert_eq!(note, 64); // E4
        assert!((gain - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_voice_silences_after_duration() {
        let mut vm = VoiceManager::new();
        let duration = 100u32;
        vm.note_on(60, 261.63, 1.0, duration, 44100.0);
        for _ in 0..duration + 10 {
            vm.process_sample();
        }
        // After duration, voice should be inactive
        let s = vm.process_sample();
        assert_eq!(s, 0.0);
    }
}
