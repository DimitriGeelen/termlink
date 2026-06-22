//! Client abstraction for the T-1155 channel bus (T-1161).
//!
//! Wraps `client::rpc_call` with an offline-tolerant envelope:
//!
//! - `BusClient::post` attempts a direct `channel.post` RPC. On transport
//!   failure (hub unreachable) it enqueues the post into the local
//!   `OfflineQueue` and returns `PostOutcome::Queued`.
//! - A background flush task (spawned by `BusClient::connect`) drains the
//!   queue every `flush_interval` (default 5s) once the hub comes back.
//! - The flush task is cancel-safe: dropping the `BusClient` notifies the
//!   task, which exits on its next tick.
//!
//! The client is deliberately thin — signing lives in `termlink-session`
//! `agent_identity`; canonical bytes live in `termlink_protocol::control::channel`.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use termlink_protocol::control::method;
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_protocol::transport::TransportAddr;

use crate::client::{rpc_call_addr, ClientError};
use crate::offline_queue::{OfflineQueue, PendingPost, QueueError};

/// Result of a `BusClient::post` attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostOutcome {
    /// The hub accepted the post. `offset` is the per-topic log offset.
    Delivered { offset: i64 },
    /// The hub was unreachable; post was enqueued for later flush.
    Queued { queue_id: i64 },
}

/// Summary of one flush pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlushReport {
    pub sent: u64,
    pub failed: u64,
    /// Entries dropped as poison after `POISON_THRESHOLD` hub-reject attempts (T-1439).
    pub dropped_poison: u64,
}

/// T-1439: After this many hub-reject responses on the same head-of-queue
/// entry, the entry is treated as poison and popped instead of head-blocking
/// the rest of the queue forever. Permanent errors (unknown topic, malformed
/// payload, signature mismatch) are the typical poison source — transient
/// hub-side issues clear well below this threshold.
pub const POISON_THRESHOLD: u64 = 10;

#[derive(Debug, thiserror::Error)]
pub enum BusClientError {
    #[error("queue: {0}")]
    Queue(#[from] QueueError),

    #[error("rpc: {0}")]
    Rpc(#[from] ClientError),

    #[error("hub returned error: {0}")]
    HubError(String),

    #[error("hub response malformed: {0}")]
    Malformed(String),
}

/// Default flush cadence for the background task.
pub const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_secs(5);

/// T-2055: ±25% jitter on per-tick sleep to desynchronise fleet-wide flush
/// pulses after a hub bounce. Without it, every spoke that queued during
/// the outage wakes on the same tick boundary and slams the hub on return,
/// defeating T-2048's RATE_LIMITED budget. Pure helper so unit tests can
/// drive a seeded RNG; production callers pass `rand::thread_rng()`.
pub fn jittered_interval(base: Duration, rng: &mut impl rand::Rng) -> Duration {
    let span_ms = (base.as_millis() as f64 * 0.25) as i64;
    if span_ms <= 0 {
        return base;
    }
    let delta_ms = rng.gen_range(-span_ms..=span_ms);
    if delta_ms >= 0 {
        base.saturating_add(Duration::from_millis(delta_ms as u64))
    } else {
        base.saturating_sub(Duration::from_millis(delta_ms.unsigned_abs()))
    }
}

/// Offline-tolerant client for `channel.*` RPCs.
pub struct BusClient {
    addr: TransportAddr,
    queue: Arc<OfflineQueue>,
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl BusClient {
    /// Open the queue at `queue_path`, spawn the flush task, and return
    /// both the client (wrapped in `Arc` so the flush task can hold it)
    /// and the task's `JoinHandle`. Dropping the returned `Arc<BusClient>`
    /// notifies the task to exit on its next tick. T-1385: accepts
    /// `TransportAddr` for TCP cross-hub posting.
    pub fn connect(
        addr: TransportAddr,
        queue_path: impl AsRef<Path>,
    ) -> Result<(Arc<Self>, JoinHandle<()>), BusClientError> {
        Self::connect_with_interval(addr, queue_path, DEFAULT_FLUSH_INTERVAL)
    }

    /// Same as `connect` but with a configurable flush cadence (tests use
    /// a short interval to drive the queue quickly).
    pub fn connect_with_interval(
        addr: TransportAddr,
        queue_path: impl AsRef<Path>,
        flush_interval: Duration,
    ) -> Result<(Arc<Self>, JoinHandle<()>), BusClientError> {
        let queue = Arc::new(OfflineQueue::open(queue_path)?);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let client = Arc::new(Self {
            addr,
            queue,
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
        });
        let handle = {
            let weak = Arc::downgrade(&client);
            tokio::spawn(async move {
                loop {
                    // T-2055: jitter the per-tick sleep so fleet-wide bounces
                    // don't produce simultaneous flush pulses against the hub.
                    let tick = jittered_interval(flush_interval, &mut rand::thread_rng());
                    tokio::select! {
                        // Recv resolves immediately when the sender is dropped.
                        _ = &mut shutdown_rx => break,
                        _ = tokio::time::sleep(tick) => {
                            let Some(c) = weak.upgrade() else { break; };
                            let _ = c.flush().await;
                        }
                    }
                }
            })
        };
        Ok((client, handle))
    }

    /// Size of the pending queue.
    pub fn queue_size(&self) -> u64 {
        self.queue.size().unwrap_or(0)
    }

    /// Notify the background flush task to exit. Usually unnecessary
    /// (dropping the client triggers the same path via the oneshot), but
    /// useful when the caller wants to `.await` the `JoinHandle` without
    /// dropping the `Arc`.
    pub fn shutdown(&self) {
        if let Some(tx) = self.shutdown_tx.lock().expect("shutdown lock").take() {
            let _ = tx.send(());
        }
    }

    /// Try to POST directly; on transport failure, enqueue and return `Queued`.
    pub async fn post(&self, post: PendingPost) -> Result<PostOutcome, BusClientError> {
        let params = post_to_params(&post);
        match rpc_call_addr(&self.addr, method::CHANNEL_POST, params).await {
            Ok(resp) => parse_post_response(resp).map(|offset| PostOutcome::Delivered { offset }),
            Err(e) => {
                // Any transport / protocol-level failure → queue locally.
                tracing::debug!(error = %e, "channel.post failed — enqueueing to offline queue");
                let id = self.queue.enqueue(&post)?;
                Ok(PostOutcome::Queued { queue_id: id.0 })
            }
        }
    }

    /// Drain the queue. Stops at the first transport failure so FIFO order
    /// is preserved (the failing entry remains at head). Hub-reject (post
    /// hit the hub but was rejected) bumps attempts; once `POISON_THRESHOLD`
    /// is crossed the entry is dropped so it cannot head-block subsequent
    /// posts (T-1439).
    pub async fn flush(&self) -> FlushReport {
        let mut report = FlushReport::default();
        loop {
            let Ok(Some((id, post, attempts))) = self.queue.peek_oldest_with_attempts() else {
                break;
            };
            let params = post_to_params(&post);
            match rpc_call_addr(&self.addr, method::CHANNEL_POST, params).await {
                Ok(resp) => match parse_post_response(resp) {
                    Ok(_offset) => {
                        let _ = self.queue.pop(id);
                        report.sent += 1;
                    }
                    Err(e) => {
                        // Hub answered but rejected — not a transport problem.
                        // T-1439: once attempts crosses POISON_THRESHOLD the
                        // entry is treated as permanent-error poison and popped
                        // so subsequent entries get a chance. Below threshold,
                        // bump and break (preserves the no-busy-loop behavior
                        // for transient errors / restart races).
                        if attempts + 1 >= POISON_THRESHOLD {
                            // T-2243 (R4): MOVE the poison post into the durable
                            // dead-letter store instead of the old bare `pop()`
                            // that silently lost it. A governance-plane "complete"
                            // rejected during a hub blip now surfaces in
                            // `queue-status`, recoverable, rather than vanishing
                            // with only this trace.
                            let reason = format!(
                                "hub rejected after {} attempts: {e}",
                                attempts + 1
                            );
                            tracing::warn!(
                                queue_id = id.0,
                                attempts = attempts + 1,
                                topic = %post.topic,
                                msg_type = %post.msg_type,
                                error = %e,
                                "flush: dead-lettering poison post after {POISON_THRESHOLD} hub-reject attempts"
                            );
                            if let Err(de) = self.queue.dead_letter(id, &reason) {
                                // Dead-letter write itself failed (disk/SQLite
                                // broken). Fall back to a drop so a single poison
                                // entry can't head-block the whole queue forever,
                                // but make the loss LOUD (error, not the silent
                                // debug the old path used).
                                tracing::error!(
                                    queue_id = id.0,
                                    error = %de,
                                    "flush: dead-letter write failed; dropping poison post to avoid head-of-line block"
                                );
                                let _ = self.queue.pop(id);
                            }
                            report.dropped_poison += 1;
                            // Continue draining — don't let the poison
                            // permanently block subsequent entries.
                            continue;
                        }
                        tracing::warn!(
                            queue_id = id.0,
                            attempts = attempts + 1,
                            error = %e,
                            "flush: hub rejected post (will retry)"
                        );
                        let _ = self.queue.bump_attempts(id);
                        report.failed += 1;
                        break;
                    }
                },
                Err(e) => {
                    tracing::debug!(queue_id = id.0, error = %e, "flush: transport error, will retry");
                    report.failed += 1;
                    break;
                }
            }
        }
        report
    }
}

impl Drop for BusClient {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.lock().expect("shutdown lock").take() {
            let _ = tx.send(());
        }
    }
}

fn post_to_params(p: &PendingPost) -> Value {
    use base64::Engine as _;
    let payload_b64 = base64::engine::general_purpose::STANDARD.encode(&p.payload);
    let mut params = json!({
        "topic": p.topic,
        "msg_type": p.msg_type,
        "payload_b64": payload_b64,
        "artifact_ref": p.artifact_ref,
        "ts": p.ts_unix_ms,
        "sender_id": p.sender_id,
        "sender_pubkey_hex": p.sender_pubkey_hex,
        "signature_hex": p.signature_hex,
    });
    // T-1313: forward metadata only when populated. Hub treats it as routing
    // hint (NOT signed) — well-known keys: conversation_id, event_type,
    // in_reply_to. Empty map omits the field for wire-shape stability.
    if !p.metadata.is_empty()
        && let Some(obj) = params.as_object_mut()
    {
        obj.insert(
            "metadata".to_string(),
            serde_json::to_value(&p.metadata).unwrap_or(Value::Null),
        );
    }
    // T-2049 Gap A: forward client_msg_id when present. Omitted otherwise
    // for wire-shape stability with pre-T-2049 callers.
    if let Some(ref cid) = p.client_msg_id
        && let Some(obj) = params.as_object_mut()
    {
        obj.insert("client_msg_id".to_string(), Value::String(cid.clone()));
    }
    params
}

fn parse_post_response(resp: RpcResponse) -> Result<i64, BusClientError> {
    match resp {
        RpcResponse::Success(ok) => ok
            .result
            .get("offset")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| BusClientError::Malformed("missing offset".into())),
        RpcResponse::Error(e) => Err(BusClientError::HubError(format!(
            "code={} message={}",
            e.error.code, e.error.message
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_post(topic: &str) -> PendingPost {
        PendingPost {
            topic: topic.to_string(),
            msg_type: "chat".to_string(),
            payload: b"hi".to_vec(),
            artifact_ref: None,
            ts_unix_ms: 1,
            sender_id: "s".into(),
            sender_pubkey_hex: "00".repeat(32),
            signature_hex: "00".repeat(64),
            metadata: Default::default(),
            client_msg_id: None,
        }
    }

    #[test]
    fn post_to_params_omits_metadata_when_empty() {
        // T-1313: empty metadata → wire shape unchanged from pre-T-1313 senders.
        let p = sample_post("t");
        let v = post_to_params(&p);
        assert!(v.get("metadata").is_none(), "metadata must be omitted when empty");
    }

    #[test]
    fn post_to_params_includes_metadata_when_populated() {
        // T-1313: in_reply_to surfaces in params for hub-side routing/filter.
        let mut p = sample_post("t");
        p.metadata.insert("in_reply_to".into(), "42".into());
        p.metadata.insert("conversation_id".into(), "c-A".into());
        let v = post_to_params(&p);
        let m = v.get("metadata").and_then(|v| v.as_object()).expect("metadata present");
        assert_eq!(m.get("in_reply_to").and_then(|v| v.as_str()), Some("42"));
        assert_eq!(m.get("conversation_id").and_then(|v| v.as_str()), Some("c-A"));
    }

    #[tokio::test]
    async fn post_queues_when_hub_unreachable() {
        let dir = tempfile::tempdir().unwrap();
        let nonexistent_socket = dir.path().join("nope.sock");
        let queue_path = dir.path().join("outbound.sqlite");
        let (client, handle) = BusClient::connect_with_interval(
            TransportAddr::unix(nonexistent_socket),
            &queue_path,
            Duration::from_secs(3600), // don't auto-flush during this test
        )
        .unwrap();

        let out = client.post(sample_post("t1")).await.unwrap();
        assert!(matches!(out, PostOutcome::Queued { .. }));
        assert_eq!(client.queue_size(), 1);

        drop(client);
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    #[tokio::test]
    async fn flush_poison_dead_letters_instead_of_silent_drop() {
        // T-2243 (R4): end-to-end proof that the flush loop's poison-drop
        // now lands in the durable dead-letter store, not a bare DELETE.
        // We point the BusClient at the in-process session server, which
        // answers `channel.post` with a JSON-RPC error (-32601 method-not-
        // found) — i.e. a hub-REJECT (not a transport failure). Each flush
        // bumps the head entry's attempt count; once POISON_THRESHOLD is
        // crossed the entry must be dead-lettered, recoverable.
        use crate::handler::SessionContext;
        use crate::registration::{Registration, SessionConfig};
        use crate::server;
        use crate::{SessionId, SessionState};
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use tokio::sync::RwLock;

        static C: AtomicU32 = AtomicU32::new(0);
        let n = C.fetch_add(1, Ordering::Relaxed);
        let socket_path =
            std::path::PathBuf::from(format!("/tmp/tl-busdl-{}-{}.sock", std::process::id(), n));
        let _ = std::fs::remove_file(&socket_path);

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();
        let id = SessionId::generate();
        let mut reg = Registration::new(id, SessionConfig::default(), socket_path.clone());
        reg.state = SessionState::Ready;
        let shared = Arc::new(RwLock::new(SessionContext::new(reg)));
        let shared_clone = shared.clone();
        let server_handle =
            tokio::spawn(async move { server::run_accept_loop(listener, shared_clone).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let dir = tempfile::tempdir().unwrap();
        let queue_path = dir.path().join("outbound.sqlite");

        // Seed the queue directly via a separate handle to the same file.
        // (We can't use `client.post()` to seed: the hub ANSWERS with an
        // error, so post() surfaces the reject rather than queuing — queuing
        // only happens on a transport failure. The poison path we're testing
        // is reached by the flush loop replaying an already-queued entry.)
        {
            let seed = crate::offline_queue::OfflineQueue::open(&queue_path).unwrap();
            seed.enqueue(&sample_post("nonexistent-topic")).unwrap();
        }

        let (client, flush_handle) = BusClient::connect_with_interval(
            TransportAddr::unix(socket_path.clone()),
            &queue_path,
            Duration::from_secs(3600), // we drive flush() manually
        )
        .unwrap();
        assert_eq!(client.queue_size(), 1);

        // Drive the flush loop POISON_THRESHOLD times. Each call gets a
        // hub-reject; the last one crosses the threshold.
        let mut dropped = 0;
        for _ in 0..POISON_THRESHOLD {
            let r = client.flush().await;
            dropped += r.dropped_poison;
        }
        assert_eq!(dropped, 1, "exactly one poison post dead-lettered");
        assert_eq!(client.queue_size(), 0, "poison no longer head-blocks the queue");

        // The crux: it was MOVED, not silently dropped.
        let q = crate::offline_queue::OfflineQueue::open(&queue_path).unwrap();
        assert_eq!(
            q.dead_letter_count().unwrap(),
            1,
            "poison post is recoverable in the dead-letter store — zero silent loss"
        );
        let rows = q.list_dead_letters(10).unwrap();
        assert_eq!(rows[0].post.topic, "nonexistent-topic");
        assert!(rows[0].attempts >= POISON_THRESHOLD - 1);
        assert!(!rows[0].reason.is_empty(), "reject reason recorded");

        drop(client);
        let _ = tokio::time::timeout(Duration::from_secs(2), flush_handle).await;
        server_handle.abort();
        let _ = std::fs::remove_file(&socket_path);
    }

    #[tokio::test]
    async fn flush_with_hub_down_leaves_queue_intact() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("nope.sock");
        let queue_path = dir.path().join("outbound.sqlite");
        let (client, handle) = BusClient::connect_with_interval(
            TransportAddr::unix(socket),
            &queue_path,
            Duration::from_secs(3600),
        )
        .unwrap();

        for _ in 0..3 {
            let _ = client.post(sample_post("t")).await.unwrap();
        }
        assert_eq!(client.queue_size(), 3);

        let r = client.flush().await;
        assert_eq!(r.sent, 0);
        assert_eq!(r.failed, 1); // breaks at first failure
        assert_eq!(client.queue_size(), 3);

        drop(client);
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    }

    #[test]
    fn jittered_interval_stays_within_25pct_band() {
        // T-2055: every sample must fall within [3750ms, 6250ms] for a 5s base.
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(0xC0FFEE);
        let base = Duration::from_secs(5);
        let lo = Duration::from_millis(3750);
        let hi = Duration::from_millis(6250);
        for _ in 0..100 {
            let d = jittered_interval(base, &mut rng);
            assert!(d >= lo && d <= hi, "jittered {d:?} outside [{lo:?},{hi:?}]");
        }
    }

    #[test]
    fn jittered_interval_actually_varies_across_samples() {
        // T-2055: confirm we're not accidentally returning the base every time
        // (e.g. via an off-by-one that collapses span_ms to 0). Sample span
        // must cover ≥50% of the ±25% band over 100 draws.
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let base = Duration::from_secs(5);
        let samples: Vec<u128> = (0..100)
            .map(|_| jittered_interval(base, &mut rng).as_millis())
            .collect();
        let min = *samples.iter().min().unwrap();
        let max = *samples.iter().max().unwrap();
        let span = max - min;
        // Full band is 2500ms (±25% of 5000ms). Require ≥50% coverage = 1250ms.
        assert!(span >= 1250, "jitter range only {span}ms — RNG not driving variance");
    }

    #[test]
    fn jittered_interval_handles_tiny_base_safely() {
        // T-2055: span_ms collapses to 0 for sub-4ms bases; helper must return
        // base unchanged rather than gen_range(0..=0) panicking.
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        let base = Duration::from_millis(2);
        let d = jittered_interval(base, &mut rng);
        assert_eq!(d, base);
    }

    #[tokio::test]
    async fn drop_notifies_flush_task() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("nope.sock");
        let queue_path = dir.path().join("outbound.sqlite");
        let (client, handle) = BusClient::connect_with_interval(
            TransportAddr::unix(socket),
            &queue_path,
            Duration::from_secs(3600),
        )
        .unwrap();
        drop(client);
        // Handle should exit promptly once shutdown fires.
        let r = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(r.is_ok(), "flush task did not exit after drop");
    }
}
