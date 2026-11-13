//! T-3770: session binding without `/proc`. On macOS the sidecar reads the parent chain and
//! the harness identity through libproc (`doorbell::MACOS_READER`); here, on any platform,
//! a stand-in reader drives the same path so the binding logic is proven where it runs.
//! The real libproc readers are type-checked against the macOS libc and exercised by
//! `termlink-session`'s `procinfo` tests on the macOS CI job.
//! Hermetic: a test estate and temp dirs; no hub, no production path.

use serde_json::json;
use std::sync::Arc;
use termlink_sidecar::daemon::meter_and_doorbell_tick;
use termlink_sidecar::deliver::Deliverer;
use termlink_sidecar::doorbell::{self, Harness, ProcReader, BINDING_UNAVAILABLE};
use termlink_sidecar::meter::{Meter, NoLedger, SessionRpc};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

const B: &str = "agent-b";
const HARNESS: u32 = 4242;
const MID: u32 = 4241;

struct Rig {
    _e: Estate,
    store: Arc<Store>,
    meter: Arc<Meter>,
    rpc: Arc<SessionRpc>,
    run: doorbell::Runner,
}

fn rig(reader: Option<ProcReader>, sessions: Option<std::path::PathBuf>) -> Rig {
    let e = Estate::create_without_hub().unwrap();
    let ka = e.identity("a").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let mut roster = Roster::new();
    roster.add(ka.verifying_key());
    let d = Arc::new(Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(roster)), None));
    let rpc = Arc::new(SessionRpc::new().unwrap());
    let meter = Arc::new(Meter::new(B, Box::new(rpc.clone()), Box::new(NoLedger)));
    // No `<proc>/self` here: the macOS shape. Only the native reader (if any) can bind.
    let empty = store.dir().parent().unwrap().join(format!("no-proc-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let mut run = doorbell::Runner::new(B, store.clone(), d, meter.clone(), rpc.clone(), None).with_proc_root(empty).with_native(reader);
    if let Some(s) = sessions {
        run = run.with_sessions_dir(s);
    }
    Rig { _e: e, store, meter, rpc, run }
}

fn report(m: &Meter, session: &str, harness_pid: u32) {
    let v = json!({"schema": "termlink.adapter/2", "event": "session_start", "agent_id": B, "session_id": session,
        "at_ms": 1, "interactive": true, "harness_pid": harness_pid});
    m.intake(v.to_string().as_bytes());
}

/// The stand-in table: harness 4242 -> 4241 -> this test process (which hosts the session).
fn chain_ppid(pid: u32) -> Option<u32> {
    match pid {
        HARNESS => Some(MID),
        MID => Some(std::process::id()),
        _ => None,
    }
}
fn chain_harness(pid: u32) -> Option<Harness> {
    (pid == HARNESS).then(|| Harness { pid, exe: Some("/usr/local/bin/claude".into()), script: None })
}
fn none_ppid(_: u32) -> Option<u32> {
    None
}
fn none_harness(_: u32) -> Option<Harness> {
    None
}
const CHAIN: ProcReader = ProcReader { name: "fake-libproc", ppid: chain_ppid, harness: chain_harness };
const DENIED: ProcReader = ProcReader { name: "fake-libproc", ppid: none_ppid, harness: none_harness };

fn session_dir_hosted_by_me(store: &Store) -> std::path::PathBuf {
    use termlink_session::identity::SessionId;
    use termlink_session::lifecycle::SessionState;
    use termlink_session::registration::{Registration, SessionConfig};
    let dir = store.dir().parent().unwrap().join(format!("tl-sessions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let sid = SessionId::generate();
    let sock = dir.join(format!("{sid}.sock"));
    let mut reg = Registration::new(sid.clone(), SessionConfig { display_name: Some("t3770".into()), capabilities: vec![], roles: vec![], tags: vec![] }, sock);
    assert_eq!(reg.pid, std::process::id(), "a registration is hosted by the process that writes it");
    reg.state = SessionState::Ready;
    reg.write_atomic(&dir.join(format!("{sid}.json"))).unwrap();
    // A live socket file, so the registry does not read the session as stale.
    let _l = std::os::unix::net::UnixListener::bind(dir.join(format!("{sid}.sock"))).unwrap();
    std::mem::forget(_l);
    dir
}

/// Without `/proc`, the native reader walks the parent chain to the hosting TermLink session.
#[test]
fn native_reader_binds_through_the_parent_chain() {
    let pre = rig(None, None);
    let dir = session_dir_hosted_by_me(&pre.store);
    let r = rig(Some(CHAIN), Some(dir));
    report(&r.meter, "cc-1", HARNESS);
    assert!(r.run.binding_available());
    assert_eq!(r.run.binding_source(), Some("fake-libproc"));
    meter_and_doorbell_tick(&r.meter, &r.run);
    assert!(r.rpc.is_bound("cc-1"), "bound via the native reader: {:?}", r.rpc.why_unbound("cc-1"));
    let st = r.run.status_view();
    assert_eq!(st["binding"]["available"], true, "{st}");
    assert_eq!(st["binding"]["source"], "fake-libproc");
    assert!(st["binding"]["why"].is_null());
    assert_eq!(st["binding"]["sessions"]["cc-1"], "bound", "{st}");
}

/// Negative control: the same chain with no hosting session registered is unbound and says so.
#[test]
fn native_reader_without_a_hosting_session_is_unbound_with_why() {
    let pre = rig(None, None);
    let empty_sessions = pre.store.dir().parent().unwrap().join(format!("tl-none-{}", std::process::id()));
    std::fs::create_dir_all(&empty_sessions).unwrap();
    let r = rig(Some(CHAIN), Some(empty_sessions));
    report(&r.meter, "cc-1", HARNESS);
    meter_and_doorbell_tick(&r.meter, &r.run);
    assert!(!r.rpc.is_bound("cc-1"));
    let why = r.rpc.why_unbound("cc-1").unwrap();
    assert!(why.contains("no TermLink session hosts harness pid 4242"), "{why}");
}

/// Negative control: a native reader that cannot read the harness (gone, or not permitted —
/// libproc returns nothing for another user's process) leaves it unbound with that reason,
/// never "bound" and never "no session hosts it".
#[test]
fn unreadable_harness_is_unbound_and_names_the_reader() {
    let pre = rig(None, None);
    let dir = session_dir_hosted_by_me(&pre.store);
    let r = rig(Some(DENIED), Some(dir));
    report(&r.meter, "cc-1", HARNESS);
    assert!(r.run.binding_available());
    meter_and_doorbell_tick(&r.meter, &r.run);
    assert!(!r.rpc.is_bound("cc-1"));
    let why = r.rpc.why_unbound("cc-1").unwrap();
    assert!(why.contains("harness pid 4242 unreadable via fake-libproc"), "{why}");
    let sess = r.run.status_view()["binding"]["sessions"]["cc-1"].as_str().unwrap().to_owned();
    assert!(sess.starts_with("unbound (") && sess.contains("unreadable via fake-libproc"), "{sess}");
}

/// No `/proc` and no native reader (neither Linux nor macOS): unavailable, never bound.
#[test]
fn no_reader_at_all_is_unavailable() {
    let r = rig(None, None);
    report(&r.meter, "cc-1", HARNESS);
    assert!(!r.run.binding_available());
    assert_eq!(r.run.binding_source(), None);
    meter_and_doorbell_tick(&r.meter, &r.run);
    assert!(!r.rpc.is_bound("cc-1"));
    let st = r.run.status_view();
    assert_eq!(st["binding"]["why"], BINDING_UNAVAILABLE);
    assert!(st["binding"]["source"].is_null());
}

/// `with_proc_root` means "read only from this directory": a native reader set before it is
/// dropped (else the T-3772 "no /proc" test would bind through libproc on a Mac).
#[test]
fn proc_root_seam_drops_an_earlier_native_reader() {
    let r = rig(None, None);
    let empty = r.store.dir().parent().unwrap().join(format!("no-proc2-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let d = Arc::new(Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(Roster::new())), None));
    let run = doorbell::Runner::new(B, store, d, r.meter.clone(), r.rpc.clone(), None).with_native(Some(CHAIN)).with_proc_root(empty);
    assert!(!run.binding_available());
    assert_eq!(run.binding_source(), None);
}

/// The platform default: Linux reads `/proc`, macOS reads libproc; both are available.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn default_runner_binding_is_available_on_linux_and_macos() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), B).unwrap());
    let d = Arc::new(Deliverer::new(B, store.clone(), Box::new(BuiltIn::new(Roster::new())), None));
    let rpc = Arc::new(SessionRpc::new().unwrap());
    let meter = Arc::new(Meter::new(B, Box::new(rpc.clone()), Box::new(NoLedger)));
    let run = doorbell::Runner::new(B, store, d, meter, rpc, None);
    assert!(run.binding_available());
    let want = if cfg!(target_os = "macos") { "libproc" } else { "/proc" };
    assert_eq!(run.binding_source(), Some(want));
}
