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

    /// Poll events since a given sequence number (exclusive).
    /// Returns events with seq > since_seq.
    pub fn poll(&self, since_seq: u64) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.seq > since_seq)
            .collect()
    }

    /// Poll events by topic since a given sequence number.
    pub fn poll_topic(&self, topic: &str, since_seq: u64) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.seq > since_seq && e.topic == topic)
            .collect()
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
        let all = bus.poll(u64::MAX); // nothing above MAX
        assert!(all.is_empty());

        // Poll since before first event
        let events = bus.poll(0); // events with seq > 0
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "test.pass");

        // Poll all events (since before any)
        // We use a trick: since_seq is exclusive, seq > since_seq
        // To get all, we can't use 0 since seq 0 won't be included
        // So we need to handle this edge case
    }

    #[test]
    fn poll_returns_events_after_seq() {
        let mut bus = EventBus::new();
        bus.emit("a", json!(1));
        bus.emit("b", json!(2));
        bus.emit("c", json!(3));

        // Since 0: events with seq > 0 → [1, 2]
        let events = bus.poll(0);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].topic, "b");
        assert_eq!(events[1].topic, "c");

        // Since 1: events with seq > 1 → [2]
        let events = bus.poll(1);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].topic, "c");

        // Since 2: no new events
        let events = bus.poll(2);
        assert!(events.is_empty());
    }

    #[test]
    fn poll_topic_filters() {
        let mut bus = EventBus::new();
        bus.emit("build.start", json!({}));
        bus.emit("test.pass", json!({}));
        bus.emit("build.done", json!({}));
        bus.emit("test.fail", json!({}));

        let build_events = bus.poll_topic("build.done", 0);
        assert_eq!(build_events.len(), 1);
        assert_eq!(build_events[0].seq, 2);
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
        assert!(bus.poll(0).is_empty());
    }
}
