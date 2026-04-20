//! Integration tests for `BusClient` offline queue + flush (T-1161).
//!
//! These exercise the full transport path (Unix-socket JSON-RPC) against a
//! minimal in-test fake hub that answers `channel.post` by appending to a
//! shared counter. No dependency on `termlink-hub` — the contract under
//! test is the JSON-RPC wire format plus `BusClient` queue/flush behaviour.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use termlink_session::bus_client::{BusClient, PostOutcome};
use termlink_session::offline_queue::PendingPost;

struct FakeHub {
    received: Arc<Mutex<Vec<String>>>,
    abort_tx: tokio::sync::watch::Sender<bool>,
    handle: tokio::task::JoinHandle<()>,
}

impl FakeHub {
    async fn spawn(socket: PathBuf) -> Self {
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind fake hub");
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let (abort_tx, mut abort_rx) = tokio::sync::watch::channel(false);
        let received_clone = received.clone();
        let next_offset = Arc::new(std::sync::atomic::AtomicI64::new(0));
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = abort_rx.changed() => break,
                    accept = listener.accept() => {
                        let Ok((stream, _)) = accept else { break; };
                        let received = received_clone.clone();
                        let next_offset = next_offset.clone();
                        tokio::spawn(async move {
                            let (r, mut w) = tokio::io::split(stream);
                            let mut lines = BufReader::new(r).lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let req: serde_json::Value = match serde_json::from_str(&line) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
                                let id = req.get("id").cloned().unwrap_or(serde_json::json!(0));
                                if method == "channel.post" {
                                    let params = req.get("params").cloned().unwrap_or(serde_json::json!({}));
                                    let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let payload_b64 = params.get("payload_b64").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    {
                                        let mut r = received.lock().await;
                                        r.push(format!("{topic}|{payload_b64}"));
                                    }
                                    let offset = next_offset.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    let resp = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "result": { "offset": offset, "ts": 0 }
                                    });
                                    let mut out = serde_json::to_string(&resp).unwrap();
                                    out.push('\n');
                                    let _ = w.write_all(out.as_bytes()).await;
                                } else {
                                    let resp = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "error": { "code": -32601, "message": "method not found" }
                                    });
                                    let mut out = serde_json::to_string(&resp).unwrap();
                                    out.push('\n');
                                    let _ = w.write_all(out.as_bytes()).await;
                                }
                            }
                        });
                    }
                }
            }
        });
        Self { received, abort_tx, handle }
    }

    async fn stop(self) {
        let _ = self.abort_tx.send(true);
        let _ = tokio::time::timeout(Duration::from_secs(2), self.handle).await;
    }
}

fn sample(topic: &str, marker: u8) -> PendingPost {
    PendingPost {
        topic: topic.to_string(),
        msg_type: "chat".into(),
        payload: vec![marker],
        artifact_ref: None,
        ts_unix_ms: 0,
        sender_id: "test".into(),
        sender_pubkey_hex: "00".repeat(32),
        signature_hex: "00".repeat(64),
    }
}

#[tokio::test]
async fn post_deliver_queue_restart_drain() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("hub.sock");
    let queue_path = dir.path().join("outbound.sqlite");

    // Start fake hub.
    let hub = FakeHub::spawn(socket.clone()).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Short flush interval so queued messages drain quickly.
    let (client, handle) = BusClient::connect_with_interval(
        socket.clone(),
        &queue_path,
        Duration::from_millis(200),
    )
    .unwrap();

    // 10 direct posts while hub is up.
    for i in 0..10u8 {
        let out = client.post(sample("t", i)).await.expect("post ok");
        assert!(matches!(out, PostOutcome::Delivered { .. }), "expected delivered for {i}");
    }
    assert_eq!(client.queue_size(), 0);

    // Kill the hub. Remove socket so new connect() fails with transport error.
    hub.stop().await;
    let _ = std::fs::remove_file(&socket);

    // 5 posts with hub down — should queue.
    for i in 10..15u8 {
        let out = client.post(sample("t", i)).await.expect("post ok");
        assert!(matches!(out, PostOutcome::Queued { .. }), "expected queued for {i}");
    }
    assert_eq!(client.queue_size(), 5);

    // Restart hub on the same socket.
    let hub2 = FakeHub::spawn(socket.clone()).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Wait for the flush task to drain the queue (at most ~3s worth of ticks).
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while client.queue_size() > 0 && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(client.queue_size(), 0, "flush task should drain queued posts");

    // Verify all 15 envelopes reached the hub in order (10 direct + 5 flushed).
    let received = hub2.received.lock().await;
    // hub1 got 0..10, hub2 got 10..15 (after restart).
    // We only have hub2's counter here — hub1 saw the first 10.
    // So received for hub2 should have the 5 flushed entries as payload 10..14.
    assert_eq!(received.len(), 5, "hub2 should see the 5 flushed entries");
    for (i, line) in received.iter().enumerate() {
        let expected_marker = (10 + i) as u8;
        let expected_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            [expected_marker],
        );
        assert_eq!(line, &format!("t|{expected_b64}"), "order mismatch at {i}");
    }
    drop(received);

    // Clean shutdown.
    drop(client);
    let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
    hub2.stop().await;
}
