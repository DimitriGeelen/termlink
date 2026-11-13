//! T-3772: session binding (S1, `command.notify`) runs on the meter's tick path whatever
//! the doorbell switch says. A meter-only sidecar (doorbell off, the T-3769 recipe) binds the
//! harness session to the TermLink PTY session that hosts it and reads S1, so the E-153
//! receiver-side origin check can run; with the doorbell off nothing is ever typed or rung.
//! An unbound session names why, and where binding cannot run (no `/proc`, macOS until
//! T-3770) status says unavailable, never bound.
//! Hermetic: a test estate and temp dirs; no hub, no production path.

use serde_json::json;
use std::sync::Arc;
use termlink_sidecar::daemon::meter_and_doorbell_tick;
use termlink_sidecar::deliver::Deliverer;
use termlink_sidecar::doorbell::{self, BINDING_UNAVAILABLE, NO_HARNESS_PID, SWITCH_FLAG};
use termlink_sidecar::meter::{Meter, NoLedger, SessionRpc};
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
    meter: Arc<Meter>,
    rpc: Arc<SessionRpc>,
}

fn rig() -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let ka = e.identity("a").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let d = Arc::new(Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(roster)), None));
    // The meter reads S1 through the same SessionRpc the binding fills (as the daemon wires it).
    let rpc = Arc::new(SessionRpc::new().unwrap());
    let meter = Arc::new(Meter::new(B, Box::new(rpc.clone()), Box::new(NoLedger)));
    Rig { _e: e, store, d, ka, meter, rpc }
}

fn runner(r: &Rig) -> doorbell::Runner {
    doorbell::Runner::new(B, r.store.clone(), r.d.clone(), r.meter.clone(), r.rpc.clone(), None)
}

fn report(m: &Meter, session: &str, ev: &str, extra: serde_json::Value) {
    let mut v = json!({"schema": "termlink.adapter/2", "event": ev, "agent_id": B, "session_id": session, "at_ms": 1});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    m.intake(v.to_string().as_bytes());
}

fn s1_of(m: &Meter, session: &str) -> String {
    m.status_view()["sessions"][session]["s1"].as_str().unwrap_or("").to_owned()
}

/// Where binding cannot run (no `<proc>/self`: the macOS shape), it says so and never binds.
/// Runs on every platform: the proc root is an empty directory.
#[test]
fn no_proc_reads_binding_unavailable_never_bound() {
    let r = rig();
    let empty = r.store.dir().parent().unwrap().join(format!("no-proc-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let run = runner(&r).with_proc_root(empty);
    report(&r.meter, "cc-1", "session_start", json!({"interactive": true, "harness_pid": std::process::id()}));
    assert!(!run.binding_available());
    assert!(meter_and_doorbell_tick(&r.meter, &run).is_none(), "doorbell off: no doorbell tick");
    assert!(!r.rpc.is_bound("cc-1"));
    let s1 = s1_of(&r.meter, "cc-1");
    assert!(s1.starts_with("unknown") && s1.contains("binding unavailable"), "S1: {s1}");
    let st = run.status_view();
    assert_eq!(st["binding"]["available"], false, "{st}");
    assert_eq!(st["binding"]["why"], BINDING_UNAVAILABLE);
    let sess = st["binding"]["sessions"]["cc-1"].as_str().unwrap();
    assert!(sess.starts_with("unbound") && sess.contains("binding unavailable"), "{sess}");
    assert_eq!(st["switch"], "off");
}

/// An unbound session names its cause: no harness pid reported (a session started before the
/// adapter was installed), or no TermLink session hosting the pid. Every platform: fake proc root.
#[test]
fn unbound_session_names_why() {
    let r = rig();
    let fake = r.store.dir().parent().unwrap().join(format!("fake-proc-{}", std::process::id()));
    std::fs::create_dir_all(fake.join("self")).unwrap();
    // pid 4242: a process whose parent is init, so nothing above it hosts it.
    std::fs::create_dir_all(fake.join("4242")).unwrap();
    std::fs::write(fake.join("4242").join("stat"), "4242 (claude) S 1 4242 4242 0").unwrap();
    let run = runner(&r).with_proc_root(fake);
    report(&r.meter, "old", "session_start", json!({"interactive": true}));
    report(&r.meter, "cc-2", "session_start", json!({"interactive": true, "harness_pid": 4242}));
    assert!(run.binding_available());
    meter_and_doorbell_tick(&r.meter, &run);
    assert!(!r.rpc.is_bound("old") && !r.rpc.is_bound("cc-2"));
    assert_eq!(r.rpc.why_unbound("old").as_deref(), Some(NO_HARNESS_PID));
    assert!(s1_of(&r.meter, "old").contains("no harness_pid"), "{}", s1_of(&r.meter, "old"));
    let why = r.rpc.why_unbound("cc-2").unwrap();
    assert!(why.contains("no TermLink session hosts harness pid 4242"), "{why}");
    let st = run.status_view();
    assert_eq!(st["binding"]["available"], true);
    assert!(st["binding"]["why"].is_null());
    assert!(st["binding"]["sessions"]["cc-2"].as_str().unwrap().contains("hosts harness pid 4242"), "{st}");
}

/// The bug (T-3772): with the doorbell OFF, the daemon's tick binds a harness session running in
/// a real TermLink PTY and the meter reads S1. Negative control: with the doorbell off nothing is
/// typed or rung, even with mail owed long enough to ring. Positive control: switching it on
/// types the SI-1 line into the same PTY, so the off-phase silence is the switch, not the rig.
/// Linux only: binding reads `/proc`; macOS binding is T-3770.
#[cfg(target_os = "linux")]
#[test]
fn t3772_doorbell_off_binds_and_reads_s1() {
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
    let mut reg = Registration::new(sid.clone(), SessionConfig { display_name: Some("t3772".into()), capabilities: vec![], roles: vec![], tags: vec![] }, sock.clone());
    reg.state = SessionState::Ready;
    reg.token_secret = Some(secret.iter().map(|b| format!("{b:02x}")).collect());
    reg.write_atomic(&dir.join(format!("{sid}.json"))).unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let pty = rt.block_on(async { Arc::new(termlink_session::pty::PtySession::spawn(Some("/bin/sh"), 65536).unwrap()) });
    let sh_pid = pty.child_pid();
    let shared = Arc::new(tokio::sync::RwLock::new(SessionContext::with_pty(reg.clone(), pty.clone())));
    let listener = rt.block_on(async { tokio::net::UnixListener::bind(&sock).unwrap() });
    rt.spawn(termlink_session::server::run_accept_loop(listener, shared));
    {
        let p = pty.clone();
        rt.spawn(async move {
            let _ = p.read_loop().await;
        });
    }

    let h = termlink_sidecar::testkit::harness_after_exec(sh_pid);
    let mut run = runner(&r).with_sessions_dir(dir.clone());
    run.registry = vec![(h.exe.clone().unwrap(), h.script.clone())];
    report(&r.meter, "cc-1", "session_start", json!({"interactive": true, "harness_pid": sh_pid}));
    report(&r.meter, "cc-1", "prompt_submit", json!({"origin": "automated", "basis": "harness-mark"}));
    report(&r.meter, "cc-1", "ready", json!({"state": "READY"}));
    let m = Msg {
        from: r.ka.fingerprint().to_owned(),
        from_agent: "agent-a".into(),
        from_instance: "inst-a".into(),
        to_agent: B.into(),
        to_instance: "inst-b".into(),
        conv: "c1".into(),
        msg_id: format!("{:032x}", 7),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: "owed".into(),
    };
    r.store.append_event("c1", "RECEIVED", &m.msg_id, &m.from, &m.digest()).unwrap();
    r.store.put(&m.from, "c1", &m.msg_id, &m.payload_bytes()).unwrap();
    let want = doorbell::compose(std::slice::from_ref(&m.msg_id), 1).unwrap();
    let scroll = || rt.block_on(async { String::from_utf8_lossy(&pty.scrollback().lock().await.last_n_bytes(65536)).into_owned() });

    // Doorbell OFF (no `doorbell-on`): one daemon tick binds and the meter reads S1.
    assert!(!r.store.dir().join(SWITCH_FLAG).exists());
    assert!(meter_and_doorbell_tick(&r.meter, &run).is_none(), "the doorbell tick does nothing while off");
    assert!(r.rpc.is_bound("cc-1"), "bound with the doorbell off: {:?}", r.rpc.why_unbound("cc-1"));
    let f = &r.meter.doorbell_facts(r.meter.now())[0];
    assert!(f.s1.is_some(), "S1 read through the bound session, doorbell off: {}", s1_of(&r.meter, "cc-1"));
    assert_eq!(s1_of(&r.meter, "cc-1"), "known");
    let st = run.status_view();
    assert_eq!(st["switch"], "off");
    assert_eq!(st["binding"]["available"], true);
    assert_eq!(st["binding"]["sessions"]["cc-1"], "bound", "{st}");

    // Negative control: mail owed past the ring delay, several ticks, still nothing typed or rung.
    std::thread::sleep(std::time::Duration::from_secs(31));
    for _ in 0..3 {
        assert!(meter_and_doorbell_tick(&r.meter, &run).is_none());
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    assert!(!scroll().contains(&want), "doorbell off typed the SI-1 line");
    let counters = pty.input_meter().sources_json();
    assert!(counters["notify"]["last_mono"].is_null(), "doorbell off: no notify keystroke: {counters}");
    assert!(!run.ring_log().exists(), "doorbell off: no ring log");
    assert!(!r.d.is_handed(&m.msg_id));

    // Positive control: the same rig with the switch on types the line.
    std::fs::write(r.store.dir().join(SWITCH_FLAG), "").unwrap();
    let mut typed = false;
    let mut seen_s1 = vec![];
    for _ in 0..5 {
        let rep = meter_and_doorbell_tick(&r.meter, &run).expect("switch on: a doorbell tick");
        seen_s1.push((s1_of(&r.meter, "cc-1"), rep.decisions.clone(), format!("{:?}", rep.rung)));
        if matches!(rep.rung, Some((_, doorbell::RingOut::Done { notified: doorbell::Notified::Typed, .. }))) {
            typed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    assert!(typed, "switch on: the rig types; per tick (S1, decisions, rung): {seen_s1:?}");
    let mut seen = String::new();
    for _ in 0..50 {
        seen = scroll();
        if seen.contains(&want) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(seen.contains(&want), "the PTY shows the line once switched on: {seen:?}");
    rt.shutdown_background();
}
