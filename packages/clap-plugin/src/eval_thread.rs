// Evaluation Thread for Background Pattern Evaluation
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use crate::event_buffer::DoubleEventBuffer;
use crate::js_runtime::StrudelRuntime;
use crossbeam_channel::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Messages sent to the evaluation thread
#[derive(Debug, Clone)]
pub enum EvalMessage {
    /// Set new pattern code to evaluate
    SetCode(String),
    /// Update the tempo (BPM)
    SetTempo(f64),
    /// Update transport state (playing, cycle position)
    UpdateTransport { playing: bool, cycle: f64 },
    /// Shutdown the evaluation thread
    Shutdown,
}

/// The evaluation thread handle
pub struct EvalThread {
    /// Sender for messages to the eval thread
    sender: Sender<EvalMessage>,
    /// Join handle for the eval thread
    handle: Option<JoinHandle<()>>,
}

impl EvalThread {
    /// Spawn a new evaluation thread
    ///
    /// # Arguments
    /// * `event_buffer` - The double event buffer to write events to
    /// * `cycles_ahead` - How many cycles to pre-render ahead
    pub fn spawn(event_buffer: Arc<DoubleEventBuffer>, cycles_ahead: i64) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();

        let handle = thread::Builder::new()
            .name("strudel-eval".to_string())
            .spawn(move || {
                eval_thread_loop(receiver, event_buffer, cycles_ahead);
            })
            .expect("Failed to spawn evaluation thread");

        Self {
            sender,
            handle: Some(handle),
        }
    }

    /// Send a message to the evaluation thread
    pub fn send(&self, message: EvalMessage) -> Result<(), String> {
        self.sender
            .send(message)
            .map_err(|e| format!("Failed to send message to eval thread: {}", e))
    }

    /// Set the pattern code
    pub fn set_code(&self, code: String) -> Result<(), String> {
        self.send(EvalMessage::SetCode(code))
    }

    /// Set the tempo
    pub fn set_tempo(&self, bpm: f64) -> Result<(), String> {
        self.send(EvalMessage::SetTempo(bpm))
    }

    /// Update transport state
    pub fn update_transport(&self, playing: bool, cycle: f64) -> Result<(), String> {
        self.send(EvalMessage::UpdateTransport { playing, cycle })
    }

    /// Shutdown the evaluation thread
    pub fn shutdown(&self) -> Result<(), String> {
        self.send(EvalMessage::Shutdown)
    }
}

impl Drop for EvalThread {
    fn drop(&mut self) {
        // Try to shutdown gracefully
        let _ = self.shutdown();

        // Wait for thread to finish (with timeout)
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// State maintained by the evaluation thread
struct EvalThreadState {
    /// The JavaScript runtime
    runtime: StrudelRuntime,
    /// Current tempo in BPM
    tempo: f64,
    /// Whether playback is active
    playing: bool,
    /// Current playback position in cycles
    current_cycle: f64,
    /// Whether a pattern is currently loaded
    has_pattern: bool,
}

impl EvalThreadState {
    fn new() -> Self {
        let runtime = StrudelRuntime::new().expect("Failed to create Strudel runtime");

        Self {
            runtime,
            tempo: 120.0,
            playing: false,
            current_cycle: 0.0,
            has_pattern: false,
        }
    }
}

/// Main loop for the evaluation thread
fn eval_thread_loop(
    receiver: Receiver<EvalMessage>,
    event_buffer: Arc<DoubleEventBuffer>,
    cycles_ahead: i64,
) {
    let mut state = EvalThreadState::new();

    loop {
        // Wait for messages
        match receiver.recv() {
            Ok(message) => {
                match message {
                    EvalMessage::SetCode(code) => {
                        handle_set_code(&mut state, &code, &event_buffer, cycles_ahead);
                    }
                    EvalMessage::SetTempo(bpm) => {
                        handle_set_tempo(&mut state, bpm);
                    }
                    EvalMessage::UpdateTransport { playing, cycle } => {
                        handle_update_transport(
                            &mut state,
                            playing,
                            cycle,
                            &event_buffer,
                            cycles_ahead,
                        );
                    }
                    EvalMessage::Shutdown => {
                        break;
                    }
                }
            }
            Err(_) => {
                // Channel closed, exit
                break;
            }
        }
    }
}

/// Handle SetCode message
fn handle_set_code(
    state: &mut EvalThreadState,
    code: &str,
    event_buffer: &Arc<DoubleEventBuffer>,
    cycles_ahead: i64,
) {
    let result = state.runtime.set_code(code);

    if result.success {
        state.has_pattern = true;

        // Pre-render events from current position
        if state.playing {
            render_events(state, event_buffer, cycles_ahead);
        }
    } else {
        state.has_pattern = false;
        event_buffer.clear();

        if let Some(error) = result.error {
            eprintln!("Pattern evaluation error: {}", error);
        }
    }
}

/// Handle SetTempo message
fn handle_set_tempo(state: &mut EvalThreadState, bpm: f64) {
    state.tempo = bpm;
    if let Err(e) = state.runtime.set_tempo(bpm) {
        eprintln!("Failed to set tempo: {}", e);
    }
}

/// Handle UpdateTransport message
fn handle_update_transport(
    state: &mut EvalThreadState,
    playing: bool,
    cycle: f64,
    event_buffer: &Arc<DoubleEventBuffer>,
    cycles_ahead: i64,
) {
    let was_playing = state.playing;
    state.playing = playing;
    state.current_cycle = cycle;

    // If playback started or position changed significantly, re-render
    if playing && (!was_playing || !event_buffer.read_contains_cycle(cycle as i64)) {
        if state.has_pattern {
            render_events(state, event_buffer, cycles_ahead);
        }
    }

    // If playback stopped, clear buffer
    if !playing && was_playing {
        event_buffer.clear();
    }
}

/// Render events for the current position + cycles_ahead
fn render_events(
    state: &EvalThreadState,
    event_buffer: &Arc<DoubleEventBuffer>,
    cycles_ahead: i64,
) {
    let start_cycle = state.current_cycle.floor() as i64;
    let end_cycle = start_cycle + cycles_ahead;

    // Query the pattern for events
    match state
        .runtime
        .query_pattern((start_cycle, 1), (end_cycle, 1))
    {
        Ok(events) => {
            // Write events to the write buffer
            event_buffer.write_events(events, start_cycle, end_cycle);

            // Swap buffers to make events available to audio thread
            event_buffer.swap_buffers();
        }
        Err(e) => {
            eprintln!("Failed to query pattern: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_thread_spawn() {
        let buffer = Arc::new(DoubleEventBuffer::new(4));
        let thread = EvalThread::spawn(buffer.clone(), 4);

        // Thread should be running
        assert!(thread.handle.is_some());

        // Shutdown
        thread.shutdown().unwrap();
    }

    #[test]
    fn test_eval_thread_set_code() {
        let buffer = Arc::new(DoubleEventBuffer::new(4));
        let thread = EvalThread::spawn(buffer.clone(), 4);

        // Set a simple pattern
        thread.set_code("s('bd')".to_string()).unwrap();

        // Give it a moment to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Shutdown
        thread.shutdown().unwrap();
    }

    #[test]
    fn test_eval_thread_set_tempo() {
        let buffer = Arc::new(DoubleEventBuffer::new(4));
        let thread = EvalThread::spawn(buffer.clone(), 4);

        // Set tempo
        thread.set_tempo(140.0).unwrap();

        // Give it a moment to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Shutdown
        thread.shutdown().unwrap();
    }

    #[test]
    fn test_eval_thread_transport() {
        let buffer = Arc::new(DoubleEventBuffer::new(4));
        let thread = EvalThread::spawn(buffer.clone(), 4);

        // Set a pattern first
        thread.set_code("s('bd sd')".to_string()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Start playback
        thread.update_transport(true, 0.0).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Buffer should have events now
        let events = buffer.read_all_events();
        assert!(!events.is_empty(), "Buffer should contain events");

        // Shutdown
        thread.shutdown().unwrap();
    }
}
