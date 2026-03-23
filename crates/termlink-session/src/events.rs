//! Session event bus — structured event ring buffer with sequence tracking.
//!
//! Each session maintains an event bus for publishing and polling structured
//! events. Events have a topic (string) and a JSON payload. Each event gets
//! a monotonically increasing sequence number for delta polling.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

/// Default capacity for the event ring buffer.
const DEFAULT_CAPACITY: usize = 1024;

/// A structured event published to the session's event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Monotonically increasing sequence number.
    pub seq: u64,
    /// Event topic (e.g., "build.complete", "test.failed", "status.change").
    pub topic: String,
    /// JSON payload.
    pub payload: serde_json::Value,
    /// Timestamp (seconds since epoch).
    pub timestamp: u64,
}

/// Result of polling the event bus.
pub struct PollResult<'a> {
    /// Events matching the poll criteria.
    pub events: Vec<&'a Event>,
    /// True if events were lost due to ring buffer overflow between the
    /// caller's cursor and the oldest event still in the buffer.
    pub gap_detected: bool,
    /// Number of events that were lost (0 if no gap).
    pub events_lost: u64,
}

/// Ring buffer event bus with sequence tracking.
pub struct EventBus {
    events: VecDeque<Event>,
    next_seq: u64,
    capacity: usize,
}

impl EventBus {
    /// Create a new event bus with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a new event bus with custom capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            next_seq: 0,
            capacity,
        }
    }

    /// Emit an event. Returns the assigned sequence number.
    pub fn emit(&mut self, topic: impl Into<String>, payload: serde_json::Value) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let event = Event {
            seq,
            topic: topic.into(),
            payload,
            timestamp,
        };

        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);

        seq
    }

    /// Get all events in the buffer.
    pub fn all(&self) -> Vec<&Event> {
        self.events.iter().collect()
    }

    /// Get all events matching a topic.
    pub fn all_by_topic(&self, topic: &str) -> Vec<&Event> {
        self.events.iter().filter(|e| e.topic == topic).collect()
    }

    /// The sequence number of the oldest event still in the buffer,
    /// or `None` if the buffer is empty.
    pub fn oldest_seq(&self) -> Option<u64> {
        self.events.front().map(|e| e.seq)
    }

    /// Poll events since a given sequence number (exclusive).
    /// Returns events with seq > since_seq, plus gap detection.
    pub fn poll(&self, since_seq: u64) -> PollResult<'_> {
        let (gap_detected, events_lost) = self.detect_gap(since_seq);
        let events = self.events
            .iter()
            .filter(|e| e.seq > since_seq)
            .collect();
        PollResult { events, gap_detected, events_lost }
    }

    /// Poll events by topic since a given sequence number.
    pub fn poll_topic(&self, topic: &str, since_seq: u64) -> PollResult<'_> {
        let (gap_detected, events_lost) = self.detect_gap(since_seq);
        let events = self.events
            .iter()
            .filter(|e| e.seq > since_seq && e.topic == topic)
            .collect();
        PollResult { events, gap_detected, events_lost }
    }

    /// Check if a cursor falls before the oldest event in the buffer,
    /// indicating that events have been lost to ring buffer overflow.
    fn detect_gap(&self, since_seq: u64) -> (bool, u64) {
        if let Some(oldest) = self.oldest_seq() {
            // If since_seq + 1 < oldest, events between since_seq and oldest were evicted.
            // since_seq is exclusive (we want events > since_seq), so the first expected
            // event is since_seq + 1. If oldest > since_seq + 1, we have a gap.
            if since_seq < u64::MAX && oldest > since_seq + 1 {
                let lost = oldest - (since_seq + 1);
                return (true, lost);
            }
        }
        (false, 0)
    }

    /// List distinct topics that have events in the buffer.
    pub fn topics(&self) -> Vec<String> {
        let mut topics: Vec<String> = self
            .events
            .iter()
            .map(|e| e.topic.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        topics.sort();
        topics
    }

    /// Current sequence number (next event will get this number).
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Number of events currently in the buffer.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn emit_and_poll() {
        let mut bus = EventBus::new();
        assert!(bus.is_empty());
        assert_eq!(bus.next_seq(), 0);

        let seq0 = bus.emit("test.start", json!({"suite": "unit"}));
        assert_eq!(seq0, 0);
        assert_eq!(bus.len(), 1);

        let seq1 = bus.emit("test.pass", json!({"name": "foo"}));
        assert_eq!(seq1, 1);
        assert_eq!(bus.len(), 2);

        // Poll all (since_seq 0 is exclusive, so we need a sentinel)
        let result = bus.poll(u64::MAX); // nothing above MAX
        assert!(result.events.is_empty());
        assert!(!result.gap_detected);

        // Poll since before first event
        let result = bus.poll(0); // events with seq > 0
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].topic, "test.pass");
        assert!(!result.gap_detected);
    }

    #[test]
    fn poll_returns_events_after_seq() {
        let mut bus = EventBus::new();
        bus.emit("a", json!(1));
        bus.emit("b", json!(2));
        bus.emit("c", json!(3));

        // Since 0: events with seq > 0 → [1, 2]
        let result = bus.poll(0);
        assert_eq!(result.events.len(), 2);
        assert_eq!(result.events[0].topic, "b");
        assert_eq!(result.events[1].topic, "c");

        // Since 1: events with seq > 1 → [2]
        let result = bus.poll(1);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].topic, "c");

        // Since 2: no new events
        let result = bus.poll(2);
        assert!(result.events.is_empty());
    }

    #[test]
    fn poll_topic_filters() {
        let mut bus = EventBus::new();
        bus.emit("build.start", json!({}));
        bus.emit("test.pass", json!({}));
        bus.emit("build.done", json!({}));
        bus.emit("test.fail", json!({}));

        let result = bus.poll_topic("build.done", 0);
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].seq, 2);
        assert!(!result.gap_detected);
    }

    #[test]
    fn ring_buffer_overflow() {
        let mut bus = EventBus::with_capacity(3);

        bus.emit("a", json!(0));
        bus.emit("b", json!(1));
        bus.emit("c", json!(2));
        assert_eq!(bus.len(), 3);

        // This should evict "a"
        bus.emit("d", json!(3));
        assert_eq!(bus.len(), 3);
        assert_eq!(bus.events[0].topic, "b");
        assert_eq!(bus.events[2].topic, "d");

        // Sequence numbers continue monotonically
        assert_eq!(bus.next_seq(), 4);
    }

    #[test]
    fn gap_detection_on_overflow() {
        let mut bus = EventBus::with_capacity(3);

        // Fill and overflow: seqs 0,1,2,3,4 — buffer holds [2,3,4]
        for i in 0..5 {
            bus.emit(format!("e{i}"), json!(i));
        }

        assert_eq!(bus.oldest_seq(), Some(2));

        // Polling with cursor 0 should detect gap (events 1 lost)
        let result = bus.poll(0);
        assert!(result.gap_detected);
        assert_eq!(result.events_lost, 1); // seq 1 was lost (seq 0 not included since exclusive)
        assert_eq!(result.events.len(), 3); // seqs 2,3,4

        // Polling with cursor 1 should also detect gap (seq 2 is oldest, expected seq 2)
        let result = bus.poll(1);
        assert!(!result.gap_detected); // oldest is 2, since+1 is 2, no gap
        assert_eq!(result.events.len(), 3);

        // Polling with cursor 2 should not detect gap
        let result = bus.poll(2);
        assert!(!result.gap_detected);
        assert_eq!(result.events.len(), 2); // seqs 3,4
    }

    #[test]
    fn gap_detection_topic_poll() {
        let mut bus = EventBus::with_capacity(2);

        bus.emit("a", json!(0)); // seq 0
        bus.emit("b", json!(1)); // seq 1
        bus.emit("a", json!(2)); // seq 2, evicts seq 0

        // Polling topic "a" with cursor 0 should detect gap
        // oldest is seq 1, since_seq+1 = 1, no gap actually (1 >= 1)
        // Wait — seq 0 was evicted, oldest is 1, cursor 0 → since+1=1, oldest=1, no gap
        let result = bus.poll_topic("a", 0);
        assert!(!result.gap_detected);

        // Add more to create actual gap
        bus.emit("c", json!(3)); // seq 3, evicts seq 1

        // Now oldest is seq 2, cursor 0 → since+1=1, oldest=2, gap of 1
        let result = bus.poll_topic("a", 0);
        assert!(result.gap_detected);
        assert_eq!(result.events_lost, 1);
    }

    #[test]
    fn no_gap_on_empty_bus() {
        let bus = EventBus::new();
        let result = bus.poll(0);
        assert!(!result.gap_detected);
        assert!(result.events.is_empty());
    }

    #[test]
    fn topics_returns_distinct() {
        let mut bus = EventBus::new();
        bus.emit("build.start", json!({}));
        bus.emit("test.pass", json!({}));
        bus.emit("build.done", json!({}));
        bus.emit("test.pass", json!({}));

        let topics = bus.topics();
        assert_eq!(topics, vec!["build.done", "build.start", "test.pass"]);
    }

    #[test]
    fn empty_bus() {
        let bus = EventBus::new();
        assert!(bus.is_empty());
        assert_eq!(bus.len(), 0);
        assert_eq!(bus.next_seq(), 0);
        assert!(bus.topics().is_empty());
        assert!(bus.poll(0).events.is_empty());
    }
}
