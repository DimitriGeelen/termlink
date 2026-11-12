//! T-3694 (B-25, K-34 / E-28 wiring): the waiting-mail signal runs inside a real sidecar.
//! A started sidecar (test estate, its own hub) with a main project stores one message; the
//! daemon's waiting-mail tick, driven with the test's clock, shows `main.mail_waiting` in
//! `status` and writes one needs-attention entry per episode. Negative control: at 9 min and
//! for a sidecar without a main project nothing is raised. The background tick is frozen
//! through the test-estate seam so only the test's clock drives it.
//! One test in this file: it sets a process-wide environment variable.
use serde_json::{Value, json};
use std::time::Duration;
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::daemon::{self, Config};
use termlink_sidecar::hook;
use termlink_sidecar::store::now_ms;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Msg;

const AGENT: &str = "claude-mw";
const INSTANCE: &str = "claude-mw.w1-test";
const MIN: u64 = 60_000;

fn status(e: &Estate) -> Value {
    hook::call(&api::socket_path(&e.runtime_dir(), AGENT, Scope::Status), &json!({"op": "status"}), Duration::from_secs(20)).unwrap()
}

fn kinds(store: &termlink_sidecar::store::Store) -> usize {
    store
        .attention_entries()
        .iter()
        .filter(|l| serde_json::from_str::<Value>(l).is_ok_and(|v| v["kind"] == termlink_sidecar::mail_waiting::ATTENTION_KIND))
        .count()
}

#[test]
fn e28_wired_status_shows_mail_waiting_and_one_attention_entry_per_episode() {
    // SAFETY: the only test in this binary; set before any sidecar thread starts.
    unsafe { std::env::set_var("TERMLINK_SIDECAR_TEST_FREEZE", "mail_waiting") };
    let e = Estate::create().unwrap();
    let ka = e.identity("peer").unwrap();
    let cfg = |project: Option<&str>| Config {
        agent: AGENT.into(),
        instance: INSTANCE.into(),
        hub: e.hub_addr(),
        hub_id: e.hub_id().unwrap(),
        key_file: e.root().join("keys").join("mw.key"),
        roster: vec![ka.public_key_hex()],
        queue_path: e.root().join("outbox.sqlite"),
        poll_ms: 200,
        timeout_ms: 2_000,
        claude_projects: None,
        key_register: None,
        book: Default::default(),
        main_project: project.map(str::to_string),
        enc_key_register: None,
    };
    let running = daemon::start(&cfg(Some("p-mw")), &e.store_root(), &e.runtime_dir(), &e.home()).unwrap();
    let mw = running.mail_waiting.clone().expect("a main project gives a waiting-mail runner");

    let m = Msg {
        from: ka.fingerprint().to_owned(),
        from_agent: "peer".into(),
        from_instance: "peer.w1-test".into(),
        to_agent: AGENT.into(),
        to_instance: INSTANCE.into(),
        conv: "c-mw".into(),
        msg_id: "m-mw-1".into(),
        seq: 0,
        priority: "0".into(),
        ts_ms: 1,
        body: "hello main".into(),
    };
    running.store.append_event(&m.conv, "RECEIVED", &m.msg_id, &m.from, &m.digest()).unwrap();
    running.store.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()).unwrap();
    let waiting = running.deliverer.waiting_mail(now_ms());
    assert_eq!(waiting.len(), 1, "the stored message is waiting: {waiting:?}");
    let t0 = waiting[0].stored_ms;

    assert_eq!(status(&e)["main"]["mail_waiting"]["state"], "unknown", "before the first tick: unknown, not null");
    // Control: 9 min is not yet waiting.
    daemon::mail_waiting_tick(&mw, &running.deliverer, &running.lease_view, &running.store, t0 + 9 * MIN);
    assert!(status(&e)["main"]["mail_waiting"].is_null(), "nothing at 9 min");
    assert_eq!(kinds(&running.store), 0);
    // 11 min, twice: raised once.
    daemon::mail_waiting_tick(&mw, &running.deliverer, &running.lease_view, &running.store, t0 + 11 * MIN);
    daemon::mail_waiting_tick(&mw, &running.deliverer, &running.lease_view, &running.store, t0 + 12 * MIN);
    let s = status(&e);
    let w = &s["main"]["mail_waiting"];
    assert_eq!(w["count"], 1, "{s}");
    assert!(w["oldest_age_s"].as_u64().is_some_and(|a| a >= 11 * 60), "{w}");
    assert_eq!(kinds(&running.store), 1, "one needs-attention entry per episode");
    // 31 min: no out-of-band channel exists, so `not available`, never "sent".
    daemon::mail_waiting_tick(&mw, &running.deliverer, &running.lease_view, &running.store, t0 + 31 * MIN);
    assert_eq!(status(&e)["main"]["mail_waiting"]["oob"], "not available");
    drop(running);

    // Control: without a main project there is no runner and status says unknown.
    let e2 = Estate::create().unwrap();
    let mut c2 = cfg(None);
    c2.hub = e2.hub_addr();
    c2.hub_id = e2.hub_id().unwrap();
    c2.key_file = e2.root().join("keys").join("mw.key");
    c2.queue_path = e2.root().join("outbox.sqlite");
    let r2 = daemon::start(&c2, &e2.store_root(), &e2.runtime_dir(), &e2.home()).unwrap();
    assert!(r2.mail_waiting.is_none());
    assert_eq!(status(&e2)["main"]["mail_waiting"]["state"], "unknown");
    drop(r2);
}
