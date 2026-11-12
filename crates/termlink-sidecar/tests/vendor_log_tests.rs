//! P-26 (T-3710): the vendor-channel log hook (B-18 log half, K-20, AL-39).
//! E-71: every vendor-channel use leaves an `_admin` row and a coalesced routine
//! needs-attention entry; the hook never blocks. E-170 (vendor half, FCW-146): a
//! row that cannot be written goes to the hook error log and `status` shows
//! `vendor_log: degraded`. E-140: the admin rows are the monitor's data.
//! Hermetic: testkit estate only.

use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use termlink_sidecar::api::{self, ADMIN_CONV, Api, ApiConfig, Scope, VENDOR_USE_EVENT, VENDOR_USE_KIND};
use termlink_sidecar::hook;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

struct Rig {
    e: Estate,
    store: Arc<Store>,
    errlog: PathBuf,
    _api: Api,
}

fn rig() -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let errlog = e.runtime_dir().join("vendor-hook-errors.log");
    let mut cfg = ApiConfig::new("agent-b", &e.runtime_dir());
    cfg.hook_errlog = Some(errlog.clone());
    let api = Api::start(cfg, store.clone()).unwrap();
    Rig { e, store, errlog, _api: api }
}

fn send_sock(r: &Rig) -> PathBuf {
    api::socket_path(&r.e.runtime_dir(), "agent-b", Scope::Send)
}

fn status(r: &Rig) -> Value {
    hook::call(&api::socket_path(&r.e.runtime_dir(), "agent-b", Scope::Status), &json!({"op": "status"}), hook::HOOK_TIMEOUT).unwrap()
}

fn vendor_rows(s: &Store) -> Vec<(String, String)> {
    s.chain_lines(ADMIN_CONV)
        .unwrap()
        .into_iter()
        .filter(|l| l.event == VENDOR_USE_EVENT)
        .map(|l| (l.sender, l.reason.unwrap_or_default()))
        .collect()
}

fn vendor_entries(s: &Store) -> Vec<Value> {
    s.attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["kind"] == VENDOR_USE_KIND)
        .collect()
}

/// Run the hook binary as Claude Code would: PreToolUse input on stdin.
fn run_bin(socket: &Path, errlog: &Path, input: &Value, extra: &[&str]) -> (i32, String) {
    let mut args = vec!["hook", "vendor-channel"];
    args.extend_from_slice(extra);
    let mut c = std::process::Command::new(BIN)
        .args(&args)
        .env(hook::SOCKET_ENV, socket)
        .env(hook::ERRLOG_ENV, errlog)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = c.wait_with_output().unwrap();
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

fn pre_tool(session: &str, tool: &str) -> Value {
    json!({"session_id": session, "hook_event_name": "PreToolUse", "tool_name": tool, "tool_input": {}})
}

// ---- E-71 -------------------------------------------------------------------

#[test]
fn e71_each_tool_twice_gives_six_rows_three_entries_and_never_blocks() {
    let r = rig();
    let sock = send_sock(&r);
    for tool in hook::VENDOR_TOOLS {
        for _ in 0..2 {
            let out = hook::run("vendor-channel", &pre_tool("s-1", tool).to_string(), Some(&sock), Some(&r.errlog));
            assert_eq!(out, hook::HookOutcome { stdout: None, finalize: None, error: None }, "{tool}: the hook prints nothing and never blocks");
        }
    }
    let rows = vendor_rows(&r.store);
    assert_eq!(rows.len(), 6, "one row per use: {rows:?}");
    for tool in hook::VENDOR_TOOLS {
        let n = rows.iter().filter(|(s, why)| s == "s-1" && why.starts_with(&format!("tool={tool} "))).count();
        assert_eq!(n, 2, "{tool}: two rows");
    }
    let entries = vendor_entries(&r.store);
    assert_eq!(entries.len(), 3, "entries coalesce per (session, day) for each tool: {entries:?}");
    assert!(entries.iter().all(|v| !v["kind"].as_str().unwrap().starts_with("urgent-")), "routine, not urgent");
    assert!(!r.errlog.exists(), "no error when every row was written");
    assert_eq!(status(&r)["vendor_log"], "ok");
    // The chain stays intact (append-only, hashed).
    assert!(matches!(r.store.verify_chain(ADMIN_CONV).unwrap(), termlink_sidecar::store::ChainStatus::Intact { .. }));
}

#[test]
fn e71_hook_binary_exits_zero_prints_nothing_and_logs_each_tool() {
    let r = rig();
    let sock = send_sock(&r);
    for tool in hook::VENDOR_TOOLS {
        let (rc, out) = run_bin(&sock, &r.errlog, &pre_tool("s-bin", tool), &[]);
        assert_eq!((rc, out.as_str()), (0, ""), "{tool}");
    }
    // `--tool` overrides the input's tool_name (the spec's command form).
    let (rc, _) = run_bin(&sock, &r.errlog, &pre_tool("s-bin", "Bash"), &["--tool", "Agent"]);
    assert_eq!(rc, 0);
    let rows = vendor_rows(&r.store);
    assert_eq!(rows.len(), 4, "{rows:?}");
    assert_eq!(rows.iter().filter(|(_, w)| w.starts_with("tool=Agent ")).count(), 2);
}

#[test]
fn e71_a_tool_that_is_not_a_vendor_channel_leaves_nothing() {
    let r = rig();
    let out = hook::run("vendor-channel", &pre_tool("s-1", "Bash").to_string(), Some(&send_sock(&r)), Some(&r.errlog));
    assert_eq!(out.error, None);
    assert!(vendor_rows(&r.store).is_empty());
    assert!(!r.errlog.exists());
}

#[test]
fn e71_coalesce_key_is_session_and_day() {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-b").unwrap();
    let day = 86_400_000u64;
    let use_at = |sess: &str, at: u64| {
        let o = json!({"session": sess, "tool": "SendMessage"});
        api::vendor_use(o.as_object().unwrap(), &s, at)
    };
    assert_eq!(use_at("s-1", 100 * day + 5)["entry"], true);
    assert_eq!(use_at("s-1", 100 * day + 9)["entry"], false, "same session, same day: coalesced");
    assert_eq!(use_at("s-2", 100 * day + 9)["entry"], true, "another session: its own entry");
    assert_eq!(use_at("s-1", 101 * day + 1)["entry"], true, "the next day: a new entry");
    let o = json!({"session": "s-1", "tool": "Agent"});
    assert_eq!(api::vendor_use(o.as_object().unwrap(), &s, 101 * day + 2)["entry"], true, "another tool: its own entry");
    assert_eq!(vendor_rows(&s).len(), 5, "every use has a row");
    assert_eq!(vendor_entries(&s).len(), 4);
}

#[test]
fn e71_matcher_names_exactly_the_three_vendor_tools() {
    // Control E-71.5: a matcher of SendMessage alone misses the sub-agent rows.
    assert_eq!(hook::vendor_matcher(), "SendMessage|Agent|Task");
    let none = json!({});
    for t in ["SendMessage", "Agent", "Task"] {
        assert_eq!(hook::vendor_tool(Some(t), &none), Some(t));
    }
    assert_eq!(hook::vendor_tool(Some("Bash"), &none), None);
    assert_eq!(hook::vendor_tool(None, &json!({"tool_name": "Task"})), Some("Task"));
}

#[test]
fn e71_vendor_use_is_a_send_scope_op_with_plain_names() {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-b").unwrap();
    let bad = api::dispatch(&json!({"op": "vendor-use", "session": "s/../x", "tool": "Agent"}), Scope::Send, &s, "agent-b", None);
    assert_eq!(bad["code"], "bad-name");
    let wrong = api::dispatch(&json!({"op": "vendor-use", "session": "s", "tool": "Agent"}), Scope::Admin, &s, "agent-b", None);
    assert_eq!(wrong["code"], "wrong-scope");
    let ok = api::dispatch(&json!({"op": "vendor-use", "session": "s", "tool": "Agent"}), Scope::Send, &s, "agent-b", None);
    assert_eq!(ok["ok"], true, "{ok}");
}

// ---- E-170 (vendor half, FCW-146) ---------------------------------------------

#[test]
fn e170_row_that_cannot_be_written_logs_the_use_and_status_degrades() {
    let r = rig();
    let sock = send_sock(&r);
    // The admin chain cannot be written (a directory where the log file goes;
    // file modes do not stop root, which the tests may run as).
    let chain = r.store.dir().join("log").join(format!("{ADMIN_CONV}.jsonl"));
    std::fs::create_dir_all(&chain).unwrap();
    let (rc, out) = run_bin(&sock, &r.errlog, &pre_tool("s-1", "SendMessage"), &[]);
    assert_eq!((rc, out.as_str()), (0, ""), "the call is not blocked");
    let log = std::fs::read_to_string(&r.errlog).unwrap();
    assert!(log.contains(" hook vendor-channel: ") && log.contains("vendor-log"), "{log}");
    let st = status(&r);
    assert_eq!(st["vendor_log"], "degraded", "{st}");
    assert_eq!(st["vendor_log_detail"]["errors"], 1);
    // The chain is writable again: the next use leaves a row and the state recovers;
    // the error count stays visible.
    std::fs::remove_dir(&chain).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let (rc, _) = run_bin(&sock, &r.errlog, &pre_tool("s-1", "SendMessage"), &[]);
    assert_eq!(rc, 0);
    let st = status(&r);
    assert_eq!(st["vendor_log"], "ok", "{st}");
    assert_eq!(st["vendor_log_detail"]["errors"], 1);
}

#[test]
fn e170_unreachable_sidecar_logs_the_use_and_exits_zero() {
    let e = Estate::create_without_hub().unwrap();
    let errlog = e.runtime_dir().join("hook-errors.log");
    let (rc, out) = run_bin(&e.runtime_dir().join("no-such.sock"), &errlog, &pre_tool("s-1", "Task"), &[]);
    assert_eq!((rc, out.as_str()), (0, ""));
    let log = std::fs::read_to_string(&errlog).unwrap();
    assert!(log.contains(" hook vendor-channel: connect"), "{log}");
    let s = Store::open(&e.store_root(), "agent-b").unwrap();
    assert_eq!(api::vendor_log_view(Some(&errlog), &s)["state"], "degraded");
}

#[test]
fn e170_control_status_is_ok_only_with_no_newer_error_and_unknown_when_unreadable() {
    let e = Estate::create_without_hub().unwrap();
    let s = Store::open(&e.store_root(), "agent-b").unwrap();
    let errlog = e.runtime_dir().join("hook-errors.log");
    assert_eq!(api::vendor_log_view(Some(&errlog), &s)["state"], "ok", "no log: nothing failed");
    // Lines of other hook events do not count.
    std::fs::write(&errlog, "100 hook post-tool-use: connect refused\n").unwrap();
    assert_eq!(api::vendor_log_view(Some(&errlog), &s)["state"], "ok");
    std::fs::write(&errlog, "100 hook post-tool-use: x\n200 hook vendor-channel: connect refused\n").unwrap();
    let v = api::vendor_log_view(Some(&errlog), &s);
    assert_eq!((v["state"].clone(), v["errors"].clone()), (json!("degraded"), json!(1)), "{v}");
    let unreadable = e.runtime_dir().join("errlog-dir");
    std::fs::create_dir_all(&unreadable).unwrap();
    assert_eq!(api::vendor_log_view(Some(&unreadable), &s)["state"], "unknown", "cannot look is never ok");
}
