//! T-2032 — end-to-end CLI smoke test for `termlink channel claim/release/renew`.
//!
//! Spins up an in-test FakeHub on a Unix socket (same pattern as
//! `termlink-session/tests/claim_client_integration.rs`) and spawns the
//! `termlink` binary as a subprocess pointed at that socket via `--hub`.
//! Verifies argument parsing → claim_client wrapper → wire roundtrip →
//! stdout rendering all line up end-to-end, plus the error-path exit.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use assert_cmd::cargo;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::process::Command;
use tokio::sync::Mutex;

use termlink_protocol::control::error_code;

#[derive(Default)]
struct HubState {
    slots: std::collections::HashMap<(String, u64), (String, String, i64)>,
    next_seq: i64,
    now_ms: i64,
}

struct FakeHub {
    #[allow(dead_code)] // retained so the Arc outlives spawn for clean shutdown
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
        Self { state, abort_tx, handle }
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
            let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let offset = params.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ttl_ms = params.get("ttl_ms").and_then(|v| v.as_u64()).unwrap_or(30_000);
            let key = (topic.clone(), offset);
            if let Some((_, _, claimed_until)) = s.slots.get(&key)
                && *claimed_until > s.now_ms
            {
                return error_response(
                    id,
                    error_code::CLAIM_CONFLICT,
                    "taken",
                    Some(json!({"topic": topic, "offset": offset})),
                );
            }
            let seq = s.next_seq;
            s.next_seq += 1;
            let claim_id = format!("clm-{seq}-{topic}-{offset}");
            let claimed_at = s.now_ms;
            let claimed_until = s.now_ms + ttl_ms as i64;
            s.slots.insert(key, (claim_id.clone(), claimer.clone(), claimed_until));
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
        "channel.release" => {
            let claim_id = params.get("claim_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ack = params.get("ack").and_then(|v| v.as_bool()).unwrap_or(false);
            let found = s.slots.iter()
                .find(|(_, v)| v.0 == claim_id)
                .map(|(k, v)| (k.clone(), v.0.clone(), v.1.clone()));
            let Some((key, cid, owner)) = found else {
                return error_response(id, error_code::CLAIM_NOT_FOUND, "gone", Some(json!({"claim_id": claim_id})));
            };
            if owner != claimer {
                return error_response(id, error_code::CLAIM_NOT_OWNED, "not yours", Some(json!({"claim_id": cid})));
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
        "channel.renew" => {
            let claim_id = params.get("claim_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let claimer = params.get("claimer").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let additional = params.get("additional_ttl_ms").and_then(|v| v.as_u64()).unwrap_or(30_000);
            let found = s.slots.iter()
                .find(|(_, v)| v.0 == claim_id)
                .map(|(k, v)| (k.clone(), v.clone()));
            let Some((key, (cid, owner, _until))) = found else {
                return error_response(id, error_code::CLAIM_NOT_FOUND, "gone", Some(json!({"claim_id": claim_id})));
            };
            if owner != claimer {
                return error_response(id, error_code::CLAIM_NOT_OWNED, "not yours", Some(json!({"claim_id": cid})));
            }
            let now = s.now_ms;
            let new_until = now + additional as i64;
            s.slots.insert(key.clone(), (cid.clone(), owner.clone(), new_until));
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
    let dir = std::env::temp_dir().join(format!("tl-cli-claim-{}-{}", std::process::id(), name));
    let _ = std::fs::create_dir_all(&dir);
    dir.join("hub.sock")
}

fn termlink_bin() -> PathBuf {
    PathBuf::from(cargo::cargo_bin!("termlink"))
}

#[tokio::test]
async fn cli_claim_release_round_trip_against_fake_hub() {
    let socket = test_socket("roundtrip");
    let hub = FakeHub::spawn(socket.clone()).await;

    // termlink channel claim T 0 --claimer worker-A --hub <socket> --json
    let out = Command::new(termlink_bin())
        .args([
            "channel", "claim", "T", "0",
            "--claimer", "worker-A",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("spawn termlink channel claim");
    assert!(out.status.success(), "claim failed: stderr={}", String::from_utf8_lossy(&out.stderr));
    let claim_resp: Value = serde_json::from_slice(&out.stdout).expect("claim json");
    assert_eq!(claim_resp["ok"], true);
    let claim_id = claim_resp["claim_id"].as_str().expect("claim_id").to_string();

    // termlink channel release --claim-id <id> --claimer worker-A --ack --hub <socket> --json
    let out = Command::new(termlink_bin())
        .args([
            "channel", "release",
            "--claim-id", &claim_id,
            "--claimer", "worker-A",
            "--ack",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("spawn termlink channel release");
    assert!(out.status.success(), "release failed: stderr={}", String::from_utf8_lossy(&out.stderr));
    let release_resp: Value = serde_json::from_slice(&out.stdout).expect("release json");
    assert_eq!(release_resp["ok"], true);
    assert_eq!(release_resp["ack"], true);
    assert_eq!(release_resp["claim_id"], claim_id);

    hub.stop().await;
}

#[tokio::test]
async fn cli_claim_conflict_exits_nonzero() {
    let socket = test_socket("conflict");
    let hub = FakeHub::spawn(socket.clone()).await;

    // First claim — succeeds.
    let out1 = Command::new(termlink_bin())
        .args([
            "channel", "claim", "T", "5",
            "--claimer", "worker-A",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("first claim");
    assert!(out1.status.success());

    // Second claim on same (topic, offset) — must surface CLAIM_CONFLICT.
    let out2 = Command::new(termlink_bin())
        .args([
            "channel", "claim", "T", "5",
            "--claimer", "worker-B",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .expect("conflicting claim");
    assert!(!out2.status.success(), "second claim should have failed");
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr.contains("already claimed") || stderr.contains("Conflict") || stderr.contains("conflict"),
        "stderr should name the conflict; got: {stderr}"
    );

    hub.stop().await;
}

#[tokio::test]
async fn cli_renew_extends_claim() {
    let socket = test_socket("renew");
    let hub = FakeHub::spawn(socket.clone()).await;

    let claim_out = Command::new(termlink_bin())
        .args([
            "channel", "claim", "T", "9",
            "--claimer", "worker-A",
            "--ttl-ms", "10000",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .output()
        .await
        .expect("claim");
    assert!(claim_out.status.success());
    let claim_resp: Value = serde_json::from_slice(&claim_out.stdout).expect("claim json");
    let claim_id = claim_resp["claim_id"].as_str().unwrap().to_string();
    let original_until = claim_resp["claimed_until"].as_i64().unwrap();

    let renew_out = Command::new(termlink_bin())
        .args([
            "channel", "renew",
            "--claim-id", &claim_id,
            "--claimer", "worker-A",
            "--additional-ttl-ms", "60000",
            "--hub", socket.to_str().unwrap(),
            "--json",
        ])
        .stdout(Stdio::piped())
        .output()
        .await
        .expect("renew");
    assert!(renew_out.status.success(), "renew should succeed");
    let renew_resp: Value = serde_json::from_slice(&renew_out.stdout).expect("renew json");
    let new_until = renew_resp["claimed_until"].as_i64().unwrap();
    assert!(
        new_until > original_until,
        "claimed_until should advance after renew (was {original_until}, now {new_until})"
    );

    hub.stop().await;
}
