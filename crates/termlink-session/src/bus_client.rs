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

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use termlink_protocol::control::method;
use termlink_protocol::jsonrpc::RpcResponse;

use crate::client::{rpc_call, ClientError};
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
}

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

/// Offline-tolerant client for `channel.*` RPCs.
pub struct BusClient {
    socket_path: PathBuf,
    queue: Arc<OfflineQueue>,
    shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
}

impl BusClient {
    /// Open the queue at `queue_path`, spawn the flush task, and return
    /// both the client (wrapped in `Arc` so the flush task can hold it)
    /// and the task's `JoinHandle`. Dropping the returned `Arc<BusClient>`
    /// notifies the task to exit on its next tick.
    pub fn connect(
        socket_path: PathBuf,
        queue_path: impl AsRef<Path>,
    ) -> Result<(Arc<Self>, JoinHandle<()>), BusClientError> {
        Self::connect_with_interval(socket_path, queue_path, DEFAULT_FLUSH_INTERVAL)
    }

    /// Same as `connect` but with a configurable flush cadence (tests use
    /// a short interval to drive the queue quickly).
    pub fn connect_with_interval(
        socket_path: PathBuf,
        queue_path: impl AsRef<Path>,
        flush_interval: Duration,
    ) -> Result<(Arc<Self>, JoinHandle<()>), BusClientError> {
        let queue = Arc::new(OfflineQueue::open(queue_path)?);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let client = Arc::new(Self {
            socket_path,
            queue,
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
        });
        let handle = {
            let weak = Arc::downgrade(&client);
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        // Recv resolves immediately when the sender is dropped.
                        _ = &mut shutdown_rx => break,
                        _ = tokio::time::sleep(flush_interval) => {
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
        match rpc_call(&self.socket_path, method::CHANNEL_POST, params).await {
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
    /// is preserved (the failing entry remains at head).
    pub async fn flush(&self) -> FlushReport {
        let mut report = FlushReport::default();
        loop {
            let Ok(Some((id, post))) = self.queue.peek_oldest() else {
                break;
            };
            let params = post_to_params(&post);
            match rpc_call(&self.socket_path, method::CHANNEL_POST, params).await {
                Ok(resp) => match parse_post_response(resp) {
                    Ok(_offset) => {
                        let _ = self.queue.pop(id);
                        report.sent += 1;
                    }
                    Err(e) => {
                        // Hub answered but rejected — not a transport problem.
                        // Bump attempts and break so we don't busy-loop on a poison message.
                        tracing::warn!(queue_id = id.0, error = %e, "flush: hub rejected post");
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
            nonexistent_socket,
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
    async fn flush_with_hub_down_leaves_queue_intact() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("nope.sock");
        let queue_path = dir.path().join("outbound.sqlite");
        let (client, handle) = BusClient::connect_with_interval(
            socket,
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

    #[tokio::test]
    async fn drop_notifies_flush_task() {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join("nope.sock");
        let queue_path = dir.path().join("outbound.sqlite");
        let (client, handle) = BusClient::connect_with_interval(
            socket,
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
