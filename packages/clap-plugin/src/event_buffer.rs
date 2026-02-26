// Event Buffer System for Strudel Plugin
// Copyright (C) 2026 Strudel Contributors
// Licensed under AGPL-3.0-or-later

use crate::js_runtime::StrudelEvent;
use parking_lot::RwLock;
use std::sync::Arc;

/// A fixed-size circular buffer for storing pattern events
/// Pre-renders N cycles ahead of the current playback position
#[derive(Debug, Clone)]
pub struct EventBuffer {
    /// All events currently in the buffer
    events: Vec<StrudelEvent>,
    /// The start cycle of the buffer (in cycles)
    start_cycle: i64,
    /// The end cycle of the buffer (in cycles)
    end_cycle: i64,
    /// Number of cycles to pre-render
    buffer_size_cycles: i64,
}

impl EventBuffer {
    /// Create a new event buffer with the specified size in cycles
    pub fn new(buffer_size_cycles: i64) -> Self {
        Self {
            events: Vec::new(),
            start_cycle: 0,
            end_cycle: 0,
            buffer_size_cycles,
        }
    }

    /// Update the buffer with new events for a cycle range
    /// This replaces all events in the buffer
    pub fn update(&mut self, events: Vec<StrudelEvent>, start_cycle: i64, end_cycle: i64) {
        self.events = events;
        self.start_cycle = start_cycle;
        self.end_cycle = end_cycle;
    }

    /// Get events that fall within a specific time range (in cycles)
    /// Returns events where onset is in [start_time, end_time)
    pub fn get_events_in_range(&self, start_time: f64, end_time: f64) -> Vec<StrudelEvent> {
        self.events
            .iter()
            .filter(|event| {
                let onset_time = Self::fraction_to_f64(event.onset);
                onset_time >= start_time && onset_time < end_time
            })
            .cloned()
            .collect()
    }

    /// Get all events in the buffer
    pub fn get_all_events(&self) -> &[StrudelEvent] {
        &self.events
    }

    /// Check if the buffer covers a specific cycle
    pub fn contains_cycle(&self, cycle: i64) -> bool {
        cycle >= self.start_cycle && cycle < self.end_cycle
    }

    /// Get the start cycle of the buffer
    pub fn start_cycle(&self) -> i64 {
        self.start_cycle
    }

    /// Get the end cycle of the buffer
    pub fn end_cycle(&self) -> i64 {
        self.end_cycle
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the number of events in the buffer
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Clear all events from the buffer
    pub fn clear(&mut self) {
        self.events.clear();
        self.start_cycle = 0;
        self.end_cycle = 0;
    }

    /// Convert a fraction (numerator, denominator) to f64
    fn fraction_to_f64(frac: (i64, i64)) -> f64 {
        frac.0 as f64 / frac.1 as f64
    }
}

/// Double-buffered event buffer for lock-free reading on the audio thread
/// The audio thread reads from one buffer while the eval thread writes to the other
pub struct DoubleEventBuffer {
    /// The buffer currently being read by the audio thread
    read_buffer: Arc<RwLock<EventBuffer>>,
    /// The buffer currently being written by the eval thread
    write_buffer: Arc<RwLock<EventBuffer>>,
}

impl DoubleEventBuffer {
    /// Create a new double event buffer with the specified size in cycles
    pub fn new(buffer_size_cycles: i64) -> Self {
        Self {
            read_buffer: Arc::new(RwLock::new(EventBuffer::new(buffer_size_cycles))),
            write_buffer: Arc::new(RwLock::new(EventBuffer::new(buffer_size_cycles))),
        }
    }

    /// Get events from the read buffer (called by audio thread)
    /// This is a non-blocking read operation
    pub fn read_events_in_range(&self, start_time: f64, end_time: f64) -> Vec<StrudelEvent> {
        let buffer = self.read_buffer.read();
        buffer.get_events_in_range(start_time, end_time)
    }

    /// Get all events from the read buffer (called by audio thread)
    pub fn read_all_events(&self) -> Vec<StrudelEvent> {
        let buffer = self.read_buffer.read();
        buffer.get_all_events().to_vec()
    }

    /// Check if the read buffer contains a specific cycle
    pub fn read_contains_cycle(&self, cycle: i64) -> bool {
        let buffer = self.read_buffer.read();
        buffer.contains_cycle(cycle)
    }

    /// Update the write buffer with new events (called by eval thread)
    pub fn write_events(&self, events: Vec<StrudelEvent>, start_cycle: i64, end_cycle: i64) {
        let mut buffer = self.write_buffer.write();
        buffer.update(events, start_cycle, end_cycle);
    }

    /// Swap the read and write buffers (called by eval thread after writing)
    /// This is an atomic operation that makes the new events available to the audio thread
    pub fn swap_buffers(&self) {
        // We need to acquire both locks to swap
        let mut read = self.read_buffer.write();
        let mut write = self.write_buffer.write();
        std::mem::swap(&mut *read, &mut *write);
    }

    /// Clear both buffers
    pub fn clear(&self) {
        self.read_buffer.write().clear();
        self.write_buffer.write().clear();
    }

    /// Get the start cycle of the read buffer
    pub fn read_start_cycle(&self) -> i64 {
        self.read_buffer.read().start_cycle()
    }

    /// Get the end cycle of the read buffer
    pub fn read_end_cycle(&self) -> i64 {
        self.read_buffer.read().end_cycle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_event(onset: (i64, i64), duration: (i64, i64)) -> StrudelEvent {
        StrudelEvent {
            onset,
            duration,
            event_type: "audio".to_string(),
            params: json!({"s": "bd"}),
        }
    }

    #[test]
    fn test_event_buffer_creation() {
        let buffer = EventBuffer::new(4);
        assert_eq!(buffer.buffer_size_cycles, 4);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_event_buffer_update() {
        let mut buffer = EventBuffer::new(4);
        let events = vec![
            create_test_event((0, 1), (1, 4)),
            create_test_event((1, 4), (1, 4)),
        ];

        buffer.update(events.clone(), 0, 1);
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.start_cycle(), 0);
        assert_eq!(buffer.end_cycle(), 1);
    }

    #[test]
    fn test_event_buffer_get_events_in_range() {
        let mut buffer = EventBuffer::new(4);
        let events = vec![
            create_test_event((0, 1), (1, 4)),     // onset = 0.0
            create_test_event((1, 4), (1, 4)),     // onset = 0.25
            create_test_event((1, 2), (1, 4)),     // onset = 0.5
            create_test_event((3, 4), (1, 4)),     // onset = 0.75
        ];

        buffer.update(events, 0, 1);

        // Get events in range [0.0, 0.5)
        let events_in_range = buffer.get_events_in_range(0.0, 0.5);
        assert_eq!(events_in_range.len(), 2); // Events at 0.0 and 0.25

        // Get events in range [0.25, 1.0)
        let events_in_range = buffer.get_events_in_range(0.25, 1.0);
        assert_eq!(events_in_range.len(), 3); // Events at 0.25, 0.5, 0.75
    }

    #[test]
    fn test_event_buffer_contains_cycle() {
        let mut buffer = EventBuffer::new(4);
        buffer.update(vec![], 5, 9);

        assert!(buffer.contains_cycle(5));
        assert!(buffer.contains_cycle(7));
        assert!(buffer.contains_cycle(8));
        assert!(!buffer.contains_cycle(4));
        assert!(!buffer.contains_cycle(9));
    }

    #[test]
    fn test_double_buffer_creation() {
        let buffer = DoubleEventBuffer::new(4);
        assert_eq!(buffer.read_all_events().len(), 0);
    }

    #[test]
    fn test_double_buffer_write_and_swap() {
        let buffer = DoubleEventBuffer::new(4);
        let events = vec![
            create_test_event((0, 1), (1, 4)),
            create_test_event((1, 4), (1, 4)),
        ];

        // Write to write buffer
        buffer.write_events(events.clone(), 0, 1);

        // Read buffer should still be empty
        assert_eq!(buffer.read_all_events().len(), 0);

        // Swap buffers
        buffer.swap_buffers();

        // Now read buffer should have the events
        assert_eq!(buffer.read_all_events().len(), 2);
        assert_eq!(buffer.read_start_cycle(), 0);
        assert_eq!(buffer.read_end_cycle(), 1);
    }

    #[test]
    fn test_double_buffer_read_events_in_range() {
        let buffer = DoubleEventBuffer::new(4);
        let events = vec![
            create_test_event((0, 1), (1, 4)),
            create_test_event((1, 2), (1, 4)),
        ];

        buffer.write_events(events, 0, 1);
        buffer.swap_buffers();

        let events_in_range = buffer.read_events_in_range(0.0, 0.5);
        assert_eq!(events_in_range.len(), 1); // Only event at 0.0
    }

    #[test]
    fn test_double_buffer_clear() {
        let buffer = DoubleEventBuffer::new(4);
        let events = vec![create_test_event((0, 1), (1, 4))];

        buffer.write_events(events, 0, 1);
        buffer.swap_buffers();

        assert_eq!(buffer.read_all_events().len(), 1);

        buffer.clear();

        assert_eq!(buffer.read_all_events().len(), 0);
    }
}
