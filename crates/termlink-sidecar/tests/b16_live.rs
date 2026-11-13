//! B-16 (T-3697): the doorbell on the real store and a real TermLink PTY session.
//! E-4 (store part: ATTEMPTED never HANDED_OVER, still owed, HANDOVER_UNCONFIRMED once),
//! E-98 (the doorbell does not bypass the atomic claim), E-161 (the kill switch: off by
//! default, an unreadable flag is off, delivery-off stops it), and the session binding for
//! S1 and `command.notify`: the sidecar finds the TermLink session from `/proc`, mints a
//! notify-scope token, reads S1 and types the SI-1 line into a real shell (L4).
//! Hermetic: a test estate and temp dirs; no hub, no production path.

use serde_json::json;
use std::sync::Arc;
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::deliver::Deliverer;
use termlink_sidecar::doorbell::{self, ATTEMPTED, HANDOVER_UNCONFIRMED, SWITCH_FLAG};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{Msg, Roster};

const B: &str = "agent-b";

struct Rig {
    _e: Estate,
    store: Arc<Store>,
    d: Arc<Deliverer>,
    ka: termlink_session::agent_identity::Identity,
}

fn rig() -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let ka = e.identity("a").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let d = Arc::new(Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(roster)), None));
    Rig { _e: e, store, d, ka }
}

fn put(r: &Rig, conv: &str, id: &str) {
    let m = Msg {
        from: r.ka.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        from_instance: "inst-a".into(),
        to_agent: B.into(),
        to_instance: "inst-b".into(),
        conv: conv.into(),
        msg_id: id.into(),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: format!("body of {id}"),
    };
    r.store.append_event(conv, "RECEIVED", id, &m.from, &m.digest()).unwrap();
    r.store.put(&m.from, conv, id, &m.payload_bytes()).unwrap();
}

fn id(n: u64) -> String {
    format!("{n:032x}")
}

/// E-4 (store part): the typed line is ATTEMPTED, the message stays owed, never HANDED_OVER.
#[test]
fn e4_store_attempted_is_not_handed_over() {
    let r = rig();
    put(&r, "c1", &id(1));
    let p = r.d.doorbell_pending(termlink_sidecar::store::now_ms(), 1_000_000);
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].attempts, 0);
    assert!(r.d.mark_stage(&id(1), ATTEMPTED, "via=doorbell ring_id=r1").unwrap());
    let p = r.d.doorbell_pending(termlink_sidecar::store::now_ms(), 1_000_000);
    assert_eq!(p.len(), 1, "still owed after a ring");
    assert_eq!(p[0].attempts, 1, "a typed line is an attempt (FCW-199)");
    assert!(!r.d.is_handed(&id(1)));
    let lines = r.store.chain_lines("c1").unwrap();
    assert!(lines.iter().any(|l| l.event == ATTEMPTED && l.reason.as_deref() == Some("via=doorbell ring_id=r1")));
    assert!(!lines.iter().any(|l| l.event == "HANDED_OVER"));
    assert!(r.d.mark_stage(&id(1), HANDOVER_UNCONFIRMED, "x").unwrap());
    assert!(!r.d.mark_stage(&id(1), HANDOVER_UNCONFIRMED, "x").unwrap(), "labelled once");
    // The hook pull still hands it over by the built rules (the evidence path decides).
    let v = api::dispatch_leased(&json!({"op": "pull", "session": "s1", "hook": "post-tool-use", "tool_use_id": "t1"}), Scope::Send, &r.store, B, None, Some(&r.d), None);
    assert!(v["frame"].is_string(), "{v}");
}

/// E-98 (re-run with B-16): a message with an open claim is never rung; the doorbell has no
/// claim or hand-over path of its own.
#[test]
fn e98_open_claim_is_not_rung() {
    let r = rig();
    put(&r, "c1", &id(1));
    put(&r, "c1", &id(2));
    r.d.claim_one(&id(1)).unwrap();
    let p = r.d.doorbell_pending(termlink_sidecar::store::now_ms(), 1_000_000);
    let ids: Vec<&str> = p.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, vec![id(2).as_str()]);
    let src = include_str!("../src/doorbell.rs");
    assert!(!src.contains(".claim(") && !src.contains("claim_one") && !src.contains("\"HANDED_OVER\""), "the doorbell never claims or hands over");
}

/// E-161 (B-16 part) and NID: off by default; an unreadable or odd flag is off; delivery-off stops it.
#[test]
fn e161_kill_switch_reads() {
    let r = rig();
    assert!(!doorbell::switch_on(&r.store), "OFF by default until the W2 go");
    let flag = r.store.dir().join(SWITCH_FLAG);
    std::fs::create_dir(&flag).unwrap();
    assert!(!doorbell::switch_on(&r.store), "a flag that is not a readable file is off");
    std::fs::remove_dir(&flag).unwrap();
    std::os::unix::fs::symlink("/nonexistent", &flag).unwrap();
    assert!(!doorbell::switch_on(&r.store), "a dangling link is off");
    std::fs::remove_file(&flag).unwrap();
    std::fs::write(&flag, "").unwrap();
    assert!(doorbell::switch_on(&r.store));
    std::fs::write(r.store.dir().join(termlink_sidecar::policy::DELIVERY_OFF_FLAG), "").unwrap();
    assert!(!doorbell::switch_on(&r.store), "delivery-off stops the doorbell too (SI-23)");
    put(&r, "c1", &id(1));
    let v = api::dispatch_leased(&json!({"op": "pull", "session": "s1", "hook": "post-tool-use"}), Scope::Send, &r.store, B, None, Some(&r.d), None);
    assert_eq!(v["why"], "delivery-off", "{v}");
}

/// Session binding (S1 and `command.notify`), live: a real PTY session hosting `/bin/sh` as
/// the harness. The Runner binds it from `/proc` and the session registry, reads S1 with a
/// minted notify token, and after 30 s types the SI-1 line, which the shell refuses (127).
/// Linux only: binding reads `/proc`, which macOS lacks; macOS binding is T-3770 (not built yet).
#[cfg(target_os = "linux")]
#[test]
fn live_binding_reads_s1_and_types_the_si1_line() {
    use termlink_session::auth;
    use termlink_session::handler::SessionContext;
    use termlink_session::identity::SessionId;
    use termlink_session::lifecycle::SessionState;
    use termlink_session::registration::{Registration, SessionConfig};

    let r = rig();
    let dir = r.store.dir().parent().unwrap().join(format!("tl-sessions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sid = SessionId::generate();
    let sock = dir.join(format!("{sid}.sock"));
    let secret = auth::generate_secret();
    let mut reg = Registration::new(sid.clone(), SessionConfig { display_name: Some("b16".into()), capabilities: vec![], roles: vec![], tags: vec![] }, sock.clone());
    reg.state = SessionState::Ready;
    reg.token_secret = Some(secret.iter().map(|b| format!("{b:02x}")).collect());
    reg.write_atomic(&dir.join(format!("{sid}.json"))).unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let pty = rt.block_on(async { Arc::new(termlink_session::pty::PtySession::spawn(Some("/bin/sh"), 65536).unwrap()) });
    let sh_pid = pty.child_pid();
    let ctx = SessionContext::with_pty(reg.clone(), pty.clone());
    let shared = Arc::new(tokio::sync::RwLock::new(ctx));
    let listener = rt.block_on(async { tokio::net::UnixListener::bind(&sock).unwrap() });
    rt.spawn(termlink_session::server::run_accept_loop(listener, shared));
    // Let the PTY's read loop collect output into the scrollback.
    {
        let p = pty.clone();
        rt.spawn(async move {
            let _ = p.read_loop().await;
        });
    }

    // The binding the Runner will find: the session server (this process) is an ancestor of the shell.
    let regs = termlink_session::manager::list_sessions_in(&dir, false).unwrap();
    let b = doorbell::find_binding(std::path::Path::new("/proc"), sh_pid, &regs).expect("bound from /proc");
    assert_eq!(b.0, sock);

    let meter = Arc::new(termlink_sidecar::meter::Meter::new(B, Box::new(termlink_sidecar::meter::Unbound), Box::new(termlink_sidecar::meter::NoLedger)));
    let rpc = Arc::new(termlink_sidecar::meter::SessionRpc::new().unwrap());
    let meter = {
        // The meter reads S1 through the same SessionRpc the doorbell binds.
        drop(meter);
        Arc::new(termlink_sidecar::meter::Meter::new(B, Box::new(rpc.clone()), Box::new(termlink_sidecar::meter::NoLedger)))
    };
    // T-3772: read the harness only after the child exec'd (else a loaded host races to the test binary's exe).
    let h = termlink_sidecar::testkit::harness_after_exec(sh_pid);
    let mut runner = doorbell::Runner::new(B, r.store.clone(), r.d.clone(), meter.clone(), rpc.clone(), None).with_sessions_dir(dir.clone());
    runner.registry = vec![(h.exe.clone().unwrap(), h.script.clone())];
    let report = |ev: &str, extra: serde_json::Value| {
        let mut v = json!({"schema": "termlink.adapter/2", "event": ev, "agent_id": B, "session_id": "cc-1", "at_ms": 1});
        for (k, x) in extra.as_object().unwrap() {
            v[k] = x.clone();
        }
        meter.intake(v.to_string().as_bytes())
    };
    report("session_start", json!({"interactive": true, "harness_pid": sh_pid}));
    report("prompt_submit", json!({"origin": "automated", "basis": "harness-mark"}));
    report("ready", json!({"state": "READY"}));
    put(&r, "c1", &id(7));

    // Off by default: nothing at all.
    assert!(runner.tick().is_none());
    std::fs::write(r.store.dir().join(SWITCH_FLAG), "").unwrap();
    runner.bind(); // T-3772: binding runs on the meter's tick path, not inside the doorbell tick
    let first = runner.tick().unwrap();
    assert!(rpc.is_bound("cc-1"), "bound for S1 and notify");
    assert_eq!(first.decisions[0].1, "hold:nothing", "mail younger than a tick waits for the hook path");
    meter.tick();
    let f = &meter.doorbell_facts(meter.now())[0];
    assert!(f.s1.is_some(), "S1 read through the bound session with a minted notify token");

    std::thread::sleep(std::time::Duration::from_secs(31));
    // As the daemon's loop: a tick whose S1 read missed its budget on a loaded host
    // holds (S1 unknown is typing, FCW-22) and the next tick decides again.
    let mut reps = vec![];
    for _ in 0..5 {
        meter.tick();
        let rep = runner.tick().unwrap();
        let done = rep.rung.is_some();
        reps.push(rep);
        if done {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let rep = reps.last().unwrap();
    assert!(matches!(rep.rung, Some((_, doorbell::RingOut::Done { notified: doorbell::Notified::Typed, .. }))), "{reps:?}");
    // The shell received the SI-1 line.
    let want = doorbell::compose(&[id(7)], 1).unwrap();
    let mut seen = String::new();
    for _ in 0..50 {
        seen = rt.block_on(async { String::from_utf8_lossy(&pty.scrollback().lock().await.last_n_bytes(65536)).into_owned() });
        if seen.contains(&want) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(seen.contains(&want), "the PTY shows the line: {seen:?}");
    let counters = pty.input_meter().sources_json();
    assert!(counters["notify"]["last_mono"].is_u64() && counters["client"]["last_mono"].is_null(), "{counters}");
    // ATTEMPTED on the chain, the ring log has intent and outcome.
    assert!(r.store.chain_lines("c1").unwrap().iter().any(|l| l.event == ATTEMPTED));
    let log = doorbell::read_log(&runner.ring_log());
    assert!(log.iter().any(|v| v["phase"] == "intent") && log.iter().any(|v| v["phase"] == "outcome" && v["outcome"] == "typed"));
    let st = runner.status_view();
    assert_eq!(st["switch"], "on");
    assert_eq!(st["sessions"]["cc-1"]["state"], "await-submit");
    rt.shutdown_background();
}
