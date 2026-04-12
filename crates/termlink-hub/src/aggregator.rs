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

                match result {
                    Ok(Ok(resp)) => {
                        if let Ok(data) = client::unwrap_result(resp) {
                            if let Some(events) = data["events"].as_array() {
                                for event in events {
                                    let agg = AggregatedEvent {
                                        session_id: target.id.clone(),
                                        session_name: target.display_name.clone(),
                                        seq: event["seq"].as_u64().unwrap_or(0),
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
                            if let Some(next) = data["next_seq"].as_u64() {
                                cursor = next;
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::debug!(
                            session = %target.id,
                            error = %e,
                            "Aggregator: session unreachable, retrying"
                        );
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    Err(_) => {
                        // Timeout — normal for idle sessions, just retry
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
                    if let Some(filter) = topic_filter {
                        if event.topic != filter {
                            continue;
                        }
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
