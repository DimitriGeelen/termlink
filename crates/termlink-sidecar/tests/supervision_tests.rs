//! PH-2 row P3 (K-3, R-8, FP-1 failure 4): the component heartbeat, its status
//! view, the exit for restart when a component goes silent, and the admin note
//! the cron backstop writes when it had to act.
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use termlink_sidecar::api::{self, Api, ApiConfig, Scope};
use termlink_sidecar::beat;
use termlink_sidecar::daemon::Config;
use termlink_sidecar::hook;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

fn wait_for(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    f()
}

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
        let _ = self.0.wait();
    }
}

fn start(e: &Estate, env: &[(&str, &str)]) -> (Proc, PathBuf) {
    let roster = vec![e.identity("a").unwrap().public_key_hex()];
    let cfg = Config {
        agent: "agent-a".into(),
        instance: "inst-a".into(),
        hub: e.hub_addr(),
        hub_id: e.hub_id().unwrap(),
        key_file: e.root().join("keys").join("a.key"),
        roster,
        queue_path: e.root().join("outbox-a.sqlite"),
        poll_ms: 200,
        timeout_ms: 15_000,
        claude_projects: None,
    };
    let path = e.root().join("sidecar-a.json");
    std::fs::write(&path, serde_json::to_string(&cfg).unwrap()).unwrap();
    let log = std::fs::File::create(e.root().join("sidecar-a.log")).unwrap();
    let mut c = e.command(BIN);
    c.args(["start", "--config"]).arg(&path).stdin(Stdio::null()).stdout(log.try_clone().unwrap()).stderr(log);
    for (k, v) in env {
        c.env(k, v);
    }
    let child = c.spawn().unwrap();
    let hb = e.store_root().join("sidecar").join("agent-a").join(beat::HEARTBEAT_FILE);
    (Proc(child), hb)
}

fn read(p: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn stale_names_only_components_older_than_the_bound() {
    let beats = vec![("send".to_string(), 1000), ("receive".to_string(), 500), ("tick".to_string(), 0)];
    assert_eq!(beat::stale(&beats, 1100, 500), vec!["receive".to_string(), "tick".to_string()]);
    assert_eq!(beat::stale(&beats, 1000, 500), vec!["tick".to_string()], "receive, exactly at the bound, is alive");
    assert!(beat::stale(&beats, 10, 500).is_empty(), "a clock behind the beat is not stale");
}

#[test]
fn heartbeat_names_every_component_fresh_and_status_reports_the_supervisor() {
    let e = Estate::create().unwrap();
    let (p, hb) = start(
        &e,
        &[(beat::SUPERVISOR_ENV, "systemd"), (beat::KEEP_RUNNING_ENV, "on"), ("TERMLINK_SIDECAR_TEST_HEARTBEAT_MS", "300")],
    );
    assert!(wait_for(30, || hb.exists()), "no heartbeat file");
    // Let the loops come round once more after the first write.
    std::thread::sleep(Duration::from_millis(700));
    let v = read(&hb);
    let comps = v["components"].as_object().unwrap();
    for c in ["send", "receive", "tick", "api.send", "api.status", "api.admin"] {
        assert!(comps.contains_key(c), "component {c} missing: {v}");
    }
    assert_eq!(v["stale"], json!([]), "{v}");
    assert_eq!(v["supervisor"], "systemd");
    assert_eq!(v["keep_running"], "on");
    assert_eq!(v["pid"], json!(p.0.id()));
    let st = hook::call(&api::socket_path(&e.runtime_dir(), "agent-a", Scope::Status), &json!({"op": "status"}), Duration::from_secs(10)).unwrap();
    let sup = &st["supervision"];
    assert_eq!(sup["supervisor"], "systemd", "{st}");
    assert!(sup["heartbeat_age_ms"].as_u64().unwrap() < 5_000, "{sup}");
    for (k, age) in sup["component_age_ms"].as_object().unwrap() {
        assert!(age.as_u64().unwrap() < 5_000, "{k} is {age} ms old: {sup}");
    }
    let printed = termlink_sidecar::client::render_status(&st);
    assert!(printed.contains("supervisor systemd  keep_running on"), "{printed}");
    drop(p);
}

#[test]
fn undeclared_supervisor_reads_none_not_a_guess() {
    let e = Estate::create().unwrap();
    let (p, hb) = start(&e, &[("TERMLINK_SIDECAR_TEST_HEARTBEAT_MS", "300")]);
    assert!(wait_for(30, || hb.exists()));
    let v = read(&hb);
    assert_eq!(v["supervisor"], "none");
    assert_eq!(v["keep_running"], "unknown");
    drop(p);
}

#[test]
fn a_silent_component_makes_the_sidecar_exit_for_restart_and_says_which() {
    let e = Estate::create().unwrap();
    let (mut p, _hb) = start(
        &e,
        &[
            ("TERMLINK_SIDECAR_TEST_HEARTBEAT_MS", "200"),
            ("TERMLINK_SIDECAR_TEST_STALE_MS", "1000"),
            ("TERMLINK_SIDECAR_TEST_FREEZE", "receive"),
        ],
    );
    let mut code = None;
    assert!(
        wait_for(30, || {
            code = p.0.try_wait().unwrap().map(|s| s.code());
            code.is_some()
        }),
        "the sidecar kept running with a silent component"
    );
    assert_eq!(code.unwrap(), Some(3), "exit code 3 means: restart me");
    let log = std::fs::read_to_string(e.root().join("sidecar-a.log")).unwrap();
    assert!(log.contains("exiting for restart") && log.contains("receive"), "{log}");
    // The reason survives the restart as a needs-attention entry.
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let ent = store.attention_entries();
    assert!(ent.iter().any(|l| l.contains("component-dead") && l.contains("receive")), "{ent:?}");
}

#[test]
fn admin_note_records_an_attention_entry_and_refuses_urgent_kinds() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-n").unwrap());
    let api = Api::start(ApiConfig::new("agent-n", &e.runtime_dir()), store.clone()).unwrap();
    let admin = api.path(Scope::Admin);
    let call = |r: Value| hook::call(admin, &r, Duration::from_secs(5)).unwrap();
    let ok = call(json!({"op": "note", "kind": "supervisor-backstop", "why": "unit was start-limit-hit; reset-failed + start"}));
    assert_eq!(ok["ok"], true, "{ok}");
    let refused = call(json!({"op": "note", "kind": "urgent-overdue", "why": "spoof"}));
    assert_eq!(refused["code"], "bad-kind", "{refused}");
    let empty = call(json!({"op": "note", "kind": "supervisor-backstop", "why": "  "}));
    assert_eq!(empty["code"], "why-required", "{empty}");
    let att = call(json!({"op": "attention"}));
    assert_eq!(att["total"], 1, "{att}");
    assert_eq!(att["entries"][0]["kind"], "supervisor-backstop");
    // The note verb is admin-only.
    let on_send = hook::call(api.path(Scope::Send), &json!({"op": "note", "kind": "x", "why": "y"}), Duration::from_secs(5)).unwrap();
    assert!(on_send["code"].is_string() && on_send.get("ok") != Some(&json!(true)), "{on_send}");
}

#[test]
fn production_mode_ignores_the_test_seams() {
    let mode = termlink_sidecar::estate::Mode::Production(PathBuf::from("/nonexistent-production-root"));
    // SAFETY: test-local env; no other test reads these names concurrently in this binary.
    unsafe { std::env::set_var("TERMLINK_SIDECAR_TEST_FREEZE", "receive") };
    let (every, after, frozen) = beat::test_overrides(&mode);
    unsafe { std::env::remove_var("TERMLINK_SIDECAR_TEST_FREEZE") };
    assert_eq!((every, after, frozen), (termlink_sidecar::deadline::TICK_MS, beat::STALE_AFTER_MS, None));
}
