//! T-2031 — integration tests for the claim-client surface against a minimal
//! in-test JSON-RPC fake hub. Same pattern as `bus_client_integration.rs`:
//! no dependency on `termlink-hub`; tests the wire contract + LeasedClaim
//! ergonomics (auto-renew, Drop-fires-nack).
//!
//! The fake hub answers `channel.claim` / `channel.renew` / `channel.release`
//! with a simple in-memory state machine: one slot per (topic, offset) tuple,
//! ttl tracking for the renew test, and per-method call counters so tests can
//! assert "the background renew task fired N times" without timing flakiness.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use termlink_protocol::control::error_code;
use termlink_protocol::transport::TransportAddr;
use termlink_session::claim_client::{
    channel_claim, channel_release, ClaimError, LeasedClaim,
};

#[derive(Default)]
struct HubState {
    // (topic, offset) -> (claim_id, claimer, claimed_until_ms)
    slots: std::collections::HashMap<(String, u64), (String, String, i64)>,
    next_claim_seq: i64,
    now_ms: i64,
    claim_calls: u64,
    renew_calls: u64,
    release_calls: u64,
}

struct FakeHub {
    state: Arc<Mutex<HubState>>,
    abort_tx: tokio::sync::watch::Sender<bool>,
    handle: tokio::task::JoinHandle<()>,
}

impl FakeHub {
    async fn spawn(socket: PathBuf) -> Self {
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind fake hub");
        let state = Arc::new(Mutex::new(HubState {
            now_ms: 1_000,
            ..Default::default()
        }));
        let (abort_tx, mut abort_rx) = tokio::sync::watch::channel(false);
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = abort_rx.changed() => break,
                    accept = listener.accept() => {
                        let Ok((stream, _)) = accept else { break; };
                        let state = state_clone.clone();
                        tokio::spawn(async move {
                            let (r, mut w) = tokio::io::split(stream);
                            let mut lines = BufReader::new(r).lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let req: Value = match serde_json::from_str(&line) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
                                let id = req.get("id").cloned().unwrap_or(json!(0));
                                let params = req.get("params").cloned().unwrap_or(json!({}));
                                let resp = handle_call(method, id, params, state.clone()).await;
                                let mut out = serde_json::to_string(&resp).unwrap_or_default();
                                out.push('\n');
                                if w.write_all(out.as_bytes()).await.is_err() {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        });
        Self {
            state,
            abort_tx,
            handle,
        }
    }

    async fn stop(self) {
        let _ = self.abort_tx.send(true);
        let _ = tokio::time::timeout(Duration::from_secs(1), self.handle).await;
    }
}

async fn handle_call(method: &str, id: Value, params: Value, state: Arc<Mutex<HubState>>) -> Value {
    let mut s = state.lock().await;
    match method {
        "channel.claim" => {
            s.claim_calls += 1;
            let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ttl_ms = params.get("ttl_ms").and_then(|v| v.as_u64()).unwrap_or(30_000);
            let key = (topic.clone(), offset);
            if let Some((_existing_id, _existing_claimer, claimed_until)) = s.slots.get(&key)
                && *claimed_until > s.now_ms
            {
                return error_response(
                    id,
                    error_code::CLAIM_CONFLICT,
                    "taken",
                    Some(json!({"topic": topic, "offset": offset})),
                );
            }
            let seq = s.next_claim_seq;
            s.next_claim_seq += 1;
            let claim_id = format!("clm-{seq}-{topic}-{offset}");
            let claimed_at = s.now_ms;
            let claimed_until = s.now_ms + ttl_ms as i64;
            s.slots.insert(
                key,
                (claim_id.clone(), claimer.clone(), claimed_until),
            );
            success_response(id, json!({
                "ok": true,
                "claim_id": claim_id,
                "topic": topic,
                "offset": offset,
                "claimer": claimer,
                "claimed_at": claimed_at,
                "claimed_until": claimed_until,
            }))
        }
        "channel.renew" => {
            s.renew_calls += 1;
            let claim_id = params.get("claim_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let additional_ttl_ms = params
                .get("additional_ttl_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(30_000);
            // Advance virtual time by half-TTL on every renew so tests can
            // observe a moving claimed_until without sleeping.
            s.now_ms += additional_ttl_ms as i64 / 2;
            let now = s.now_ms;
            // Find slot owning this claim_id.
            let found = s
                .slots
                .iter()
                .find(|(_k, v)| v.0 == claim_id)
                .map(|(k, v)| (k.clone(), v.clone()));
            let Some((key, (cid, owner, until))) = found else {
                return error_response(
                    id,
                    error_code::CLAIM_NOT_FOUND,
                    "gone",
                    Some(json!({"claim_id": claim_id})),
                );
            };
            if until <= now {
                s.slots.remove(&key);
                return error_response(
                    id,
                    error_code::CLAIM_EXPIRED,
                    "expired",
                    Some(json!({"claim_id": cid})),
                );
            }
            if owner != claimer {
                return error_response(
                    id,
                    error_code::CLAIM_NOT_OWNED,
                    "not yours",
                    Some(json!({"claim_id": cid})),
                );
            }
            let new_until = now + additional_ttl_ms as i64;
            s.slots
                .insert(key.clone(), (cid.clone(), owner.clone(), new_until));
            success_response(id, json!({
                "ok": true,
                "claim_id": cid,
                "topic": key.0,
                "offset": key.1,
                "claimer": owner,
                "claimed_at": now,
                "claimed_until": new_until,
            }))
        }
        "channel.release" => {
            s.release_calls += 1;
            let claim_id = params.get("claim_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ack = params.get("ack").and_then(|v| v.as_bool()).unwrap_or(false);
            let found = s
                .slots
                .iter()
                .find(|(_k, v)| v.0 == claim_id)
                .map(|(k, v)| (k.clone(), v.0.clone(), v.1.clone()));
            let Some((key, cid, owner)) = found else {
                return error_response(
                    id,
                    error_code::CLAIM_NOT_FOUND,
                    "gone",
                    Some(json!({"claim_id": claim_id})),
                );
            };
            if owner != claimer {
                return error_response(
                    id,
                    error_code::CLAIM_NOT_OWNED,
                    "not yours",
                    Some(json!({"claim_id": cid})),
                );
            }
            s.slots.remove(&key);
            success_response(id, json!({
                "ok": true,
                "claim_id": cid,
                "topic": key.0,
                "offset": key.1,
                "ack": ack,
            }))
        }
        _ => error_response(id, -32601, "unknown method", None),
    }
}

fn success_response(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn error_response(id: Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut err = json!({"code": code, "message": message});
    if let Some(d) = data {
        err.as_object_mut().unwrap().insert("data".into(), d);
    }
    json!({"jsonrpc": "2.0", "id": id, "error": err})
}

fn test_socket(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tl-claim-it-{}-{}", std::process::id(), name));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("hub.sock")
}

#[tokio::test]
async fn claim_and_ack_round_trip() {
    let socket = test_socket("claim_ack");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let summary = channel_claim(&addr, "T", 0, "worker-A", 30_000)
        .await
        .expect("claim ok");
    assert_eq!(summary.topic, "T");
    assert_eq!(summary.offset, 0);
    assert_eq!(summary.claimer, "worker-A");
    assert_eq!(summary.claimed_until - summary.claimed_at, 30_000);

    let release =
        channel_release(&addr, &summary.claim_id, "worker-A", true)
            .await
            .expect("release ok");
    assert!(release.ack);
    assert_eq!(release.offset, 0);

    let s = hub.state.lock().await;
    assert_eq!(s.claim_calls, 1);
    assert_eq!(s.release_calls, 1);
    assert!(s.slots.is_empty());
    drop(s);
    hub.stop().await;
}

#[tokio::test]
async fn second_claim_of_same_offset_returns_conflict() {
    let socket = test_socket("conflict");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let _first = channel_claim(&addr, "T", 5, "worker-A", 30_000)
        .await
        .expect("first claim ok");
    let second = channel_claim(&addr, "T", 5, "worker-B", 30_000).await;
    match second {
        Err(ClaimError::Conflict { topic, offset }) => {
            assert_eq!(topic, "T");
            assert_eq!(offset, 5);
        }
        other => panic!("expected Conflict, got {other:?}"),
    }
    hub.stop().await;
}

#[tokio::test]
async fn leased_claim_auto_renews_past_original_ttl() {
    let socket = test_socket("auto_renew");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    // Use a short TTL so the renew task fires multiple times within ~500ms.
    let ttl_ms = 200_u32;
    let lease = LeasedClaim::acquire(addr.clone(), "T", 7, "worker-A", ttl_ms)
        .await
        .expect("acquire");
    let original_until = lease.claimed_until();
    // Renew cadence = ttl/2 = 100ms. Sleep ~450ms → at least 3 renews.
    tokio::time::sleep(Duration::from_millis(450)).await;
    let after_until = lease.claimed_until();
    assert!(
        after_until > original_until,
        "claimed_until should have advanced via auto-renew (was {original_until}, now {after_until})"
    );

    let renew_calls = hub.state.lock().await.renew_calls;
    assert!(
        renew_calls >= 3,
        "expected ≥3 renew calls, got {renew_calls}"
    );

    // Ack to consume cleanly.
    lease.ack().await.expect("ack");
    hub.stop().await;
}

#[tokio::test]
async fn dropping_leased_claim_fires_nack_release() {
    let socket = test_socket("drop_nack");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    {
        let _lease = LeasedClaim::acquire(addr.clone(), "T", 9, "worker-A", 30_000)
            .await
            .expect("acquire");
        // Drop the lease at end of scope without ack/nack.
    }
    // Give the fire-and-forget release time to land.
    tokio::time::sleep(Duration::from_millis(150)).await;
    let s = hub.state.lock().await;
    assert_eq!(s.release_calls, 1, "Drop should have fired one release");
    assert!(
        s.slots.is_empty(),
        "slot should be freed after Drop-released the claim"
    );
    drop(s);
    hub.stop().await;
}

#[tokio::test]
async fn leased_claim_nack_consumes_with_ack_false() {
    let socket = test_socket("nack");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let lease = LeasedClaim::acquire(addr.clone(), "T", 11, "worker-A", 30_000)
        .await
        .expect("acquire");
    let release = lease.nack().await.expect("nack");
    assert!(!release.ack);
    let s = hub.state.lock().await;
    assert_eq!(s.release_calls, 1);
    assert!(s.slots.is_empty());
    drop(s);
    hub.stop().await;
}
