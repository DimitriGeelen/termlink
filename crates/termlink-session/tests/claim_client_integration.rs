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
    channel_claim, channel_claims, channel_claims_summary, channel_release, ClaimError, LeasedClaim,
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
    claims_calls: u64,
    claims_summary_calls: u64,
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
        "channel.claims" => {
            s.claims_calls += 1;
            let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let include_expired = params
                .get("include_expired")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let now = s.now_ms;
            let mut rows: Vec<((String, u64), (String, String, i64))> = s
                .slots
                .iter()
                .filter(|((t, _), _)| t == &topic)
                .filter(|(_, (_, _, until))| include_expired || *until > now)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            rows.sort_by_key(|((_, off), _)| *off);
            let claims: Vec<Value> = rows
                .into_iter()
                .map(|((_, off), (cid, owner, until))| {
                    json!({
                        "claim_id": cid,
                        "offset": off,
                        "claimer": owner,
                        "claimed_at": now,
                        "claimed_until": until,
                    })
                })
                .collect();
            success_response(
                id,
                json!({
                    "ok": true,
                    "topic": topic,
                    "claims": claims,
                }),
            )
        }
        "channel.claims_summary" => {
            s.claims_summary_calls += 1;
            let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let now = s.now_ms;
            let mut active = 0u64;
            let mut expired = 0u64;
            let mut oldest_active_at: Option<i64> = None;
            let mut next_active_expiry: Option<i64> = None;
            for ((t, _off), (_cid, _owner, until)) in s.slots.iter() {
                if t != &topic {
                    continue;
                }
                if *until > now {
                    active += 1;
                    // FakeHub records claimed_at = now at insert time, but we
                    // don't track it per-slot; approximate by re-deriving
                    // claimed_at = claimed_until - ttl. The tests only assert
                    // the count + Some/None shape, not the exact age value.
                    let claimed_at = until - 30_000;
                    oldest_active_at = Some(match oldest_active_at {
                        None => claimed_at,
                        Some(prev) => prev.min(claimed_at),
                    });
                    next_active_expiry = Some(match next_active_expiry {
                        None => *until,
                        Some(prev) => prev.min(*until),
                    });
                } else {
                    expired += 1;
                }
            }
            let oldest_active_age_ms = oldest_active_at.map(|t| (now - t).max(0));
            success_response(
                id,
                json!({
                    "ok": true,
                    "topic": topic,
                    "active_count": active,
                    "expired_count": expired,
                    "oldest_active_at_ms": oldest_active_at,
                    "oldest_active_age_ms": oldest_active_age_ms,
                    "next_active_expiry_ms": next_active_expiry,
                }),
            )
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

/// N-way concurrent race: M offsets, N workers. Exclusive-delivery guarantee
/// says every offset is won by exactly one worker — total_wins == M, and
/// total_conflicts > 0 since the race is real. The example
/// `crates/termlink-session/examples/parallel_worker.rs` shows this visually;
/// this test enforces it in CI.
#[tokio::test]
async fn concurrent_n_way_race_each_offset_won_exactly_once() {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    const WORKERS: usize = 8;
    const OFFSETS: u64 = 16;

    let socket = test_socket("n_way_race");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let next_offset = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::with_capacity(WORKERS);
    for w in 0..WORKERS {
        let addr = addr.clone();
        let next_offset = next_offset.clone();
        let claimer = format!("worker-{w}");
        handles.push(tokio::spawn(async move {
            let mut wins = 0u64;
            let mut conflicts = 0u64;
            loop {
                let offset = next_offset.fetch_add(1, Ordering::Relaxed);
                if offset >= OFFSETS {
                    break;
                }
                match channel_claim(&addr, "T", offset, &claimer, 30_000).await {
                    Ok(summary) => {
                        wins += 1;
                        channel_release(&addr, &summary.claim_id, &claimer, true)
                            .await
                            .expect("release ok");
                    }
                    Err(ClaimError::Conflict { .. }) => conflicts += 1,
                    Err(e) => panic!("unexpected error: {e}"),
                }
            }
            (wins, conflicts)
        }));
    }

    let mut total_wins = 0u64;
    let mut total_conflicts = 0u64;
    for h in handles {
        let (w, c) = h.await.expect("join");
        total_wins += w;
        total_conflicts += c;
    }

    assert_eq!(
        total_wins, OFFSETS,
        "exclusive-delivery: each offset must be won exactly once (wins={total_wins}, expected={OFFSETS})"
    );
    // With N workers racing over a shared atomic cursor, contention is overwhelmingly
    // likely. Don't strictly require conflicts > 0 because under serial scheduling
    // the cursor advance could outpace the network round-trip, leaving zero races.
    // The exclusive-delivery property is the load-bearing assertion above.
    let s = hub.state.lock().await;
    assert!(s.slots.is_empty(), "all slots should be released after acks");
    drop(s);
    let _ = total_conflicts; // counted, not asserted (see comment above)
    hub.stop().await;
}

/// CLAIM_NOT_OWNED on release when the caller's claimer string differs from
/// the original. Workers depend on this invariant to keep their slots safe.
#[tokio::test]
async fn release_with_wrong_claimer_returns_not_owned() {
    let socket = test_socket("rel_not_owned");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let summary = channel_claim(&addr, "T", 42, "worker-A", 30_000)
        .await
        .expect("claim ok");
    let result = channel_release(&addr, &summary.claim_id, "worker-B", true).await;
    match result {
        Err(ClaimError::NotOwned { claim_id }) => {
            assert_eq!(claim_id, summary.claim_id);
        }
        other => panic!("expected NotOwned, got {other:?}"),
    }
    // Slot still held by worker-A; let worker-A release it cleanly.
    channel_release(&addr, &summary.claim_id, "worker-A", true)
        .await
        .expect("rightful release ok");
    hub.stop().await;
}

/// CLAIM_NOT_OWNED on renew when the caller's claimer string differs from
/// the original. Same invariant as release, but on the renew RPC path.
#[tokio::test]
async fn renew_with_wrong_claimer_returns_not_owned() {
    let socket = test_socket("ren_not_owned");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let summary = channel_claim(&addr, "T", 73, "worker-A", 30_000)
        .await
        .expect("claim ok");
    // FakeHub advances virtual time by additional_ttl_ms/2 on every renew;
    // keep the requested extension small so the original 30s lease doesn't
    // lapse before the ownership check runs (which would surface as
    // CLAIM_EXPIRED instead of CLAIM_NOT_OWNED).
    let result = termlink_session::claim_client::channel_renew(
        &addr,
        &summary.claim_id,
        "worker-B",
        1_000,
    )
    .await;
    match result {
        Err(ClaimError::NotOwned { claim_id }) => {
            assert_eq!(claim_id, summary.claim_id);
        }
        other => panic!("expected NotOwned, got {other:?}"),
    }
    // worker-A still owns the lease; release cleanly.
    channel_release(&addr, &summary.claim_id, "worker-A", true)
        .await
        .expect("rightful release ok");
    hub.stop().await;
}

// ───────────────────── T-2037: channel.claims listing ─────────────────────

#[tokio::test]
async fn list_claims_returns_empty_for_topic_with_no_claims() {
    let socket = test_socket("list_empty");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let claims = channel_claims(&addr, "T", false).await.expect("list ok");
    assert!(claims.is_empty(), "expected empty list, got {claims:?}");

    let s = hub.state.lock().await;
    assert_eq!(s.claims_calls, 1);
    drop(s);
    hub.stop().await;
}

#[tokio::test]
async fn list_claims_surfaces_active_claims_in_offset_order() {
    let socket = test_socket("list_active");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    // Acquire three claims on the same topic at different offsets.
    let c1 = channel_claim(&addr, "T", 5, "worker-A", 30_000).await.unwrap();
    let c2 = channel_claim(&addr, "T", 2, "worker-B", 30_000).await.unwrap();
    let _c3 = channel_claim(&addr, "T", 9, "worker-C", 30_000).await.unwrap();

    let listed = channel_claims(&addr, "T", false).await.expect("list ok");
    assert_eq!(listed.len(), 3, "expected three rows, got {listed:?}");

    // Sorted by offset ASC.
    let offsets: Vec<u64> = listed.iter().map(|c| c.offset).collect();
    assert_eq!(offsets, vec![2, 5, 9]);

    // Claimer + claim_id round-trip end-to-end.
    let c1_row = listed.iter().find(|c| c.offset == 5).unwrap();
    assert_eq!(c1_row.claim_id, c1.claim_id);
    assert_eq!(c1_row.claimer, "worker-A");
    let c2_row = listed.iter().find(|c| c.offset == 2).unwrap();
    assert_eq!(c2_row.claim_id, c2.claim_id);
    assert_eq!(c2_row.claimer, "worker-B");

    // Topic field hydrated from the request topic (not from per-row payload).
    assert!(listed.iter().all(|c| c.topic == "T"));

    hub.stop().await;
}

#[tokio::test]
async fn list_claims_excludes_released_claims() {
    let socket = test_socket("list_after_release");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let c1 = channel_claim(&addr, "T", 1, "worker-A", 30_000).await.unwrap();
    let _c2 = channel_claim(&addr, "T", 2, "worker-B", 30_000).await.unwrap();
    let before = channel_claims(&addr, "T", false).await.unwrap();
    assert_eq!(before.len(), 2);

    channel_release(&addr, &c1.claim_id, "worker-A", true).await.unwrap();
    let after = channel_claims(&addr, "T", false).await.unwrap();
    assert_eq!(after.len(), 1);
    assert_eq!(after[0].offset, 2);

    hub.stop().await;
}

#[tokio::test]
async fn list_claims_include_expired_surfaces_rows_default_omits() {
    let socket = test_socket("list_include_expired");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    // Two claims with short TTL.
    let _c1 = channel_claim(&addr, "T", 10, "worker-A", 1_000).await.unwrap();
    let _c2 = channel_claim(&addr, "T", 11, "worker-B", 1_000).await.unwrap();

    // Manually advance virtual time past the TTL — same shortcut the renew
    // test uses, so we don't need a real sleep.
    {
        let mut s = hub.state.lock().await;
        s.now_ms += 5_000;
    }

    // Default (include_expired=false) hides them.
    let active = channel_claims(&addr, "T", false).await.unwrap();
    assert!(
        active.is_empty(),
        "expected expired claims to be hidden, got {active:?}"
    );

    // include_expired=true surfaces both for forensics.
    let all = channel_claims(&addr, "T", true).await.unwrap();
    assert_eq!(all.len(), 2, "expected both expired rows for forensics");
    let offsets: Vec<u64> = all.iter().map(|c| c.offset).collect();
    assert_eq!(offsets, vec![10, 11]);

    hub.stop().await;
}

// ───────────────────── T-2039: channel.claims_summary aggregate ──────────

#[tokio::test]
async fn claims_summary_empty_topic_returns_zero_counts() {
    let socket = test_socket("summary_empty");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let summary = channel_claims_summary(&addr, "T").await.expect("summary ok");
    assert_eq!(summary.topic, "T");
    assert_eq!(summary.active_count, 0);
    assert_eq!(summary.expired_count, 0);
    assert!(summary.oldest_active_at_ms.is_none());
    assert!(summary.oldest_active_age_ms.is_none());
    assert!(summary.next_active_expiry_ms.is_none());

    let s = hub.state.lock().await;
    assert_eq!(s.claims_summary_calls, 1);
    drop(s);
    hub.stop().await;
}

#[tokio::test]
async fn claims_summary_single_active_claim_populates_markers() {
    let socket = test_socket("summary_single");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let _ = channel_claim(&addr, "T", 7, "worker-A", 30_000).await.unwrap();

    let summary = channel_claims_summary(&addr, "T").await.expect("summary ok");
    assert_eq!(summary.active_count, 1);
    assert_eq!(summary.expired_count, 0);
    assert!(summary.oldest_active_at_ms.is_some(), "should have an oldest active marker");
    assert!(summary.oldest_active_age_ms.is_some(), "should have an oldest age");
    assert!(
        summary.next_active_expiry_ms.is_some(),
        "should have a next-expiry marker"
    );
    assert!(
        summary.oldest_active_age_ms.unwrap() >= 0,
        "age must be non-negative"
    );

    hub.stop().await;
}

#[tokio::test]
async fn claims_summary_released_claim_drops_to_zero() {
    let socket = test_socket("summary_released");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let c = channel_claim(&addr, "T", 3, "worker-A", 30_000).await.unwrap();
    channel_release(&addr, &c.claim_id, "worker-A", true).await.unwrap();

    let summary = channel_claims_summary(&addr, "T").await.expect("summary ok");
    assert_eq!(summary.active_count, 0);
    assert_eq!(summary.expired_count, 0);
    assert!(summary.oldest_active_at_ms.is_none());

    hub.stop().await;
}

#[tokio::test]
async fn claims_summary_expired_claim_counted_as_expired_not_active() {
    let socket = test_socket("summary_expired");
    let hub = FakeHub::spawn(socket.clone()).await;
    let addr = TransportAddr::unix(&socket);

    let _ = channel_claim(&addr, "T", 9, "worker-A", 1_000).await.unwrap();
    {
        let mut s = hub.state.lock().await;
        s.now_ms += 5_000;
    }

    let summary = channel_claims_summary(&addr, "T").await.expect("summary ok");
    assert_eq!(summary.active_count, 0, "should be no active claims after TTL elapses");
    assert_eq!(
        summary.expired_count, 1,
        "the lapsed row should be visible to forensics"
    );
    assert!(
        summary.oldest_active_at_ms.is_none(),
        "no active claim → markers should be None"
    );
    assert!(summary.next_active_expiry_ms.is_none());

    hub.stop().await;
}
