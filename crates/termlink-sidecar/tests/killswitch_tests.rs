//! PH-2 row P6 (SI-23, OQ-e ruled A): the kill switch for one sidecar or every
//! listed one, through the client binary, each flip read back.
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

fn call(sock: &Path, req: Value) -> Value {
    let mut s = UnixStream::connect(sock).unwrap();
    writeln!(s, "{req}").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

/// Two sidecars' APIs in one estate (agents b and c).
fn two() -> (Estate, Vec<(Arc<Store>, Api)>) {
    let e = Estate::create_without_hub().unwrap();
    let mut v = vec![];
    for name in ["agent-b", "agent-c"] {
        let store = Arc::new(Store::open(&e.store_root(), name).unwrap());
        let api = Api::start(ApiConfig::new(name, &e.runtime_dir()), store.clone()).unwrap();
        v.push((store, api));
    }
    (e, v)
}

fn run(args: &[&str], env: &[(&str, &str)]) -> (i32, String, String) {
    let mut c = Command::new(BIN);
    c.args(args).env_remove("TERMLINK_SIDECAR_ADMIN_SOCKET").env_remove("TERMLINK_SIDECAR_ADMIN_SOCKETS");
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).into_owned(), String::from_utf8_lossy(&o.stderr).into_owned())
}

#[test]
fn delivery_query_reads_the_flag_and_logs_nothing() {
    let (_e, s) = two();
    let (store, api) = &s[0];
    let admin = api.path(Scope::Admin);
    assert_eq!(call(admin, json!({"op": "delivery", "state": "query"}))["delivery"], "on");
    let before = store.chain_lines("_admin").map(|l| l.len()).unwrap_or(0);
    call(admin, json!({"op": "delivery", "state": "off", "why": "t"}));
    assert_eq!(call(admin, json!({"op": "delivery", "state": "query"}))["delivery"], "off");
    let after = store.chain_lines("_admin").map(|l| l.len()).unwrap_or(0);
    assert_eq!(after, before + 1, "the flip is logged once; the queries are not");
}

#[test]
fn fleet_switch_flips_every_listed_sidecar_and_verifies_each() {
    let (e, s) = two();
    let list = e.root().join("admin-sockets.list");
    let socks: Vec<String> = s.iter().map(|(_, a)| a.path(Scope::Admin).display().to_string()).collect();
    std::fs::write(&list, format!("# fleet\n{}\n\n{}\n", socks[0], socks[1])).unwrap();
    let (rc, out, err) = run(&["admin", "delivery", "off", "--why", "drill", "--all", "--list", list.to_str().unwrap()], &[]);
    assert_eq!(rc, 0, "{out}{err}");
    assert!(out.contains("delivery off: 2 of 2 sidecars verified"), "{out}");
    for (store, api) in &s {
        assert_eq!(call(api.path(Scope::Admin), json!({"op": "delivery", "state": "query"}))["delivery"], "off");
        let logged = store.chain_lines("_admin").unwrap();
        assert!(logged.iter().any(|l| l.event == "DELIVERY_OFF" && l.reason.as_deref() == Some("drill")), "{logged:?}");
    }
    // Back on through the env form, as JSON.
    let joined = socks.join(":");
    let (rc, out, _) = run(&["admin", "delivery", "on", "--why", "drill over", "--all", "--json"], &[("TERMLINK_SIDECAR_ADMIN_SOCKETS", &joined)]);
    assert_eq!(rc, 0);
    let v: Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!((v["ok"].clone(), v["results"].as_array().unwrap().len()), (json!(true), 2), "{v}");
}

#[test]
fn fleet_switch_names_a_sidecar_it_could_not_flip_and_exits_1() {
    let (e, s) = two();
    let list = e.root().join("admin-sockets.list");
    let gone = e.root().join("gone.admin.sock");
    std::fs::write(&list, format!("{}\n{}\n", s[0].1.path(Scope::Admin).display(), gone.display())).unwrap();
    let (rc, out, _) = run(&["admin", "delivery", "off", "--why", "drill", "--all", "--list", list.to_str().unwrap()], &[]);
    assert_eq!(rc, 1, "{out}");
    assert!(out.contains("FAILED") && out.contains("gone.admin.sock") && out.contains("1 of 2 sidecars verified"), "{out}");
    // The reachable one was still switched off.
    assert_eq!(call(s[0].1.path(Scope::Admin), json!({"op": "delivery", "state": "query"}))["delivery"], "off");
}

#[test]
fn fleet_switch_refuses_without_a_reason_or_a_socket() {
    let (_e, s) = two();
    let sock = s[0].1.path(Scope::Admin).display().to_string();
    let (rc, _, err) = run(&["admin", "delivery", "off", "--socket", &sock], &[]);
    assert_eq!(rc, 2, "{err}");
    assert!(err.contains("--why is required"), "{err}");
    let (rc, _, err) = run(&["admin", "delivery", "off", "--why", "x", "--all"], &[]);
    assert_eq!(rc, 2, "{err}");
    let (rc, _, _) = run(&["admin", "delivery", "sideways", "--why", "x", "--socket", &sock], &[]);
    assert_eq!(rc, 2);
    // Nothing was flipped by the refused calls.
    assert_eq!(call(s[0].1.path(Scope::Admin), json!({"op": "delivery", "state": "query"}))["delivery"], "on");
}
