//! E-153 (T-3694, B-25 with B-27; CR-5.4, CR-6.4, FCW-195): a `human` claim with no client
//! bytes behind it never renews main. The adapter reports every prompt `human/inferred`; the
//! receiver checks it against the session's PTY input counters (fake S1, simulated clock) on
//! the real send socket, and the lease plan is decided with the origin the reply carries
//! (`hook::checked_origin`), for a session that holds main with `last_human` 6 min old.
//!   (a) the doorbell typed a line, submit reported human with no doorbell_ids: automated
//!   (b) a `/loop` submit with no client byte since the previous submit: automated
//!   (c) the sidecar unreachable: the claim carries unknown
//!   (d) control: a person typed (client bytes) and submitted: human, Touch + renewal
//! `client_auto` (terminal replies) never counts as a client byte.
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termlink_session::main_lease::{self as ml, ClaimKind, ClaimRequest, Event, Holder, Logged, Plan, Row, Timing};
use termlink_sidecar::api::{self, Api, ApiConfig, Scope};
use termlink_sidecar::meter::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const A: &str = "agent-e153";
const MIN: u64 = 60_000;

#[derive(Default)]
struct FakeS1(Mutex<Option<S1Reply>>);
struct S1Box(Arc<FakeS1>);
impl S1Source for S1Box {
    fn read(&self, _: &str) -> Result<S1Reply, String> {
        self.0.0.lock().unwrap().clone().ok_or("unbound".into())
    }
}
struct NoLedger;
impl Ledger for NoLedger {
    fn owed(&self, _: &str, _: u64) -> Result<bool, String> {
        Ok(false)
    }
}

fn s1(client: Option<u64>, client_auto: Option<u64>, notify: Option<u64>) -> S1Reply {
    let mut last = BTreeMap::new();
    last.insert("client".into(), client);
    last.insert("client_auto".into(), client_auto);
    last.insert("notify".into(), notify);
    last.insert("inject".into(), None);
    S1Reply { boot_id: termlink_session::input_meter::boot_id(), last, foreground: Value::Null }
}

/// The lease plan for session `S` (holder, human acquire 6 min ago) claiming with `origin`.
fn plan_with(origin: &str, basis: &str) -> Plan {
    let me = Holder { session: "S".into(), agent: A.into(), host: "h".into(), pid: 1 };
    let acq = Logged {
        offset: 1,
        event: Event::Acquire {
            generation: 1,
            holder: me.clone(),
            claim_id: "c1".into(),
            at_ms: 0,
            reason: "first".into(),
            from: None,
            lapsed_at_ms: None,
            obligations: vec![],
            origin: Some("human".into()),
            basis: Some("pty-input".into()),
            provisional: false,
            holder_human_age_s: None,
        },
        sender: Some("key-S".into()),
        hub_rx_ms: None,
    };
    let f = ml::fold(&[acq]);
    let row = Row { claim_id: "c1".into(), claimer: me.claimer(), claimed_at: 0, claimed_until: i64::MAX };
    let v = ml::with_origin(ml::view("p", &f, Some(&row), 6 * MIN, Timing::default()), &f, Some(&[]), Some(vec![ml::DEFAULT_HARNESS.into()]));
    let req = ClaimRequest { kind: ClaimKind::Prompt, origin: ml::Origin::parse(origin), basis: Some(basis.into()), ..ClaimRequest::default() };
    ml::plan_reporting(&v, &f, &me, &req, v.last_human_at_ms)
}

#[test]
fn e153_receiver_checked_origin_decides_the_claim() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), A).unwrap());
    let fake = Arc::new(FakeS1::default());
    let now = Arc::new(AtomicU64::new(1_000_000));
    let (n1, n2) = (now.clone(), now.clone());
    let meter = Arc::new(
        Meter::new(A, Box::new(S1Box(fake.clone())), Box::new(NoLedger))
            .with_clocks(Arc::new(move || n1.load(Ordering::SeqCst)), Arc::new(move || n2.load(Ordering::SeqCst))),
    );
    let mut cfg = ApiConfig::new(A, &e.runtime_dir());
    cfg.meter = Some(meter.clone());
    let _api = Api::start(cfg, store).unwrap();
    let sock = api::socket_path(&e.runtime_dir(), A, Scope::Send);
    let at = || now.load(Ordering::SeqCst);
    let adv = |ms: u64| now.fetch_add(ms, Ordering::SeqCst);
    let report = |n: u64| {
        json!({"schema": "termlink.adapter/2", "harness": "fake", "agent_id": A, "session_id": "S", "event": "prompt_submit",
               "at_ms": n, "origin": "human", "basis": "inferred"})
    };
    let check = |n: u64| termlink_sidecar::hook::checked_origin(&sock, &report(n), Duration::from_secs(10));
    let overridden = || meter.counters().origin_overridden;

    // (d) control first: a person typed the prompt (client bytes now) and submitted.
    *fake.0.lock().unwrap() = Some(s1(Some(at() - 5), None, None));
    let (o, b) = check(1);
    assert_eq!((o.as_str(), b.as_str()), ("human", "inferred"));
    assert!(matches!(plan_with(&o, &b), Plan::TouchRenew { .. }), "a checked human prompt renews (Touch)");

    // (b) a /loop submit 2 min later with no client byte since the previous submit: only a
    // terminal reply (client_auto) arrived, which is never a client byte.
    adv(2 * MIN);
    *fake.0.lock().unwrap() = Some(s1(Some(at() - 2 * MIN - 5), Some(at() - 1), None));
    let before = overridden();
    let (o, b) = check(2);
    assert_eq!((o.as_str(), b.as_str()), ("automated", "pty-input"), "(b) no client byte since the previous submit");
    assert_eq!(overridden(), before + 1, "origin-overridden counted");
    assert_eq!(plan_with(&o, &b), Plan::Ignored, "(b) no Touch, no renewal");

    // (a) the doorbell typed a line (receiver-typed, no doorbell_ids in the report); a person
    // typed before it but not after it.
    adv(2 * MIN);
    *fake.0.lock().unwrap() = Some(s1(Some(at() - 10), None, None));
    adv(1);
    meter.note_typed("S", &["m-9".into()]);
    adv(1);
    let before = overridden();
    let (o, b) = check(3);
    assert_eq!(o, "automated", "(a) follows a typed line with no client byte after it ({b})");
    assert_eq!(overridden(), before + 1);
    assert_eq!(plan_with(&o, &b), Plan::Ignored, "(a) no Touch, no renewal");

    // (c) the sidecar unreachable when the hook asks: the claim carries unknown.
    let gone = e.runtime_dir().join("no-such-sidecar.sock");
    let (o, b) = termlink_sidecar::hook::checked_origin(&gone, &report(4), Duration::from_secs(2));
    assert_eq!((o.as_str(), b.as_str()), ("unknown", "absent"));
    assert_eq!(plan_with(&o, &b), Plan::Ignored, "(c) unknown never renews");

    // The hook's own `human` is never what the claim carries: the plan with it would renew.
    assert!(matches!(plan_with("human", "inferred"), Plan::TouchRenew { .. }), "the check is what stops (a)-(c)");
}
