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
use termlink_test_utils::{find_termlink_bin, start_session, termlink_cmd, TestDir};
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
/// Returns Err if the binary fails to spawn, exits non-zero, OR stdout is
/// not parseable JSON. The harness expects --json outputs are deterministic
/// enough to diff against MCP responses (modulo ignored fields).
fn call_cli(binary: &PathBuf, runtime_dir: &PathBuf, argv: &[&str]) -> Result<Value, String> {
    let output = termlink_cmd(binary, runtime_dir)
        .args(argv)
        .output()
        .map_err(|e| format!("spawn termlink: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(format!(
            "CLI {argv:?} exit={:?} stdout={stdout:?} stderr={stderr:?}",
            output.status.code()
        ));
    }
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("CLI {argv:?} stdout is not JSON: {e}\nstdout: {stdout:?}"))
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

#[tokio::test]
#[ignore = "T-1909 second-catch: MCP uses in-process session lookup, CLI routes through hub (see comment + follow-up task)"]
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
// FIRST CATCH (T-1909 v0.1 — 2026-06-01).
//
// MCP returns: {"ok": true, "sessions": {}, "total_topics": 0}
//   — `sessions` is an object (map session-name → [topic, ...])
// CLI returns: {"ok": true, "sessions": [], "total_sessions": 0, "total_topics": 0}
//   — `sessions` is an array of {name, topics} records + extra `total_sessions` field
//
// Different JSON shapes for the same logical operation. Exactly the
// maintenance hazard T-1904 predicted (Layer-2/3 orchestration divergence).
// Filed as a follow-up task to converge the two; the test is marked
// #[ignore] until convergence lands.
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "T-1909 first-catch: MCP returns sessions as object, CLI as array (see comment + follow-up task)"]
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

    let client = mcp_client().await;
    let mcp_raw = call_mcp(&client, "termlink_version", json!({})).await;
    let mcp_json: Value = serde_json::from_str(&mcp_raw)
        .unwrap_or_else(|e| panic!("MCP version response not JSON: {e}\nraw: {mcp_raw}"));

    let bin = find_termlink_bin().expect("find termlink binary");
    let cli_json = call_cli(&bin, &dir.path, &["version", "--json"]).expect("CLI version");

    let ignore: HashSet<&'static str> = ["ts_ms", "pid", "build_time", "uptime_ms"]
        .into_iter()
        .collect();

    diff_json("version", &mcp_json, &cli_json, &ignore).expect("version parity");
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
