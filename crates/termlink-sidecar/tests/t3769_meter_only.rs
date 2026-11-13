//! T-3769: the meter-only sidecar recipe for the AEF adapter/2 trial
//! (docs/operations/sidecar-meter-only.md), proven end to end in a throwaway estate.
//!
//! The recipe is the ordinary `termlink-sidecar start --config F` with two flag files
//! written into the agent's store BEFORE the first start, and no third one:
//!   `delivery.off`    (SI-23)  nothing is handed to the session; the doorbell switch needs delivery on
//!   `receive.drained` (P4)     the receive loop pulls nothing from the hub
//!   no `doorbell-on`  (B-16)   the doorbell and its watchdog never ring or type
//!
//! Proven, against a real hub and the real binary, with a message for the meter agent
//! already waiting on the hub from a roster sender (so it WOULD be delivered):
//!   (1) adapter/2 report lines are accepted (`session_start`, `ready`)
//!   (2) a `prompt_submit` report is answered with `origin_checked` / `basis_checked`
//!   (3) `status` shows the meter's session data, delivery off, doorbell off, the lease view
//!   (4) nothing is received, nothing is pulled into the session, nothing is rung or typed
//! Negative control: the same running sidecar with delivery on and the drain lifted (the
//! admin socket, the operator's switch) receives the waiting message and `pull` hands it over.
use serde_json::{Value, json};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::daemon::Config;
use termlink_sidecar::hook;
use termlink_sidecar::testkit::Estate;

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");
const METER: &str = "aef-meter";
const PEER: &str = "agent-a";
const SESSION: &str = "11111111-2222-4333-8444-555555555555";

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
        let _ = self.0.wait();
    }
}

fn wait_for(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

fn config(e: &Estate, who: &str, name: &str, inst: &str, hub_id: &str, main_project: Option<&str>) -> Config {
    Config {
        agent: name.into(),
        instance: inst.into(),
        hub: e.hub_addr(),
        hub_id: hub_id.into(),
        key_file: e.root().join("keys").join(format!("{who}.key")),
        // The peer is on the meter's roster: its mail is own-project mail and WOULD be
        // delivered (the control below proves it), so only the flags keep it out.
        roster: ["a", "b"].iter().map(|w| e.identity(w).unwrap().public_key_hex()).collect(),
        queue_path: e.root().join(format!("outbox-{who}.sqlite")),
        poll_ms: 200,
        timeout_ms: 15_000,
        claude_projects: None,
        key_register: None,
        book: Default::default(),
        main_project: main_project.map(str::to_owned),
        enc_key_register: None,
    }
}

fn start(e: &Estate, cfg: &Config, tag: &str) -> Proc {
    let path = e.root().join(format!("sidecar-{tag}.json"));
    std::fs::write(&path, serde_json::to_string(cfg).unwrap()).unwrap();
    let log = std::fs::File::create(e.root().join(format!("sidecar-{tag}.log"))).unwrap();
    let child = e
        .command(BIN)
        .args(["start", "--config"])
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let sock = api::socket_path(&e.runtime_dir(), &cfg.agent, Scope::Send);
    assert!(wait_for(60, || sock.exists()), "{} never opened {}", cfg.agent, sock.display());
    Proc(child)
}

fn call(e: &Estate, agent: &str, scope: Scope, req: &Value) -> Value {
    hook::call(&api::socket_path(&e.runtime_dir(), agent, scope), req, Duration::from_secs(15)).unwrap()
}

fn report(event: &str, at_ms: u64, extra: Value) -> Value {
    let mut v = json!({"schema": "termlink.adapter/2", "harness": "claude-code", "agent_id": METER,
                       "session_id": SESSION, "event": event, "at_ms": at_ms});
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

fn peer_stage(e: &Estate, id: &str) -> String {
    call(e, PEER, Scope::Status, &json!({"op": "status", "msg_id": id}))["messages"][0]["stage"]
        .as_str()
        .unwrap_or("")
        .to_owned()
}

#[test]
fn meter_only_takes_reports_and_never_receives_delivers_rings_or_types() {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();

    // The recipe: the store dir with both flags, before the first start; no doorbell-on.
    let store_dir = e.store_root().join("sidecar").join(METER);
    std::fs::create_dir_all(&store_dir).unwrap();
    std::fs::write(store_dir.join("delivery.off"), "meter-only (T-3769)\n").unwrap();
    std::fs::write(store_dir.join("receive.drained"), "meter-only (T-3769)\n").unwrap();
    assert!(!store_dir.join("doorbell-on").exists());

    // A roster peer puts a message for the meter agent on the hub before it starts.
    let _peer = start(&e, &config(&e, "a", PEER, "inst-a", &hub_id, None), "peer");
    let sent = call(&e, PEER, Scope::Send, &json!({"op": "send", "to": METER, "to_instance": "inst-meter", "conv": "trial", "body": "T-3769 waiting mail"}));
    let id = sent["msg_id"].as_str().unwrap_or_else(|| panic!("peer send: {sent}")).to_owned();
    assert_eq!(sent["stage"], "SENT", "peer send: {sent}");

    let _meter = start(&e, &config(&e, "b", METER, "inst-meter", &hub_id, Some("trial-project")), "meter");

    // (1) reports accepted.
    let r = call(&e, METER, Scope::Send, &report("session_start", 1, json!({"interactive": true, "transcript_path": null})));
    assert_eq!(r["ok"], true, "session_start: {r}");
    let r = call(&e, METER, Scope::Send, &report("ready", 2, json!({"state": "READY"})));
    assert_eq!(r["ok"], true, "ready: {r}");
    // A malformed report is refused, not taken (the strict parse is live).
    let r = call(&e, METER, Scope::Send, &json!({"schema": "termlink.adapter/2", "event": "ready", "agent_id": METER}));
    assert_eq!(r["ok"], false, "malformed report must be ignored: {r}");

    // (2) a prompt report is answered with the receiver-checked origin.
    let r = call(&e, METER, Scope::Send, &report("prompt_submit", 3, json!({"origin": "human", "basis": "inferred"})));
    assert_eq!(r["ok"], true, "prompt_submit: {r}");
    let (o, b) = (r["origin_checked"].as_str(), r["basis_checked"].as_str());
    assert!(o.is_some() && b.is_some(), "prompt_submit must carry origin_checked/basis_checked: {r}");
    eprintln!("T-3769 prompt_submit reply: {r}");
    let (o2, _) = hook::checked_origin(
        &api::socket_path(&e.runtime_dir(), METER, Scope::Send),
        &report("prompt_submit", 4, json!({"origin": "human", "basis": "inferred"})),
        Duration::from_secs(10),
    );
    assert_ne!(o2, "unknown", "the hook helper reads the same origin_checked");

    // (3) status: meter data (after one meter tick, 30 s), both switches off, the lease view present.
    assert!(wait_for(90, || {
        let s = call(&e, METER, Scope::Status, &json!({"op": "status"}));
        s["activity"]["sessions"][SESSION].get("stamped_at").is_some()
    }), "the meter never ticked the reported session");
    let s = call(&e, METER, Scope::Status, &json!({"op": "status"}));
    eprintln!("T-3769 status.activity: {}", s["activity"]);
    eprintln!("T-3769 status.doorbell.switch: {}  delivery: {}  main.lease: {}", s["doorbell"]["switch"], s["delivery"], s["main"]["lease"]);
    assert_eq!(s["ok"], true);
    assert_eq!(s["delivery"], "off", "status.delivery: {s}");
    assert_eq!(s["doorbell"]["switch"], "off", "status.doorbell: {}", s["doorbell"]);
    assert!(s["activity"]["counters"].is_object());
    assert_ne!(s["main"]["lease"]["why"], "this sidecar reads no main lease", "the lease check is wired: {}", s["main"]["lease"]);

    // (4) nothing received, delivered, rung or typed: several receive polls and ticks pass.
    std::thread::sleep(Duration::from_secs(6));
    let stage = peer_stage(&e, &id);
    assert!(!matches!(stage.as_str(), "RECEIVED" | "STORED" | "HANDED_OVER" | "REPLIED"), "drained sidecar received the mail: stage {stage}");
    let msgs = std::fs::read_dir(store_dir.join("msgs")).unwrap().count();
    assert_eq!(msgs, 0, "nothing may be stored in the meter's store");
    let p = call(&e, METER, Scope::Send, &json!({"op": "pull", "session": SESSION, "hook": "post-tool-use"}));
    assert_eq!(p["none"], true, "pull must hand nothing over: {p}");
    assert!(p.get("frame").is_none());
    let s = call(&e, METER, Scope::Status, &json!({"op": "status"}));
    assert_eq!(s["doorbell"]["h11"]["a_check_to_keystroke"]["rings"], 0, "the doorbell never rang: {}", s["doorbell"]);
    assert!(!store_dir.join("doorbell-on").exists());

    // Negative control: same config, same process; the operator lifts the drain and turns
    // delivery on. The waiting message is now received and handed over.
    let r = call(&e, METER, Scope::Admin, &json!({"op": "undrain", "why": "T-3769 control"}));
    assert_eq!(r["ok"], true, "undrain: {r}");
    let r = call(&e, METER, Scope::Admin, &json!({"op": "delivery", "state": "on", "why": "T-3769 control"}));
    assert_eq!(r["ok"], true, "delivery on: {r}");
    assert!(wait_for(60, || matches!(peer_stage(&e, &id).as_str(), "RECEIVED" | "STORED" | "HANDED_OVER")), "control: the mail was never received (stage {})", peer_stage(&e, &id));
    let mut got = Value::Null;
    assert!(
        wait_for(30, || {
            got = call(&e, METER, Scope::Send, &json!({"op": "pull", "session": SESSION, "hook": "post-tool-use"}));
            got.get("frame").is_some()
        }),
        "control: pull never handed the mail over: {got}"
    );
    assert!(got["frame"].as_str().unwrap().contains("T-3769 waiting mail"), "control frame: {got}");
    assert!(got["msg_ids"].as_array().unwrap().iter().any(|m| m == id.as_str()));
}
