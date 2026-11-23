//! T-3854 (T-3694 finding, 2026-11-23 21:41Z): a generated prompt that arrives while the
//! operator only moves the mouse is not counted human. Claude Code turns on any-motion SGR
//! mouse reporting, so every movement writes `ESC[<35;x;yM` into the PTY input. Those writes
//! are classified `client_pointer` by the session's real `InputMeter`; the receiver's origin
//! check counts only `client` (typing) since the previous submit. The counters below come
//! from a real `InputMeter` fed real bytes, read back through `S1Reply::from_result`, and
//! the check runs on the sidecar's real send socket.
//!   (1) control: the operator typed and submitted: human
//!   (2) only mouse movement since that submit, then a `human` report: automated
//!   (3) control: typing again, then a submit: human
//!   (4) presence still sees the mouse (the doorbell's typing hold and the operator window)
use serde_json::{Value, json};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termlink_session::input_meter::{InputMeter, InputSource, boot_id};
use termlink_sidecar::api::{self, Api, ApiConfig, Scope};
use termlink_sidecar::meter::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const A: &str = "agent-t3854";
const MIN: u64 = 60_000;

struct LiveS1(Arc<Mutex<Option<S1Reply>>>);
impl S1Source for LiveS1 {
    fn read(&self, _: &str) -> Result<S1Reply, String> {
        self.0.lock().unwrap().clone().ok_or("unbound".into())
    }
}
struct NoLedger;
impl Ledger for NoLedger {
    fn owed(&self, _: &str, _: u64) -> Result<bool, String> {
        Ok(false)
    }
}

/// Write `bytes` as client input at `at` into the meter, classified as the PTY does.
fn write(m: &InputMeter, bytes: &[u8], at: u64) {
    m.record_at(InputMeter::classify(InputSource::Client, bytes), bytes.len(), at);
}

fn reply(m: &InputMeter) -> S1Reply {
    S1Reply::from_result(&json!({"boot_id": boot_id(), "sources": m.sources_json(), "foreground": Value::Null})).unwrap()
}

#[test]
fn t3854_mouse_movement_does_not_make_a_generated_prompt_human() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), A).unwrap());
    let s1 = Arc::new(Mutex::new(None));
    let now = Arc::new(AtomicU64::new(1_000_000));
    let (n1, n2) = (now.clone(), now.clone());
    let meter = Arc::new(
        Meter::new(A, Box::new(LiveS1(s1.clone())), Box::new(NoLedger))
            .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst))),
    );
    let mut cfg = ApiConfig::new(A, &e.runtime_dir());
    cfg.meter = Some(meter.clone());
    let _api = Api::start(cfg, store).unwrap();
    let sock = api::socket_path(&e.runtime_dir(), A, Scope::Send);
    let at = || now.load(Ordering::SeqCst);
    let adv = |ms: u64| now.fetch_add(ms, Ordering::SeqCst);
    let check = |n: u64| {
        let report = json!({"schema": "termlink.adapter/2", "harness": "fake", "agent_id": A, "session_id": "S",
                            "event": "prompt_submit", "at_ms": n, "origin": "human", "basis": "inferred"});
        termlink_sidecar::hook::checked_origin(&sock, &report, Duration::from_secs(10))
    };
    let pty = InputMeter::new();

    // (1) control: the operator typed a prompt and pressed Enter.
    write(&pty, b"status?", at() - 900);
    write(&pty, b"\r", at() - 5);
    *s1.lock().unwrap() = Some(reply(&pty));
    let (o, b) = check(1);
    assert_eq!((o.as_str(), b.as_str()), ("human", "inferred"), "(1) a typed prompt is human");

    // (2) 2 min later: only mouse movement (any-motion SGR reports, one and several per write),
    // then a background-task notification submits and the adapter reports it human/inferred.
    adv(2 * MIN);
    write(&pty, b"\x1b[<35;40;12M", at() - 30_000);
    write(&pty, b"\x1b[<35;41;12M\x1b[<35;42;13M\x1b[<35;44;13M", at() - 2);
    *s1.lock().unwrap() = Some(reply(&pty));
    let r = reply(&pty);
    assert_eq!(r.at("client"), Some(1_000_000 - 5), "mouse movement did not move client: {r:?}");
    let before = meter.counters().origin_overridden;
    let (o, b) = check(2);
    assert_eq!((o.as_str(), b.as_str()), ("automated", "pty-input"), "(2) mouse movement is not typing");
    assert_eq!(meter.counters().origin_overridden, before + 1, "(2) counted as overridden");

    // (3) control: the operator types again and submits.
    adv(MIN);
    write(&pty, b"go", at() - 400);
    write(&pty, b"\r", at() - 3);
    *s1.lock().unwrap() = Some(reply(&pty));
    let (o, b) = check(3);
    assert_eq!((o.as_str(), b.as_str()), ("human", "inferred"), "(3) typing is human");

    // (4) presence (doorbell typing hold, operator window) still sees the mouse.
    adv(MIN);
    write(&pty, b"\x1b[<35;10;10M", at() - 1);
    let r = reply(&pty);
    assert_eq!(r.presence(), Some(at() - 1), "(4) the mouse is presence");
    assert!(r.at("client") < r.presence(), "(4) but not typing");
}
