//! T-3708 (P-24 slice a) on a real hub (testkit estate): E-55 (fields stamped by their producers; p95 ≤ 5 s over
//! 100 calls with the hub up and with the hub stopped, the fault seam that exists before P-19), FCW-120 (a stopped
//! hub leaves `hub` fresh only within its bound), E-58 at L4 on this host (strength flags against `ps -o user=`).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

struct Rig {
    store: Arc<Store>,
    rx: Receiver,
    api: Api,
}

fn rig(e: &Estate) -> Rig {
    let id = e.hub_id().unwrap();
    let agent = "fx-b".to_string();
    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: "b1".into(),
                identity: e.identity("b").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join("outbox-b1.sqlite"),
                timeout: Duration::from_secs(5),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let roster = Roster::new();
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: "b1".into(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            roster: roster.clone(),
            timeout: Duration::from_secs(5),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster)),
    )
    .unwrap();
    let mut cfg = ApiConfig::new(&agent, &e.runtime_dir());
    cfg.hub_id = Some(id);
    cfg.sender = Some(sender);
    let api = Api::start(cfg, store.clone()).unwrap();
    Rig { store, rx, api }
}

fn status(r: &Rig) -> (Value, Duration) {
    let t = Instant::now();
    let mut s = UnixStream::connect(r.api.path(Scope::Status)).unwrap();
    writeln!(s, "{}", json!({"op": "status"})).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    (serde_json::from_str(&line).unwrap(), t.elapsed())
}

fn p95(mut d: Vec<Duration>) -> Duration {
    d.sort();
    d[(d.len() * 95).div_ceil(100) - 1]
}

#[test]
fn t3708_e55_fields_stamped_by_producers_and_fast_with_the_hub_up_or_down() {
    let Ok(mut e) = Estate::create() else {
        eprintln!("SKIP: no termlink binary for a test hub");
        return;
    };
    let r = rig(&e);
    let _ = r.rx.pass();
    let (v, _) = status(&r);
    // the pull pass read the hub: hub and revocation are fresh, with the producer's time
    for n in ["hub", "revocation", "consumers", "skew"] {
        assert_eq!(v["fields"][n]["state"], "fresh", "{n} after a pass: {}", v["fields"][n]);
    }
    assert_eq!(v["fields"]["hub"]["value"]["match"], true);
    let skew = &v["fields"]["skew"]["value"];
    assert_eq!(skew["clock"], "ok", "same host, no skew: {skew}");

    let mut up = vec![];
    for _ in 0..100 {
        up.push(status(&r).1);
    }
    e.stop_hub();
    let mut down = vec![];
    for _ in 0..100 {
        down.push(status(&r).1);
    }
    let (pu, pd) = (p95(up), p95(down));
    println!("E-55 p95 status latency: hub up {pu:?}, hub stopped {pd:?}");
    assert!(pu <= Duration::from_secs(5) && pd <= Duration::from_secs(5), "p95 up {pu:?} down {pd:?}");
    // status makes no hub call: a stopped hub costs nothing (design mutant 10)
    assert!(pd < Duration::from_millis(1500), "status waited on the stopped hub: p95 {pd:?}");
    // a pass against the stopped hub does not refresh `hub`
    let before = status(&r).0["fields"]["hub"]["at_ms"].clone();
    let _ = r.rx.pass();
    assert_eq!(status(&r).0["fields"]["hub"]["at_ms"], before, "a failed pass must not stamp the hub field");
    let lines = std::fs::read_to_string(r.store.dir().join(termlink_sidecar::api::LATENCY_LOG)).unwrap();
    assert!(lines.lines().count() >= 200, "one latency line per call");
    let _ = e.teardown();
}

/// E-58 at L4: every flag matches what the host shows (`ps -o user=` over the sidecar processes).
#[test]
fn t3708_e58_strength_flags_match_the_host() {
    let v = termlink_sidecar::host::strength_now();
    let out = std::process::Command::new("ps").args(["-eo", "user=,args="]).output().unwrap();
    let ps = String::from_utf8_lossy(&out.stdout);
    let users: std::collections::BTreeSet<&str> = ps
        .lines()
        .filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent")))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let agents = ps.lines().filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent"))).count();
    // SAFETY: geteuid has no preconditions.
    let root = unsafe { libc::geteuid() } == 0;
    println!("E-58 host: {agents} sidecar processes under users {users:?}; flags {v}");
    let pa = v["per_agent_accounts"].as_str().unwrap();
    if users.len() < agents || users.contains("root") {
        assert!(pa.starts_with("no"), "per_agent_accounts claims strength the host lacks: {pa}");
    }
    if root {
        assert_eq!(v["same_user_separation"], "absent: agents run as root");
    }
    assert_eq!(v["operator_key"], "no");
    assert_eq!(v["confirmation_provider"], "no");
    assert_eq!(v["policy_records"], "none (fixed defaults)");
}
