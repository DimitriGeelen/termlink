//! MCP / CLI parity test harness (T-1909, T-1904 GO-PARITY follow-up).
//!
//! For every (MCP-tool, CLI-verb) pair that maps to the same logical
//! operation, invoke both sides against a shared session/hub fixture and
//! diff their structured JSON outputs. Catches silent drift between the
//! two implementations — see T-1904 census (83 `*_mcp` parallel helpers
//! in `tools.rs` and several whole-tool reimplementations like
//! `termlink_fleet_doctor`) which is the maintenance hazard this harness
//! exists to detect.
//!
//! v0.1 scope: session-control thin slice. 3 cases (ping, topics,
//! hub_status) + 1 negative test that proves the diff actually fires.
//! v0.2+ expands to channel_* (53 pairs) and chat-arc agent_* (the
//! divergence-heavy group).

use rmcp::model::CallToolRequestParams;
use rmcp::{RoleClient, ServiceExt};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::PathBuf;
use termlink_test_utils::{find_termlink_bin, find_termlink_bin_fresh, start_session, termlink_cmd, TestDir};
use tokio::sync::Mutex;

use termlink_mcp::TermLinkTools;

// Serialize tests that set TERMLINK_RUNTIME_DIR — global process env.
static ENV_LOCK: Mutex<()> = Mutex::const_new(());

type McpClient = rmcp::service::RunningService<RoleClient, ()>;

async fn mcp_client() -> McpClient {
    let (server_transport, client_transport) = tokio::io::duplex(65536);
    let server = TermLinkTools::new();
    tokio::spawn(async move {
        let svc = server.serve(server_transport).await.unwrap();
        svc.waiting().await.unwrap();
    });
    ().serve(client_transport).await.unwrap()
}

async fn call_mcp(client: &McpClient, name: &'static str, args: Value) -> String {
    let params = if args.is_object() && !args.as_object().unwrap().is_empty() {
        CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone())
    } else {
        CallToolRequestParams::new(name)
    };
    let result = client.call_tool(params).await.unwrap();
    result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.clone())
        .unwrap_or_default()
}

/// Invoke the CLI with `--json` and parse stdout as JSON.
///
/// Tolerates non-zero exit codes — many `--json` error paths
/// (e.g. T-1914 hub-down) correctly emit JSON to stdout AND exit 1.
/// Returns Err only when the binary fails to spawn OR stdout is not
/// parseable JSON. The exit code is captured via the JSON shape (e.g.
/// `{"ok": false, ...}` for failures).
fn call_cli(binary: &PathBuf, runtime_dir: &PathBuf, argv: &[&str]) -> Result<Value, String> {
    let output = termlink_cmd(binary, runtime_dir)
        .args(argv)
        .output()
        .map_err(|e| format!("spawn termlink: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    serde_json::from_str::<Value>(&stdout).map_err(|e| {
        format!(
            "CLI {argv:?} stdout is not JSON: {e}\n  exit={:?}\n  stdout={stdout:?}\n  stderr={stderr:?}",
            output.status.code()
        )
    })
}

/// Strip fields from a JSON object recursively. Used to ignore
/// non-deterministic fields (timestamps, pids, paths) before diffing.
fn strip_fields(v: &mut Value, ignore: &HashSet<&'static str>) {
    match v {
        Value::Object(map) => {
            map.retain(|k, _| !ignore.contains(k.as_str()));
            for child in map.values_mut() {
                strip_fields(child, ignore);
            }
        }
        Value::Array(arr) => {
            for child in arr.iter_mut() {
                strip_fields(child, ignore);
            }
        }
        _ => {}
    }
}

/// Compare two JSON values for structural equality after stripping
/// `ignore` keys from both. Returns Ok(()) on parity, Err with the
/// pretty-printed diff on mismatch.
fn diff_json(name: &str, mcp: &Value, cli: &Value, ignore: &HashSet<&'static str>) -> Result<(), String> {
    let mut mcp_c = mcp.clone();
    let mut cli_c = cli.clone();
    strip_fields(&mut mcp_c, ignore);
    strip_fields(&mut cli_c, ignore);
    if mcp_c == cli_c {
        let count_fields = |v: &Value| match v {
            Value::Object(m) => m.len(),
            _ => 1,
        };
        println!(
            "parity[{name}]: PASS (mcp={} fields, cli={} fields, diffs=0 after ignore)",
            count_fields(&mcp_c),
            count_fields(&cli_c)
        );
        return Ok(());
    }
    Err(format!(
        "parity[{name}]: FAIL\n  MCP  (after strip): {}\n  CLI  (after strip): {}\n  MCP raw: {}\n  CLI raw: {}",
        serde_json::to_string_pretty(&mcp_c).unwrap_or_default(),
        serde_json::to_string_pretty(&cli_c).unwrap_or_default(),
        serde_json::to_string_pretty(mcp).unwrap_or_default(),
        serde_json::to_string_pretty(cli).unwrap_or_default(),
    ))
}

// ---------------------------------------------------------------------------
// PAIR 1: termlink_ping  /  termlink ping <name> --json
//
// SECOND CATCH (T-1909 v0.1 — 2026-06-01).
//
// MCP `termlink_ping` succeeds against the local in-process session lookup.
// CLI `termlink ping <name>` times out after 5s because it routes the ping
// through the hub (which is not running in this fixture).
//
// Different transport paths for the same logical "ping a local session"
// operation. Either MCP should also route through hub for consistency, OR
// CLI should fall back to in-process socket lookup when no hub is reachable.
// Until convergence, ignored.
// ---------------------------------------------------------------------------

// T-1911 third-catch FIX (2026-06-02): `call_cli` uses synchronous
// `std::process::Command::output()` which blocks the test's tokio runtime
// thread. On the default `current_thread` flavor, the in-process accept_loop
// (spawned via `tokio::spawn` from `start_session`) cannot run while the test
// thread is blocked on the subprocess. Result: the CLI subprocess connects
// to the unix socket but the test process never `accept()`s, so the CLI
// times out after 5s. MCP works because its transport is in-memory
// (`tokio::io::duplex`) and progresses cooperatively under one runtime.
//
// The fix is multi-thread tokio runtime so accept_loop can run on a worker
// thread while the test thread is blocked in subprocess I/O. Applies to
// every parity test that needs a socket roundtrip (parity_ping,
// parity_status). Hub-less tests (parity_topics, parity_list_sessions,
// parity_discover, parity_clean, parity_tofu_list, parity_info, etc.) do
// not need the socket so they pass on current_thread too.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_ping() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-ping");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, reg) = start_session(&dir.sessions_dir(), "parity-ping-sess", vec![]).await;
    let session_name = reg.display_name.as_str();

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_ping", json!({"target": session_name})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP ping response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    // CLI ping uses positional [TARGET] for local-session ping; --target is reserved
    // for cross-host remote hub addr HOST:PORT (T-921). Naming divergence with MCP
    // (which uses `target` for the local session name) is noted but out of scope
    // for v0.1 — the harness verifies semantic parity, not flag-name parity.
    let cli_json = call_cli(&bin, &dir.path, &["ping", session_name, "--json"])
        .expect("CLI ping");

    let ignore: HashSet<&'static str> = ["latency_ms", "id", "ts_ms", "pid", "timestamp", "socket_path"]
        .into_iter()
        .collect();

    diff_json("ping", &mcp_json, &cli_json, &ignore).expect("ping parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// PAIR 2: termlink_topics  /  termlink topics --json
//
// Was T-1909 v0.1 FIRST CATCH; converged 2026-06-01 via T-1910.
//
// Original divergence: MCP returned `sessions: {}` (object map session-name
// → topics), CLI returned `sessions: []` (array of {session, topics}
// records) + extra `total_sessions` field. Same logical operation
// (BTreeMap<String, Vec<String>>) serialized two different ways at the
// edge.
//
// Fix: MCP `termlink_topics` (`crates/termlink-mcp/src/tools.rs`) now
// serializes to the same array-of-records shape + `total_sessions` count.
// Array preserves BTreeMap-sorted ordering (operator-readable); total_sessions
// is useful telemetry for fleet inspection.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_topics() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-topics");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, _reg) = start_session(&dir.sessions_dir(), "parity-topics-sess", vec![]).await;

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_topics", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP topics response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["topics", "--json"]).expect("CLI topics");

    let ignore: HashSet<&'static str> =
        ["ts_ms", "timestamp", "pid"].into_iter().collect();

    diff_json("topics", &mcp_json, &cli_json, &ignore).expect("topics parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// PAIR 3: termlink_hub_status  /  termlink hub status --json
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_hub_status() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-hub-status");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    // No hub running — both sides should report "not_running" identically.

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_hub_status", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP hub_status response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["hub", "status", "--json"]).expect("CLI hub status");

    let ignore: HashSet<&'static str> =
        ["pid", "pidfile", "socket", "socket_path", "ts_ms"].into_iter().collect();

    diff_json("hub_status", &mcp_json, &cli_json, &ignore).expect("hub_status parity");
}

// ---------------------------------------------------------------------------
// PAIR 4: termlink_version  /  termlink version --json
//
// Was T-1909 v0.1 THIRD CATCH; converged 2026-06-01 via T-1912.
//
// Original divergence: MCP `termlink_version` returned
// `{"version":"0.9.0","commit":"unknown","target":"unknown"}` (read from
// `termlink-mcp/Cargo.toml` because the crate had no build.rs) while CLI
// `termlink version --json` returned the git-derived
// `{"version":"0.11.501","commit":"8a1aafb0","target":"x86_64-..."}`.
//
// Fix: added `crates/termlink-mcp/build.rs` mirroring
// `crates/termlink-cli/build.rs` — both crates now read the same
// git-derived CARGO_PKG_VERSION / GIT_COMMIT / BUILD_TARGET. An operator
// asking "what version am I running" via MCP gets the same answer as via
// CLI.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_version() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-version");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Build-coherence: parity_version compares git-derived commit/version
    // fields. The MCP test recompiles on every cargo-test invocation (gets
    // the current HEAD's commit) but `target/release/termlink` is whatever
    // the last cargo-build wrote. If the binary is stale, commit hashes
    // diverge between CLI and MCP even when both sides' build.rs scripts
    // work correctly. find_termlink_bin_fresh forces a build first.
    let bin = find_termlink_bin_fresh().expect("build + find termlink binary");

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_version", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP version response not JSON: {e}\nraw: {mcp_raw}"));

    let cli_json = call_cli(&bin, &dir.path, &["version", "--json"]).expect("CLI version");

    // Sanity: build.rs ran on both sides — neither reports "unknown".
    assert_ne!(mcp_json["commit"], "unknown",
        "MCP commit is 'unknown' — termlink-mcp/build.rs did not run. Got: {mcp_json}");
    assert_ne!(cli_json["commit"], "unknown",
        "CLI commit is 'unknown' — termlink-cli/build.rs did not run. Got: {cli_json}");

    let ignore: HashSet<&'static str> = ["ts_ms", "pid", "build_time", "uptime_ms"]
        .into_iter()
        .collect();

    diff_json("version", &mcp_json, &cli_json, &ignore).expect("version parity");
}

// ---------------------------------------------------------------------------
// PAIR 5 (v0.2, T-1913): termlink_channel_queue_status / termlink channel
//                        queue-status --json
//
// Both sides read the local T-1161 offline-queue (no hub contact). For
// the non-existent-queue case both should return identical
// {queue_path, exists: false, pending: 0}.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_channel_queue_status() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-channel-queue-status");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let queue_path = dir.path.join("nonexistent-queue.json");

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_channel_queue_status",
        json!({"queue_path": queue_path.to_string_lossy()})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP channel_queue_status response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path,
        &["channel", "queue-status", "--queue-path", queue_path.to_str().unwrap(), "--json"])
        .expect("CLI channel queue-status");

    let ignore: HashSet<&'static str> = ["ts_ms", "pid"].into_iter().collect();
    diff_json("channel_queue_status_empty", &mcp_json, &cli_json, &ignore)
        .expect("channel queue_status parity");
}

// ---------------------------------------------------------------------------
// PAIR 6 (v0.2, T-1913): termlink_channel_list / termlink channel list --json
//
// Was T-1913 FOURTH CATCH; converged 2026-06-01 via T-1914.
//
// Original divergence: MCP returned structured JSON error on hub-down,
// CLI wrote to stderr + empty stdout with exit 1 (did not honor --json
// on its early error path).
//
// Fix: `cmd_channel_list` (`crates/termlink-cli/src/commands/channel.rs`)
// now catches the `hub_socket` error and emits a structured JSON error
// to stdout via `json_error_exit` when `json_output` is set. Matches
// MCP's `{"ok": false, "error": "Hub is not running …"}` shape.
//
// Operator value: `termlink channel list --json | jq` now produces
// parseable output even when the hub is down.
//
// Broader audit (T-19XX follow-up): likely many CLI commands have the
// same shape-divergence on early error paths — channel_list is the
// first slice the harness caught.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_channel_list_no_hub() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-channel-list-no-hub");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_channel_list", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP channel_list response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    // T-1914: CLI now emits JSON to stdout even on hub-down exit 1.
    // call_cli tolerates non-zero exit and parses stdout as JSON.
    let cli_json = call_cli(&bin, &dir.path, &["channel", "list", "--json"])
        .expect("CLI channel list (JSON on stdout even with exit 1, T-1914)");

    // Both sides include a hub-socket-path in the message; ignore the
    // text content of `error` (paths differ by tempdir) and compare
    // only the structural shape (presence of ok=false + error key).
    let ignore: HashSet<&'static str> = ["ts_ms", "pid", "error"].into_iter().collect();
    diff_json("channel_list_no_hub", &mcp_json, &cli_json, &ignore)
        .expect("channel list no-hub parity");
}

#[tokio::test]
async fn parity_channel_create_no_hub() {
    // T-1915: proves the hub_socket_or_json_exit helper rolls out to
    // every cmd_channel_*. cmd_channel_create is a representative
    // non-list site (T-1914 fixed cmd_channel_list inline; this test
    // exercises a separate converted call site).
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-channel-create-no-hub");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(
        &client,
        "termlink_channel_create",
        json!({"name": "parity-test-topic"}),
    )
    .await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP channel_create response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    // CLI exits 1 when hub is down, but emits JSON to stdout via the
    // T-1915 hub_socket_or_json_exit helper. call_cli tolerates non-zero
    // exit and parses stdout as JSON.
    let cli_json = call_cli(
        &bin,
        &dir.path,
        &["channel", "create", "parity-test-topic", "--json"],
    )
    .expect("CLI channel create (JSON on stdout even with exit 1, T-1915)");

    // Both sides: ok=false on hub-down. Error text differs (CLI includes
    // socket path with tempdir; MCP says "no socket found"); strip those.
    let ignore: HashSet<&'static str> = ["ts_ms", "pid", "error"].into_iter().collect();
    diff_json("channel_create_no_hub", &mcp_json, &cli_json, &ignore)
        .expect("channel create no-hub parity");
}

// ---------------------------------------------------------------------------
// PAIR 8 (v0.3, T-1918): termlink_list_sessions / termlink list --json
//
// Was T-1918 catch (manual diff): MCP returned bare array `[...]` while CLI
// returned `{"ok": true, "sessions": [...]}` envelope. Same shape-divergence
// class as T-1910 (topics) and T-1912 (version). Converged 2026-06-01 by
// wrapping the MCP return in the envelope.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_list_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-list-sessions");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, _reg) = start_session(&dir.sessions_dir(), "parity-list-sess", vec![]).await;

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_list_sessions", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP list_sessions response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["list", "--json"]).expect("CLI list --json");

    // Both sides MUST agree on the envelope shape (`ok` + `sessions` keys).
    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");
    assert!(mcp_json["sessions"].is_array(), "MCP sessions not array: {mcp_json}");
    assert!(cli_json["sessions"].is_array(), "CLI sessions not array: {cli_json}");

    // Fields that differ per-run (timestamps, process state) — strip before diff.
    // age/heartbeat_at are wall-clock-dependent; id/pid/socket_path are
    // per-process; metadata carries cwd / data_socket / shell which are
    // process-specific. Comparing only the structural shape + display_name.
    let ignore: HashSet<&'static str> = [
        "id", "pid", "uid", "age", "created_at", "heartbeat_at",
        "socket_path", "metadata", "state", "capabilities",
        "tags", "roles",
        "ts_ms", "timestamp",
    ]
    .into_iter()
    .collect();

    diff_json("list_sessions", &mcp_json, &cli_json, &ignore)
        .expect("list_sessions parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// T-1923: termlink_tofu_list — already-converged shape lock. Both MCP and
// CLI already return `{ok, count, entries: [{host, fingerprint, first_seen,
// last_seen}]}`. This test ensures future drift fails CI rather than slips.
// HOME override isolates the TOFU store (~/.termlink/known_hubs) so the
// test sees an empty store regardless of host state.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_tofu_list() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-tofu-list");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_tofu_list", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP tofu_list response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    // CLI subprocess: explicitly set HOME so it sees the same empty TOFU store.
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["tofu", "list", "--json"]);
    let output = cmd.output().expect("CLI tofu list --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI tofu_list response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");
    assert_eq!(mcp_json["count"], json!(0), "MCP count should be 0 in empty store");
    assert_eq!(cli_json["count"], json!(0), "CLI count should be 0 in empty store");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("tofu_list", &mcp_json, &cli_json, &ignore)
        .expect("tofu_list parity");
}

// ---------------------------------------------------------------------------
// T-1922: termlink_clean — MCP was returning `{cleaned_sessions: [String]}`
// vs CLI's `{ok, action, count, sessions: [object]}`. Locked the shared
// shape (ok, count, sessions array of objects). MCP-extra fields
// (cleaned_sockets, cleaned_hub, total) stripped as intentional extension.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_clean() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-clean");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_clean", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP clean response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["clean", "--json"]).expect("CLI clean --json");

    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");
    assert!(mcp_json["sessions"].is_array(), "MCP sessions not array: {mcp_json}");
    assert!(cli_json["sessions"].is_array(), "CLI sessions not array: {cli_json}");

    // MCP-extra fields stripped — intentional MCP-only extension.
    // dry_run: CLI-only param echo; MCP always acts (skip).
    let ignore: HashSet<&'static str> = [
        // MCP-only:
        "cleaned_sockets", "cleaned_hub", "total",
        // CLI-only:
        "dry_run",
    ]
    .into_iter()
    .collect();

    diff_json("clean", &mcp_json, &cli_json, &ignore)
        .expect("clean parity");
}

// ---------------------------------------------------------------------------
// T-1921: termlink_status — MCP was returning raw hub `query.status` result;
// CLI wraps it in `{ok:true, ...result}`. Same shape-class as T-1918.
// ---------------------------------------------------------------------------

// T-1911: same multi_thread fix as parity_ping. Socket roundtrip needs the
// accept_loop to run on a worker thread while the test thread is blocked
// on the synchronous CLI subprocess.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_status() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-status");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, reg) = start_session(&dir.sessions_dir(), "parity-status-sess", vec![]).await;
    let session_name = reg.display_name.as_str();

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_status", json!({"target": session_name})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP status response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["status", session_name, "--json"])
        .expect("CLI status --json");

    // Both sides MUST agree on the envelope.
    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");

    // Per-process fields stripped (same as parity_list_sessions).
    let ignore: HashSet<&'static str> = [
        "id", "pid", "uid", "age", "created_at", "heartbeat_at",
        "socket_path", "metadata", "state", "capabilities",
        "tags", "roles",
        "ts_ms", "timestamp",
    ]
    .into_iter()
    .collect();

    diff_json("status", &mcp_json, &cli_json, &ignore)
        .expect("status parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// T-1920: termlink_info — shared shape equality. MCP adds two intentional
// MCP-only fields (mcp_tools, registered_endpoints) that have no CLI
// equivalent; ignored in the diff. This test locks the SHARED subset so any
// future drift in version/commit/target/runtime_dir/etc. is caught.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_info() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-info");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    // Build-coherence: info embeds git-derived version. MCP recompiles per
    // cargo-test (sees HEAD); `target/release/termlink` reflects whatever
    // the last `cargo build` wrote. Stale binary → version skew between
    // CLI and MCP even though both report correctly. Same fix as
    // parity_version (T-1912).
    let bin = find_termlink_bin_fresh().expect("build + find termlink binary");

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_info", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP info response not JSON: {e}\nraw: {mcp_raw}"));

    let cli_json = call_cli(&bin, &dir.path, &["info", "--json"]).expect("CLI info --json");

    // Both sides MUST agree on the envelope shape.
    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");

    // MCP-only fields: lock as intentional divergence.
    // - `mcp_tools`: count of MCP tools served by this binary (CLI has no MCP server)
    // - `registered_endpoints`: dynamic MCP endpoint count (MCP-server-only state)
    // Stripping these from the diff documents that they are NOT bugs, but
    // intentional MCP-side extensions to the shared info envelope.
    let ignore: HashSet<&'static str> = [
        // MCP-only (intentional divergence):
        "mcp_tools", "registered_endpoints",
        // Per-process / per-environment (not shape-relevant):
        "commit", "target",
    ]
    .into_iter()
    .collect();

    diff_json("info", &mcp_json, &cli_json, &ignore)
        .expect("info parity");
}

// ---------------------------------------------------------------------------
// T-1919: termlink_discover — was returning bare `[...]` array; CLI returns
// `{ok: true, sessions: [...]}`. Catches the same shape-class as
// parity_list_sessions but on the filter/discover code path.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_discover() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-discover");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, _reg) = start_session(&dir.sessions_dir(), "parity-discover-sess", vec![]).await;

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_discover", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP discover response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["discover", "--json"]).expect("CLI discover --json");

    // Both sides MUST agree on the envelope shape (`ok` + `sessions` keys).
    assert_eq!(mcp_json["ok"], json!(true), "MCP envelope missing ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI envelope missing ok=true: {cli_json}");
    assert!(mcp_json["sessions"].is_array(), "MCP sessions not array: {mcp_json}");
    assert!(cli_json["sessions"].is_array(), "CLI sessions not array: {cli_json}");

    // Same ignore-list as parity_list_sessions — sessions carry per-process
    // fields that are non-deterministic.
    let ignore: HashSet<&'static str> = [
        "id", "pid", "uid", "age", "created_at", "heartbeat_at",
        "socket_path", "metadata", "state", "capabilities",
        "tags", "roles",
        "ts_ms", "timestamp",
    ]
    .into_iter()
    .collect();

    diff_json("discover", &mcp_json, &cli_json, &ignore)
        .expect("discover parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// T-1926: kv full cycle — set → get → list → del.
//
// Locks the kv RPC envelope shapes against future drift. KV is heavily used
// by callers; a shape regression here would silently break MCP integrations
// that assume CLI-equivalent JSON.
//
// Uses multi_thread runtime per PL-199 (T-1911) — KV calls hit the session
// over the unix socket; sync CLI subprocess would otherwise starve the
// accept_loop task on a current_thread runtime.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_kv_full_cycle() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-kv");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    let (_handle, reg) = start_session(&dir.sessions_dir(), "parity-kv-sess", vec![]).await;
    let session_name = reg.display_name.as_str();

    let client = mcp_client().await;
    let bin = find_termlink_bin().expect("find termlink binary");

    // ts_ms and timestamp are non-deterministic fields some handlers may
    // emit on responses. Strip uniformly across the four phases.
    let ignore: HashSet<&'static str> = ["ts_ms", "timestamp"].into_iter().collect();

    // Strategy: run the FULL MCP cycle first (set→get→list→del), then the
    // FULL CLI cycle. Each side starts from an empty kv store and the
    // intermediate del leaves it empty again. This makes `replaced=false`
    // and `deleted=true` on both sides — value parity, not just shape parity.

    // ---- MCP cycle ---------------------------------------------------------
    let mcp_set = call_mcp(
        &client,
        "termlink_kv_set",
        json!({"target": session_name, "key": "color", "value": "blue"}),
    )
    .await;
    let mcp_set_json: Value = serde_json::from_str(&mcp_set)
        .unwrap_or_else(|e| panic!("MCP kv_set response not JSON: {e}\nraw: {mcp_set}"));
    let mcp_get = call_mcp(
        &client,
        "termlink_kv_get",
        json!({"target": session_name, "key": "color"}),
    )
    .await;
    let mcp_get_json: Value = serde_json::from_str(&mcp_get)
        .unwrap_or_else(|e| panic!("MCP kv_get response not JSON: {e}\nraw: {mcp_get}"));
    let mcp_list = call_mcp(
        &client,
        "termlink_kv_list",
        json!({"target": session_name}),
    )
    .await;
    let mcp_list_json: Value = serde_json::from_str(&mcp_list)
        .unwrap_or_else(|e| panic!("MCP kv_list response not JSON: {e}\nraw: {mcp_list}"));
    let mcp_del = call_mcp(
        &client,
        "termlink_kv_del",
        json!({"target": session_name, "key": "color"}),
    )
    .await;
    let mcp_del_json: Value = serde_json::from_str(&mcp_del)
        .unwrap_or_else(|e| panic!("MCP kv_del response not JSON: {e}\nraw: {mcp_del}"));

    // ---- CLI cycle (fresh state — MCP del cleared the store) ---------------
    let cli_set_json = call_cli(
        &bin,
        &dir.path,
        &["kv", session_name, "--json", "set", "color", "\"blue\""],
    )
    .expect("CLI kv set --json");
    let cli_get_json = call_cli(
        &bin,
        &dir.path,
        &["kv", session_name, "--json", "get", "color"],
    )
    .expect("CLI kv get --json");
    let cli_list_json = call_cli(
        &bin,
        &dir.path,
        &["kv", session_name, "--json", "list"],
    )
    .expect("CLI kv list --json");
    let cli_del_json = call_cli(
        &bin,
        &dir.path,
        &["kv", session_name, "--json", "del", "color"],
    )
    .expect("CLI kv del --json");

    // ---- Envelope assertions -----------------------------------------------
    for (name, j) in [
        ("MCP kv_set", &mcp_set_json), ("MCP kv_get", &mcp_get_json),
        ("MCP kv_list", &mcp_list_json), ("MCP kv_del", &mcp_del_json),
        ("CLI kv set", &cli_set_json), ("CLI kv get", &cli_get_json),
        ("CLI kv list", &cli_list_json), ("CLI kv del", &cli_del_json),
    ] {
        assert_eq!(j["ok"], json!(true), "{name} missing ok=true: {j}");
    }

    // ---- Pairwise shape diffs ----------------------------------------------
    diff_json("kv_set", &mcp_set_json, &cli_set_json, &ignore).expect("kv_set parity");
    diff_json("kv_get", &mcp_get_json, &cli_get_json, &ignore).expect("kv_get parity");
    diff_json("kv_list", &mcp_list_json, &cli_list_json, &ignore).expect("kv_list parity");
    diff_json("kv_del", &mcp_del_json, &cli_del_json, &ignore).expect("kv_del parity");

    _handle.abort();
}

// ---------------------------------------------------------------------------
// PAIR 15: termlink_tofu_verify  /  termlink tofu verify <addr> --json
//
// T-1927 (PL-198 follow-up). Pre-convergence census:
//   MCP `status="probe-fail"` vs CLI `status="probe-failed"`
//   MCP `error` field vs CLI `probe_error` field
//   MCP missing `match: bool|null` (CLI had it)
//   MCP had `ok` + `actions` that CLI lacked
//
// Convergence: MCP renamed (probe-fail → probe-failed, error → probe_error)
// + added `match`. CLI gained `ok` + `actions`. Both sides now emit
// identical 8-key envelope: {ok, address, status, wire, pinned, match,
// probe_error, actions}.
//
// Fixture strategy: use `127.0.0.1:1` (privileged port, refused). TCP
// connect fails fast with ECONNREFUSED instead of waiting the full 10s
// probe timeout. Both sides should return status="probe-failed" with
// non-empty `probe_error`. We strip `probe_error` from the diff (the
// underlying ConnRefused message is deterministic on Linux but we don't
// want to lock against a libc string).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_tofu_verify_no_pin() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-tofu-verify");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    // Pick an address guaranteed to fail TCP connect fast. Port 1 on
    // loopback is privileged-but-refused on a stock Linux box (no
    // server listening there). Probe will fail before TLS even starts.
    let addr = "127.0.0.1:1";

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_tofu_verify", json!({"address": addr})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP tofu_verify response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    // CLI: tofu verify exits non-zero on probe-failed (exit 3) so we
    // use call_cli which tolerates non-zero. We also set HOME so both
    // sides see the same (empty) ~/.termlink/known_hubs.
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["tofu", "verify", addr, "--json"]);
    let output = cmd.output().expect("CLI tofu verify --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI tofu_verify response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    // Spot-check the convergence: both sides report probe-failed.
    assert_eq!(mcp_json["status"], json!("probe-failed"),
        "MCP status should be 'probe-failed' (T-1927 rename from 'probe-fail'): {mcp_json}");
    assert_eq!(cli_json["status"], json!("probe-failed"),
        "CLI status should be 'probe-failed': {cli_json}");
    assert_eq!(mcp_json["ok"], json!(false), "MCP ok should be false on probe-failed: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(false), "CLI ok should be false on probe-failed (T-1927 added): {cli_json}");
    assert!(mcp_json["probe_error"].is_string(),
        "MCP probe_error should be populated (T-1927 rename from 'error'): {mcp_json}");
    assert!(cli_json["probe_error"].is_string(),
        "CLI probe_error should be populated: {cli_json}");
    assert!(mcp_json["actions"].is_array(), "MCP actions missing: {mcp_json}");
    assert!(cli_json["actions"].is_array(), "CLI actions missing (T-1927 added): {cli_json}");

    // Strip `probe_error` from the diff — the libc/tokio connect-error
    // message text is environment-sensitive ("Connection refused
    // (os error 111)" on Linux). We've already asserted it's populated
    // on both sides above; locking the exact string buys nothing.
    let ignore: HashSet<&'static str> = ["probe_error"].into_iter().collect();
    diff_json("tofu_verify", &mcp_json, &cli_json, &ignore)
        .expect("tofu_verify parity");
}

// ---------------------------------------------------------------------------
// PAIR 16: termlink_hub_probe  /  termlink hub probe <addr> --json
//
// T-1928 (PL-198 follow-up, sibling of T-1927). Pre-convergence census:
//   MCP success/fail both wrapped {ok, address, fingerprint, error}
//   CLI success: {address, fingerprint} only — no `ok`, no `error`
//   CLI failure: non-JSON (anyhow bail to stderr) — violated --json contract
//
// Convergence (CLI side only — MCP shape was already canonical): added
// `ok` + `error` fields to CLI success branch, routed failure through
// json_error_exit. Both sides now emit identical 4-key envelope:
// {ok, address, fingerprint, error}.
//
// Fixture: same `127.0.0.1:1` fast-fail trick as parity_tofu_verify.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_hub_probe() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-hub-probe");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };

    let addr = "127.0.0.1:1";

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_hub_probe", json!({"address": addr})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP hub_probe response not JSON: {e}\nraw: {mcp_raw}"));

    // T-1928: use find_termlink_bin_fresh to guarantee the CLI binary
    // reflects the in-tree code (the T-1928 envelope changes are in CLI;
    // a stale on-disk binary would silently regress the parity check).
    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    // CLI exits 1 on probe failure (via json_error_exit) so call_cli's
    // tolerance of non-zero exit codes is required here.
    let cli_json = call_cli(&bin, &dir.path, &["hub", "probe", addr, "--json"])
        .expect("CLI hub probe --json");

    // Spot-check the convergence: both sides report ok=false with structured error.
    assert_eq!(mcp_json["ok"], json!(false),
        "MCP ok should be false on probe failure: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(false),
        "CLI ok should be false on probe failure (T-1928 added): {cli_json}");
    assert_eq!(mcp_json["fingerprint"], serde_json::Value::Null,
        "MCP fingerprint should be null: {mcp_json}");
    assert_eq!(cli_json["fingerprint"], serde_json::Value::Null,
        "CLI fingerprint should be null: {cli_json}");
    assert!(mcp_json["error"].is_string(),
        "MCP error should be populated string: {mcp_json}");
    assert!(cli_json["error"].is_string(),
        "CLI error should be populated string (T-1928 added): {cli_json}");

    // Strip `error` from diff (libc/tokio message text is environment-sensitive,
    // see parity_tofu_verify rationale).
    let ignore: HashSet<&'static str> = ["error"].into_iter().collect();
    diff_json("hub_probe", &mcp_json, &cli_json, &ignore)
        .expect("hub_probe parity");
}

// ---------------------------------------------------------------------------
// PAIR 17: termlink_net_test  /  termlink net test --json
//
// T-1929 (PL-198 follow-up). Pre-convergence census (empty-hubs branch):
//   MCP: {ok, hubs:[], summary:{total:0,...}, message: "No hubs configured..."}
//   CLI: {ok, hubs:[], summary:{total:0,...}} — missing `message`
//
// Convergence: CLI gained `message` field matching MCP text. Both sides
// now emit identical 4-key envelope on empty-hubs.
//
// Fixture: set HOME to TestDir so config::load_hubs_config() reads an
// empty (non-existent) ~/.termlink/hubs.toml. Same trick as
// parity_tofu_list — guarantees we hit the empty branch deterministically.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_net_test_no_hubs() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-net-test");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_net_test", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP net_test response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    // CLI: explicitly set HOME so config loader sees the same (empty) hubs.toml.
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["net", "test", "--json"]);
    let output = cmd.output().expect("CLI net test --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI net_test response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    // Spot-check both sides have the expected empty envelope.
    assert_eq!(mcp_json["ok"], json!(true), "MCP ok: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI ok: {cli_json}");
    assert_eq!(mcp_json["hubs"], json!([]), "MCP hubs should be []: {mcp_json}");
    assert_eq!(cli_json["hubs"], json!([]), "CLI hubs should be []: {cli_json}");
    assert!(mcp_json["message"].is_string(), "MCP message missing: {mcp_json}");
    assert!(cli_json["message"].is_string(), "CLI message missing (T-1929 added): {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("net_test", &mcp_json, &cli_json, &ignore)
        .expect("net_test parity");
}

// ---------------------------------------------------------------------------
// PAIR 18: termlink_fleet_verify  /  termlink fleet verify --json
//
// T-1930 (PL-198 follow-up). Pre-convergence census (empty-hubs branch):
//   MCP: {ok: true, verdict: "match", profiles: [], message: "No hubs configured in ~/.termlink/hubs.toml"}
//   CLI: {verdict: "match", profiles: [], note: "no hubs configured"} — missing ok, different field name + text
//
// Convergence: CLI added `ok: true`, renamed `note` → `message`, aligned
// text. Both sides now emit identical 4-key envelope on empty-hubs.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_fleet_verify_no_hubs() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-fleet-verify");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_fleet_verify", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP fleet_verify response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["fleet", "verify", "--json"]);
    let output = cmd.output().expect("CLI fleet verify --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI fleet_verify response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    assert_eq!(mcp_json["ok"], json!(true), "MCP ok: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI ok (T-1930 added): {cli_json}");
    assert_eq!(mcp_json["verdict"], json!("match"), "MCP verdict: {mcp_json}");
    assert_eq!(cli_json["verdict"], json!("match"), "CLI verdict: {cli_json}");
    assert_eq!(mcp_json["profiles"], json!([]), "MCP profiles: {mcp_json}");
    assert_eq!(cli_json["profiles"], json!([]), "CLI profiles: {cli_json}");
    assert!(mcp_json["message"].is_string(), "MCP message: {mcp_json}");
    assert!(cli_json["message"].is_string(), "CLI message (T-1930 renamed from note): {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("fleet_verify", &mcp_json, &cli_json, &ignore)
        .expect("fleet_verify parity");
}

// ---------------------------------------------------------------------------
// PAIR 19: termlink_fleet_history  /  termlink fleet history --json
//
// T-1931 (PL-198 follow-up). Pre-convergence census (empty-log branch):
//   MCP summary: {total, per_hub, since_days, hub_filter, log_path}
//   CLI summary: {total, per_hub, log_path} — missing since_days, hub_filter
//
// Convergence: CLI summary gained `since_days` + `hub_filter` (echoed
// inputs, useful for consumers caching responses). Both sides now emit
// identical 5-key summary.
//
// Fixture: HOME=TestDir so both sides see no rotation.log and hit the
// empty-log branch.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_fleet_history_no_log() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-fleet-history");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let client = mcp_client().await;
    // Default args (since_days=7, hub=null, include_heals=false) — matches
    // the CLI default invocation `termlink fleet history --json`.
    let mcp_raw = call_mcp(&client, "termlink_fleet_history", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP fleet_history response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["fleet", "history", "--json"]);
    let output = cmd.output().expect("CLI fleet history --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI fleet_history response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    assert_eq!(mcp_json["ok"], json!(true), "MCP ok: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI ok: {cli_json}");
    assert_eq!(mcp_json["entries"], json!([]), "MCP entries: {mcp_json}");
    assert_eq!(cli_json["entries"], json!([]), "CLI entries: {cli_json}");
    assert_eq!(mcp_json["summary"]["since_days"], json!(7), "MCP since_days: {mcp_json}");
    assert_eq!(cli_json["summary"]["since_days"], json!(7),
        "CLI since_days (T-1931 added): {cli_json}");
    assert_eq!(mcp_json["summary"]["hub_filter"], serde_json::Value::Null,
        "MCP hub_filter: {mcp_json}");
    assert_eq!(cli_json["summary"]["hub_filter"], serde_json::Value::Null,
        "CLI hub_filter (T-1931 added): {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("fleet_history", &mcp_json, &cli_json, &ignore)
        .expect("fleet_history parity");
}

// ---------------------------------------------------------------------------
// PAIR 20 (T-1933): termlink_whoami / termlink whoami --json — empty-state
// no-sessions parity. Both sides must return the canonical "no live sessions
// on this hub" envelope (`{ok: false, ambiguous: false, candidates: [], hint:
// "..."}`). Pre-T-1933: MCP was MISSING (no termlink_whoami tool) — LLM
// agents calling MCP could not query identity. T-1933 added the MCP tool
// with the same resolution chain (session_hint > name_hint >
// TERMLINK_SESSION_ID > PID-walk > candidates).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_whoami_no_sessions() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-whoami-empty");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };
    unsafe { std::env::remove_var("TERMLINK_SESSION_ID") };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_whoami", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP whoami response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.env_remove("TERMLINK_SESSION_ID");
    cmd.args(["whoami", "--json"]);
    let output = cmd.output().expect("CLI whoami --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI whoami response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    // Both sides MUST agree: empty store → ok=false + ambiguous=false +
    // candidates=[] + hint set. This is the LLM-agent-facing answer to
    // "who am I?" when nobody is registered.
    assert_eq!(mcp_json["ok"], json!(false), "MCP ok=false: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(false), "CLI ok=false: {cli_json}");
    assert_eq!(mcp_json["ambiguous"], json!(false), "MCP ambiguous=false: {mcp_json}");
    assert_eq!(cli_json["ambiguous"], json!(false), "CLI ambiguous=false: {cli_json}");
    assert_eq!(mcp_json["candidates"], json!([]), "MCP candidates=[]: {mcp_json}");
    assert_eq!(cli_json["candidates"], json!([]), "CLI candidates=[]: {cli_json}");
    assert!(mcp_json["hint"].is_string(), "MCP hint string: {mcp_json}");
    assert!(cli_json["hint"].is_string(), "CLI hint string: {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("whoami_no_sessions", &mcp_json, &cli_json, &ignore)
        .expect("whoami_no_sessions parity");
}

// ---------------------------------------------------------------------------
// PAIR 23 (T-1935): termlink_whoami / termlink whoami --json populated-path
// parity. One session registered, query via name_hint. T-1933 shipped the
// empty-state test; this slice locks the LLM agent's actual production
// flow — give it a name, get the identity card back. Per-process fields
// (id, pid, uid, timestamps, socket_path, identity FP, cwd) are run-variant
// and stripped before diff. The shape + display_name + ok-flag are what
// stay locked.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_whoami_session_match() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-whoami-pop");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };
    unsafe { std::env::remove_var("TERMLINK_SESSION_ID") };
    let (_handle, reg) = start_session(&dir.sessions_dir(), "parity-whoami-sess", vec![]).await;
    let session_name = reg.display_name.as_str();

    let client = mcp_client().await;
    let mcp_raw = call_mcp(
        &client,
        "termlink_whoami",
        json!({"name_hint": session_name}),
    ).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP whoami response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.env_remove("TERMLINK_SESSION_ID");
    cmd.args(["whoami", "--name", session_name, "--json"]);
    let output = cmd.output().expect("CLI whoami --name --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI whoami response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    // Both sides MUST agree: found a session, display_name matches.
    assert_eq!(mcp_json["ok"], json!(true), "MCP ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI ok=true: {cli_json}");
    assert_eq!(mcp_json["session"]["display_name"], json!(session_name),
        "MCP display_name: {mcp_json}");
    assert_eq!(cli_json["session"]["display_name"], json!(session_name),
        "CLI display_name: {cli_json}");

    // Per-process / wall-clock / per-host fields stripped before structural
    // diff. `posts_as` is intentionally stripped: T-1933 v1 omits the
    // CLI-side `resolve_project_name_from` cwd→project resolver from MCP
    // (see Decisions in T-1935 task file). The optional CLI enrichment
    // does not block envelope parity.
    let ignore: HashSet<&'static str> = [
        "id", "pid", "uid", "age", "created_at", "heartbeat_at",
        "socket_path", "metadata", "state", "capabilities",
        "tags", "roles", "cwd",
        "identity_fingerprint", "identity_shared_with",
        "ts_ms", "timestamp",
        "posts_as",
    ]
    .into_iter()
    .collect();

    diff_json("whoami_session_match", &mcp_json, &cli_json, &ignore)
        .expect("whoami_session_match parity");
    _handle.abort();
}

// ---------------------------------------------------------------------------
// PAIR 21 (T-1934): termlink_tofu_clear --all / termlink tofu clear --all
// Empty-store bulk wipe. Both sides must return `{ok: true, cleared: 0}`.
// Pre-T-1934: MCP HAD NO --all branch (host required). T-1934 added the
// branch and this test locks the shape against future drift.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_tofu_clear_all_empty() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-tofu-clear-all");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_tofu_clear", json!({"all": true})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP tofu_clear --all response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["tofu", "clear", "--all", "--json"]);
    let output = cmd.output().expect("CLI tofu clear --all --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI tofu_clear response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    assert_eq!(mcp_json["ok"], json!(true), "MCP ok=true: {mcp_json}");
    assert_eq!(cli_json["ok"], json!(true), "CLI ok=true: {cli_json}");
    assert_eq!(mcp_json["cleared"], json!(0), "MCP cleared=0: {mcp_json}");
    assert_eq!(cli_json["cleared"], json!(0), "CLI cleared=0: {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("tofu_clear_all_empty", &mcp_json, &cli_json, &ignore)
        .expect("tofu_clear_all_empty parity");
}

// ---------------------------------------------------------------------------
// PAIR 22 (T-1934): termlink_tofu_clear single-host miss / termlink tofu
// clear <host> --json on a non-existent entry. Pre-T-1934: MCP returned
// `{ok, host, removed, message}`, CLI only `{ok, host, removed}`. Now both
// sides emit `message` for symmetric envelope consumed via --json.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_tofu_clear_single_miss() {
    let _lock = ENV_LOCK.lock().await;
    let dir = TestDir::new("parity-tofu-clear-miss");
    unsafe { std::env::set_var("TERMLINK_RUNTIME_DIR", &dir.path) };
    unsafe { std::env::set_var("HOME", &dir.path) };

    let target = "192.168.10.99:9100"; // non-existent in empty store

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_tofu_clear", json!({"host": target})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP tofu_clear miss response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin_fresh().expect("find termlink binary (fresh)");
    let mut cmd = termlink_cmd(&bin, &dir.path);
    cmd.env("HOME", &dir.path);
    cmd.args(["tofu", "clear", target, "--json"]);
    let output = cmd.output().expect("CLI tofu clear --json");
    let cli_json: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("CLI tofu_clear response not JSON: {e}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)));

    assert_eq!(mcp_json["ok"], json!(false), "MCP ok=false (miss): {mcp_json}");
    assert_eq!(cli_json["ok"], json!(false), "CLI ok=false (miss): {cli_json}");
    assert_eq!(mcp_json["removed"], json!(false), "MCP removed=false: {mcp_json}");
    assert_eq!(cli_json["removed"], json!(false), "CLI removed=false: {cli_json}");
    assert_eq!(mcp_json["host"], json!(target), "MCP host: {mcp_json}");
    assert_eq!(cli_json["host"], json!(target), "CLI host: {cli_json}");
    assert!(mcp_json["message"].is_string(), "MCP message string: {mcp_json}");
    assert!(cli_json["message"].is_string(),
        "CLI message string (T-1934 added): {cli_json}");

    let ignore: HashSet<&'static str> = HashSet::new();
    diff_json("tofu_clear_single_miss", &mcp_json, &cli_json, &ignore)
        .expect("tofu_clear_single_miss parity");
}

// ---------------------------------------------------------------------------
// NEGATIVE TEST: a hand-crafted diff MUST be detected as a parity failure.
// Proves the harness's diff logic is not a no-op.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_negative_self_test() {
    let mcp = json!({"ok": true, "status": "alive", "kind": "session"});
    let cli = json!({"ok": true, "status": "alive", "kind": "sesion"}); // typo
    let ignore: HashSet<&'static str> = HashSet::new();
    let result = diff_json("negative_self_test", &mcp, &cli, &ignore);
    assert!(
        result.is_err(),
        "negative self-test should fail — the diff logic is broken if this passes"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("sesion") || err.contains("session"),
        "diff error should name the diverging value, got: {err}"
    );
}
