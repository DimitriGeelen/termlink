//! Hub-level event aggregator (T-966).
//!
//! Maintains persistent subscriptions to session event buses and republishes
//! events into a single broadcast channel. Consumers call `subscribe()` once
//! instead of fanning out N RPCs.
//!
//! Lifecycle:
//!   - `add_session()` spawns a background task that long-polls `event.subscribe`
//!   - `remove_session()` aborts the background task
//!   - `subscribe()` returns a broadcast receiver for aggregated events

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;

use termlink_protocol::control;
use termlink_protocol::TransportAddr;
use termlink_session::client;

/// An event enriched with session metadata.
#[derive(Clone, Debug, serde::Serialize)]
pub struct AggregatedEvent {
    pub session_id: String,
    pub session_name: String,
    pub seq: u64,
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: u64,
}

/// Session connection info for the aggregator.
#[derive(Clone, Debug)]
pub struct SessionTarget {
    pub id: String,
    pub display_name: String,
    pub addr: TransportAddr,
}

/// Hub-level event aggregator.
pub struct EventAggregator {
    tx: broadcast::Sender<AggregatedEvent>,
    tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl EventAggregator {
    /// Create a new aggregator with the given broadcast channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to the aggregated event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<AggregatedEvent> {
        self.tx.subscribe()
    }

    /// Number of active session subscriptions.
    pub async fn session_count(&self) -> usize {
        self.tasks.read().await.len()
    }

    pub fn inject(&self, event: AggregatedEvent) { let _ = self.tx.send(event); } // T-1636
    /// Add a session subscription. Spawns a background long-poll loop.
    pub async fn add_session(&self, target: SessionTarget) {
        let sid = target.id.clone();

        // Remove existing subscription if any
        self.remove_session(&sid).await;

        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let mut cursor: u64 = 0;

            loop {
                let params = json!({
                    "timeout_ms": 5000,
                    "since": cursor,
                    "max_events": 100,
                });

                let result = tokio::time::timeout(
                    Duration::from_secs(10),
                    client::rpc_call_addr(&target.addr, control::method::EVENT_SUBSCRIBE, params),
                )
                .await;

                // T-2496: reduce the layered timeout/transport/rpc result into
                // one PollOutcome, then dispatch on the pure `poll_action`
                // classifier. This closes the silent hot-loop: an in-band
                // JSON-RPC error (transport OK, hub returned an error) now backs
                // off like a transport error instead of falling through a bare
                // `if let Ok(..)` with no sleep and re-spinning immediately.
                let outcome = match result {
                    Ok(Ok(resp)) => match client::unwrap_result(resp) {
                        Ok(data) => PollOutcome::Success(data),
                        Err(e) => PollOutcome::RpcError(e),
                    },
                    Ok(Err(e)) => PollOutcome::Transport(e.to_string()),
                    Err(_) => PollOutcome::Idle,
                };

                match poll_action(&outcome) {
                    PollAction::Deliver => {
                        if let PollOutcome::Success(data) = &outcome {
                            let mut delivered_max_seq: Option<u64> = None;
                            if let Some(events) = data["events"].as_array() {
                                for event in events {
                                    let seq = event["seq"].as_u64();
                                    if let Some(s) = seq {
                                        delivered_max_seq =
                                            Some(delivered_max_seq.map_or(s, |m| m.max(s)));
                                    }
                                    let agg = AggregatedEvent {
                                        session_id: target.id.clone(),
                                        session_name: target.display_name.clone(),
                                        seq: seq.unwrap_or(0),
                                        topic: event["topic"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string(),
                                        payload: event["payload"].clone(),
                                        timestamp: event["timestamp"].as_u64().unwrap_or(0),
                                    };
                                    // Best-effort send — if no subscribers, discard
                                    let _ = tx.send(agg);
                                }
                            }
                            // T-2503: advance the cursor via the pure `next_cursor`
                            // helper. A bare `if let Some(next) = ...next_seq` with
                            // no else silently stalled the cursor when `next_seq`
                            // was missing-despite-events → the same batch was
                            // re-fetched + re-broadcast every poll (silent duplicate
                            // storm). The fallback advances past what we delivered.
                            let (new_cursor, used_fallback) =
                                next_cursor(data, delivered_max_seq, cursor);
                            if used_fallback {
                                tracing::warn!(
                                    session = %target.id,
                                    from_cursor = cursor,
                                    to_cursor = new_cursor,
                                    "aggregator: session poll returned events but no valid next_seq; \
                                     advancing cursor to max(delivered_seq)+1 to avoid a duplicate \
                                     re-send storm (a healthy session always stamps next_seq)"
                                );
                            }
                            cursor = new_cursor;
                        }
                    }
                    PollAction::Backoff => {
                        let reason = match &outcome {
                            PollOutcome::RpcError(e) => format!("session returned RPC error: {e}"),
                            PollOutcome::Transport(e) => format!("session unreachable: {e}"),
                            _ => String::new(),
                        };
                        tracing::debug!(
                            session = %target.id,
                            reason = %reason,
                            "Aggregator: backing off before retry"
                        );
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    PollAction::IdleRetry => {
                        // Outer timeout — normal for idle sessions, just retry.
                        tracing::trace!(session = %target.id, "Aggregator: subscribe timeout (idle)");
                    }
                }
            }
        });

        self.tasks.write().await.insert(sid.clone(), handle);
        tracing::info!(session = %sid, "Aggregator: subscribed");
    }

    /// Remove a session subscription (aborts the background task).
    pub async fn remove_session(&self, session_id: &str) {
        if let Some(handle) = self.tasks.write().await.remove(session_id) {
            handle.abort();
            tracing::info!(session = %session_id, "Aggregator: unsubscribed");
        }
    }

    /// Collect events with a timeout (convenience for event.collect backward compat).
    /// Returns events received within the timeout window.
    pub async fn collect(
        &self,
        timeout: Duration,
        topic_filter: Option<&str>,
    ) -> Vec<AggregatedEvent> {
        let mut rx = self.subscribe();
        let mut events = Vec::new();

        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                break;
            }

            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Ok(event)) => {
                    if topic_filter.is_some_and(|f| event.topic != f) {
                        continue;
                    }
                    events.push(event);
                }
                Ok(Err(broadcast::error::RecvError::Lagged(n))) => {
                    tracing::warn!(lost = n, "Aggregator subscriber lagged");
                }
                Ok(Err(_)) => break, // channel closed
                Err(_) => break,     // timeout
            }
        }

        events
    }
}

impl Default for EventAggregator {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// T-2496: reduced outcome of one long-poll `event.subscribe` attempt, with the
/// raw timeout/transport/rpc layers collapsed into one value. Carries the
/// payload on success and the error text on failure (for the backoff log).
enum PollOutcome {
    /// Transport OK and the hub returned a Success result.
    Success(serde_json::Value),
    /// Transport OK but the hub returned an in-band JSON-RPC error. This
    /// returns immediately (no server-side long-poll wait), so it MUST back
    /// off — otherwise the loop hot-spins hammering the failing session.
    RpcError(String),
    /// Client-layer transport failure (connection refused, reset, etc.).
    Transport(String),
    /// The outer 10s timeout elapsed — a normal idle long-poll expiry.
    Idle,
}

/// T-2496: the loop's action for one attempt. Factored out so the
/// "in-band RPC error backs off, not hot-loops" rule is unit-testable
/// without a live hub.
#[derive(Debug, PartialEq, Eq)]
enum PollAction {
    /// Deliver the payload's events and advance the cursor.
    Deliver,
    /// A failure (transport OR in-band RPC error) — log + sleep before retry.
    Backoff,
    /// Idle long-poll expiry — retry immediately (no backoff needed).
    IdleRetry,
}

/// Pure classifier: both failure kinds (transport AND in-band RPC error) back
/// off; only the outer idle-timeout retries immediately. This is the single
/// source of truth the long-poll loop dispatches on.
fn poll_action(outcome: &PollOutcome) -> PollAction {
    match outcome {
        PollOutcome::Success(_) => PollAction::Deliver,
        PollOutcome::RpcError(_) | PollOutcome::Transport(_) => PollAction::Backoff,
        PollOutcome::Idle => PollAction::IdleRetry,
    }
}

/// T-2503: pure cursor-advance policy for a delivered poll batch. Factored out so
/// the "never stall the cursor when events were delivered" rule is unit-testable
/// without a live session.
///
/// Prefer the server-provided `next_seq`. If it is absent or non-integer BUT the
/// batch delivered at least one event (`delivered_max_seq` is `Some`), fall back
/// to `max(delivered_seq) + 1` so the loop advances past what it just delivered —
/// otherwise the cursor stalls and the identical batch is re-fetched and
/// re-broadcast every poll (a silent duplicate-event storm). With no `next_seq`
/// and no delivered events there is nothing to advance past, so the cursor is
/// unchanged. The returned bool is `true` iff the fallback was used (the caller
/// warns — a healthy server always stamps `next_seq`).
fn next_cursor(
    data: &serde_json::Value,
    delivered_max_seq: Option<u64>,
    current: u64,
) -> (u64, bool) {
    if let Some(next) = data["next_seq"].as_u64() {
        (next, false)
    } else if let Some(max_seq) = delivered_max_seq {
        (max_seq.saturating_add(1), true)
    } else {
        (current, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T-2496 regression: an in-band JSON-RPC error MUST back off, not hot-loop.
    /// This is the exact defect — before the fix the RPC-error branch fell
    /// through a bare `if let Ok(..)` with no sleep and re-spun immediately.
    #[test]
    fn poll_action_backs_off_on_in_band_rpc_error() {
        assert_eq!(
            poll_action(&PollOutcome::RpcError("hub rejected subscribe".into())),
            PollAction::Backoff
        );
    }

    #[test]
    fn poll_action_backs_off_on_transport_error() {
        assert_eq!(
            poll_action(&PollOutcome::Transport("connection refused".into())),
            PollAction::Backoff
        );
    }

    #[test]
    fn poll_action_delivers_on_success() {
        assert_eq!(
            poll_action(&PollOutcome::Success(serde_json::json!({"events": []}))),
            PollAction::Deliver
        );
    }

    #[test]
    fn poll_action_idle_retry_on_outer_timeout() {
        assert_eq!(poll_action(&PollOutcome::Idle), PollAction::IdleRetry);
    }

    // T-2503: next_cursor must never stall the cursor when events were delivered.

    #[test]
    fn next_cursor_uses_server_next_seq_when_present() {
        let data = serde_json::json!({"next_seq": 42});
        // Even with delivered events, an explicit next_seq wins verbatim.
        assert_eq!(next_cursor(&data, Some(10), 5), (42, false));
    }

    #[test]
    fn next_cursor_falls_back_to_max_seq_plus_one_when_next_seq_absent() {
        // The storm case: events delivered (max seq 10) but no next_seq. Must
        // advance to 11 (not stall at the current cursor) and flag the fallback.
        let data = serde_json::json!({"events": [{"seq": 10}]});
        assert_eq!(next_cursor(&data, Some(10), 5), (11, true));
    }

    #[test]
    fn next_cursor_unchanged_when_no_next_seq_and_no_events() {
        // Nothing delivered and no next_seq → nothing to advance past.
        let data = serde_json::json!({"events": []});
        assert_eq!(next_cursor(&data, None, 7), (7, false));
    }

    #[test]
    fn next_cursor_treats_non_integer_next_seq_as_absent() {
        // A malformed (non-integer) next_seq must not be trusted — fall back.
        let data = serde_json::json!({"next_seq": "not-a-number"});
        assert_eq!(next_cursor(&data, Some(3), 1), (4, true));
    }
}
