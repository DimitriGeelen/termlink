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

    #[test]
    fn all_and_all_by_topic() {
        let mut bus = EventBus::new();
        bus.emit("build.start", json!({}));
        bus.emit("test.pass", json!({}));
        bus.emit("build.done", json!({}));

        let all = bus.all();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].seq, 0);
        assert_eq!(all[2].seq, 2);

        let builds = bus.all_by_topic("build.start");
        assert_eq!(builds.len(), 1);
        assert_eq!(builds[0].seq, 0);

        let missing = bus.all_by_topic("nonexistent");
        assert!(missing.is_empty());
    }

    #[test]
    fn event_serde_roundtrip() {
        let event = Event {
            seq: 42,
            topic: "task.complete".into(),
            payload: json!({"result": "ok", "duration_ms": 1500}),
            timestamp: 1711756800,
        };
        let json_str = serde_json::to_string(&event).unwrap();
        let parsed: Event = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.seq, 42);
        assert_eq!(parsed.topic, "task.complete");
        assert_eq!(parsed.payload["result"], "ok");
        assert_eq!(parsed.timestamp, 1711756800);
    }

    #[test]
    fn poll_topic_no_matches() {
        let mut bus = EventBus::new();
        bus.emit("build.start", json!({}));
        bus.emit("build.done", json!({}));

        let result = bus.poll_topic("test.pass", 0);
        assert!(result.events.is_empty());
        assert!(!result.gap_detected);
    }

    #[test]
    fn capacity_one_ring_buffer() {
        let mut bus = EventBus::with_capacity(1);

        bus.emit("a", json!(0)); // seq 0
        assert_eq!(bus.len(), 1);
        assert_eq!(bus.oldest_seq(), Some(0));

        bus.emit("b", json!(1)); // seq 1, evicts seq 0
        assert_eq!(bus.len(), 1);
        assert_eq!(bus.oldest_seq(), Some(1));
        assert_eq!(bus.all()[0].topic, "b");
        assert_eq!(bus.next_seq(), 2);

        // Gap: cursor 0, oldest is 1 → since+1=1, oldest=1, no gap
        let result = bus.poll(0);
        assert!(!result.gap_detected);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn default_uses_default_capacity() {
        let bus = EventBus::default();
        assert!(bus.is_empty());
        assert_eq!(bus.capacity, DEFAULT_CAPACITY);
    }

    #[test]
    fn large_burst_overflow_sequence_continuity() {
        let mut bus = EventBus::with_capacity(5);

        // Emit 20 events — buffer holds only last 5
        for i in 0..20u64 {
            let seq = bus.emit(format!("e{i}"), json!(i));
            assert_eq!(seq, i);
        }

        assert_eq!(bus.len(), 5);
        assert_eq!(bus.next_seq(), 20);
        assert_eq!(bus.oldest_seq(), Some(15));

        // All 5 remaining events have correct sequences
        let all = bus.all();
        assert_eq!(all.len(), 5);
        for (i, event) in all.iter().enumerate() {
            assert_eq!(event.seq, 15 + i as u64);
        }

        // Gap from cursor 0: oldest=15, since+1=1, lost=14
        let result = bus.poll(0);
        assert!(result.gap_detected);
        assert_eq!(result.events_lost, 14);
        assert_eq!(result.events.len(), 5);
    }
}
