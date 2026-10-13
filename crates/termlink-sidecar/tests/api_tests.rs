//! B5 tests for C-4 (api.rs) and the status view. Hermetic: testkit estate only.

use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use termlink_sidecar::api::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

struct Rig {
    e: Estate,
    store: Arc<Store>,
    api: Api,
}

fn rig_with(mut tweak: impl FnMut(&mut ApiConfig)) -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let mut cfg = ApiConfig::new("agent-b", &e.runtime_dir());
    tweak(&mut cfg);
    let api = Api::start(cfg, store.clone()).unwrap();
    Rig { e, store, api }
}

fn rig() -> Rig {
    rig_with(|_| {})
}

fn call(r: &Rig, scope: Scope, req: Value) -> Value {
    let mut s = UnixStream::connect(r.api.path(scope)).unwrap();
    writeln!(s, "{req}").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap_or_else(|e| panic!("bad reply {line:?}: {e}"))
}

#[test]
fn api_sockets_are_owner_only() {
    let r = rig();
    let dir = r.e.runtime_dir().join("sidecar");
    assert_eq!(fs::metadata(&dir).unwrap().permissions().mode() & 0o777, 0o700);
    for sc in Scope::ALL {
        let m = fs::metadata(r.api.path(sc)).unwrap().permissions().mode() & 0o777;
        assert_eq!(m, 0o600, "{sc:?}");
    }
    let names: Vec<_> = Scope::ALL.iter().map(|s| r.api.path(*s).file_name().unwrap().to_string_lossy().into_owned()).collect();
    assert_eq!(names, ["agent-b.send.sock", "agent-b.status.sock", "agent-b.admin.sock"]);
}

#[test]
fn api_other_uid_refused_on_every_scope() {
    // Owner is declared to be some other uid, so our own connection is the stranger.
    let r = rig_with(|c| c.owner_uid = c.owner_uid.wrapping_add(4242));
    for sc in Scope::ALL {
        let v = call(&r, sc, json!({"op": "status"}));
        assert_eq!(v["code"], "peer-refused", "{sc:?}: {v}");
        assert!(v.get("rows").is_none());
    }
    // And the same connection is served when the owner is right.
    let ok = rig();
    assert_eq!(call(&ok, Scope::Status, json!({"op": "status"}))["ok"], true);
}

#[test]
fn api_real_other_uid_cannot_reach_the_socket() {
    // SAFETY: getuid has no preconditions.
    if unsafe { libc::getuid() } != 0 {
        return; // needs root to drop to another uid
    }
    let r = rig();
    let path = r.api.path(Scope::Send).to_path_buf();
    let mut cmd = std::process::Command::new("python3");
    cmd.args(["-I", "-c", "import socket,sys\ns=socket.socket(socket.AF_UNIX)\ns.connect(sys.argv[1])", path.to_str().unwrap()]);
    // SAFETY: setgid/setuid are async-signal-safe and the closure allocates nothing.
    unsafe {
        use std::os::unix::process::CommandExt;
        cmd.pre_exec(|| {
            if libc::setgid(65534) != 0 || libc::setuid(65534) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    // No python3 or cannot drop uid here: nothing to assert.
    if let Ok(o) = cmd.output() {
        assert!(!o.status.success(), "uid 65534 connected to an owner-only socket");
    }
}

#[test]
fn api_agent_sockets_cannot_reach_admin() {
    let r = rig();
    let off = json!({"op": "delivery", "state": "off"});
    for sc in [Scope::Send, Scope::Status] {
        let v = call(&r, sc, off.clone());
        assert_eq!(v["code"], "wrong-scope", "{sc:?}: {v}");
    }
    assert!(!r.store.dir().join(DELIVERY_OFF_FLAG).exists(), "flag set from an agent socket");
    // The admin socket does serve it, and nothing else.
    assert_eq!(call(&r, Scope::Admin, off)["delivery"], "off");
    assert!(r.store.dir().join(DELIVERY_OFF_FLAG).exists());
    assert_eq!(call(&r, Scope::Status, json!({"op": "status"}))["delivery"], "off");
    assert_eq!(call(&r, Scope::Admin, json!({"op": "status"}))["code"], "wrong-scope");
    assert_eq!(call(&r, Scope::Admin, json!({"op": "send"}))["code"], "wrong-scope");
    assert_eq!(call(&r, Scope::Admin, json!({"op": "delivery", "state": "on"}))["delivery"], "on");
    assert!(!r.store.dir().join(DELIVERY_OFF_FLAG).exists());
}

#[test]
fn api_status_scope_cannot_send() {
    let r = rig();
    let v = call(&r, Scope::Status, json!({"op": "send", "to": "x", "body": "hi"}));
    assert_eq!(v["code"], "wrong-scope");
}

#[test]
fn api_unknown_op_field_and_malformed_are_refused_not_ignored() {
    let r = rig();
    assert_eq!(call(&r, Scope::Send, json!({"op": "frobnicate"}))["code"], "unknown-op");
    assert_eq!(call(&r, Scope::Send, json!({"nope": 1}))["code"], "bad-request");
    assert_eq!(call(&r, Scope::Send, json!([1, 2]))["code"], "bad-request");
    // Structured error shape on every refusal.
    let v = call(&r, Scope::Send, json!({"op": "frobnicate"}));
    for k in ["error", "code", "hint"] {
        assert!(v[k].is_string(), "{k} missing in {v}");
    }
    // Raw garbage line.
    let mut s = UnixStream::connect(r.api.path(Scope::Send)).unwrap();
    writeln!(s, "this is not json").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&line).unwrap()["code"], "bad-request");
}

#[test]
fn api_no_verb_takes_a_path_or_command() {
    let r = rig();
    for (sc, req) in [
        (Scope::Send, json!({"op": "handed-over", "msg_id": "m1", "nonce": "n", "transcript_path": "/etc/passwd"})),
        (Scope::Send, json!({"op": "send", "to": "b", "body": "x", "cmd": "rm -rf /"})),
        (Scope::Status, json!({"op": "status", "path": "/tmp/x"})),
    ] {
        assert_eq!(call(&r, sc, req)["code"], "unknown-field");
    }
    // Names that look like paths are refused in every name-typed field.
    for bad in ["../../etc/passwd", "/abs", "a/b", "", "x y", "a\nb"] {
        let v = call(&r, Scope::Status, json!({"op": "status", "conv": bad}));
        assert_eq!(v["code"], "bad-name", "{bad:?}: {v}");
    }
}

#[test]
fn api_unserved_verbs_are_loud_not_faked() {
    let r = rig();
    // B11: reply is served through the send path; without one it says so.
    let v = call(&r, Scope::Send, json!({"op": "reply", "conv": "c1", "in_reply_to": "m1", "body": "x"}));
    assert_eq!(v["code"], "not-configured", "{v}");
    assert!(v.get("msg_id").is_none() && v.get("stage").is_none(), "{v}");
    // B9: served, but a sidecar with no deliver path says so; it never fakes a stage.
    for req in [
        json!({"op": "pull", "session": "s1", "hook": "stop"}),
        json!({"op": "claim", "msg_id": "m1"}),
        json!({"op": "handed-over", "msg_id": "m1", "nonce": "n"}),
    ] {
        let v = call(&r, Scope::Send, req);
        assert_eq!(v["code"], "not-configured", "{v}");
        assert!(v.get("msg_id").is_none() && v.get("stage").is_none(), "{v}");
    }
    // B6: `send` is served, but a sidecar with no send path says so; it never fakes SENT.
    let v = call(&r, Scope::Send, json!({"op": "send", "to": "b", "to_instance": "i", "conv": "c1", "body": "x"}));
    assert_eq!(v["code"], "not-configured", "{v}");
    assert!(v.get("msg_id").is_none() && v.get("stage").is_none(), "{v}");
}

#[test]
fn api_oversize_request_refused() {
    let r = rig();
    let mut s = UnixStream::connect(r.api.path(Scope::Send)).unwrap();
    let big = "a".repeat(MAX_REQUEST as usize + 10);
    // The server may close after replying; ignore a broken pipe on the tail.
    let _ = s.write_all(big.as_bytes());
    let _ = s.write_all(b"\n");
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    assert_eq!(serde_json::from_str::<Value>(&line).unwrap()["code"], "bad-request");
}

#[test]
fn api_refuses_to_start_outside_the_estate() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let outside = std::env::temp_dir().join(format!("tl-api-unmarked-{}", std::process::id()));
    fs::create_dir_all(&outside).unwrap();
    let r = Api::start(ApiConfig::new("agent-b", &outside), store);
    assert!(r.is_err());
    assert!(!outside.join("sidecar").exists());
    let _ = fs::remove_dir_all(&outside);
}

#[test]
fn api_status_names_missing_strength_flags_and_hub_id() {
    let r = rig();
    let v = call(&r, Scope::Status, json!({"op": "status"}));
    assert_eq!(v["ok"], true);
    assert_eq!(v["hub_id"], "not configured");
    // PH-2 row P1: status says where the store runs (the rig is a test estate).
    assert_eq!(v["mode"], "estate");
    let missing = v["missing_strength"].to_string();
    for needle in ["no per-agent accounts", "no operator key", "tail truncation: not yet detectable", "no hub id configured"] {
        assert!(missing.contains(needle), "missing {needle:?} in {missing}");
    }
    assert_eq!(v["strength"]["per_agent_accounts"], false);
    assert_eq!(v["strength"]["operator_key"], false);
    // A pinned hub id is named and no longer listed as missing.
    let p = rig_with(|c| c.hub_id = Some("hub-abc123".into()));
    let v = call(&p, Scope::Status, json!({"op": "status"}));
    assert_eq!(v["hub_id"], "hub-abc123");
    assert!(!v["missing_strength"].to_string().contains("no hub id configured"));
    assert_eq!(v["strength"]["hub_id_pinned"], true);
    assert_eq!(v["stages"], json!(["SENT", "RECEIVED", "STORED", "HANDED_OVER", "REPLIED"]));
}

#[test]
fn api_status_never_says_delivered_without_handed_over() {
    let r = rig();
    r.store.put("fp-s", "c1", "m1", b"hello").unwrap();
    r.store.append_event("c1", "RECEIVED", "m2", "fp-s", "d").unwrap();
    r.store.append_event("c1", "CLAIMED", "m1", "fp-s", "d").unwrap();
    let all = call(&r, Scope::Status, json!({"op": "status"}));
    let text = all.to_string().to_lowercase();
    assert!(!text.contains("delivered"), "{text}");
    assert!(!text.contains("handed_over\"}") || text.contains("stages"));
    let m1 = all["messages"].as_array().unwrap().iter().find(|m| m["msg_id"] == "m1").unwrap().clone();
    assert_eq!(m1["stage"], "STORED");
    assert!(m1.get("delivered").is_none());
    // Once the chain holds HANDED_OVER for m1, and only then, m1 is marked delivered.
    r.store.append_event("c1", "HANDED_OVER", "m1", "fp-s", "d").unwrap();
    let all = call(&r, Scope::Status, json!({"op": "status"}));
    let msgs = all["messages"].as_array().unwrap();
    let m1 = msgs.iter().find(|m| m["msg_id"] == "m1").unwrap();
    assert_eq!((m1["stage"].as_str(), m1["delivered"].as_bool()), (Some("HANDED_OVER"), Some(true)));
    let m2 = msgs.iter().find(|m| m["msg_id"] == "m2").unwrap();
    assert!(m2.get("delivered").is_none());
    // Filtering by msg_id keeps the rule.
    let one = call(&r, Scope::Status, json!({"op": "status", "msg_id": "m2"}));
    assert!(!one.to_string().to_lowercase().contains("delivered"));
    assert_eq!(one["rows"].as_array().unwrap().len(), 1);
}

#[test]
fn api_status_reports_chain_head_and_first_break() {
    let r = rig();
    for i in 0..3 {
        r.store.put("fp-s", "c1", &format!("m{i}"), b"x").unwrap();
    }
    let v = call(&r, Scope::Status, json!({"op": "status", "conv": "c1"}));
    assert_eq!(v["chains"]["c1"]["intact"], true);
    assert_eq!(v["chains"]["c1"]["len"], 3);
    assert_eq!(v["chains"]["c1"]["head"].as_str().unwrap().len(), 64);
    // Edit a middle line: status names the break.
    let log = r.store.dir().join("log/c1.jsonl");
    let text = fs::read_to_string(&log).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines[1] = lines[1].replace("m1", "mX");
    fs::write(&log, lines.join("\n") + "\n").unwrap();
    let v = call(&r, Scope::Status, json!({"op": "status", "conv": "c1"}));
    assert_eq!(v["chains"]["c1"]["intact"], false);
    assert_eq!(v["chains"]["c1"]["first_break_seq"], 1);
}

#[test]
fn api_shutdown_removes_the_sockets() {
    let r = rig();
    let paths: Vec<_> = Scope::ALL.iter().map(|s| r.api.path(*s).to_path_buf()).collect();
    r.api.shutdown();
    for p in paths {
        assert!(!p.exists());
    }
}

#[test]
fn api_attention_lists_every_entry_numbered_with_urgent_counted_apart() {
    let r = rig();
    r.store.attention("receive-stalled", "offset 4").unwrap();
    r.store.attention("urgent-overdue", "c/m7 urgent").unwrap();
    r.store.attention("receive-stalled", "offset 5").unwrap();
    let v = call(&r, Scope::Admin, json!({"op": "attention"}));
    assert_eq!((v["ok"].clone(), v["total"].clone(), v["urgent"].clone()), (json!(true), json!(3), json!(1)), "{v}");
    assert_eq!(v["by_kind"]["receive-stalled"], 2, "{v}");
    assert_eq!(v["entries"][1]["n"], 2);
    assert_eq!(v["entries"][1]["urgent"], true);
    assert!(v["entries"][0]["ts_ms"].as_u64().is_some(), "entries are timed: {v}");
    // `since` skips what a reader already showed.
    let w = call(&r, Scope::Admin, json!({"op": "attention", "since": 2}));
    assert_eq!((w["total"].clone(), w["entries"].as_array().unwrap().len()), (json!(3), 1), "{w}");
    // Not reachable from the agent's own socket.
    let x = call(&r, Scope::Send, json!({"op": "attention"}));
    assert!(x.get("code").is_some(), "{x}");
}
