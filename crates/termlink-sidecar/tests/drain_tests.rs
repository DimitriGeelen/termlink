//! PH-2 row P4 (FP 6.2.3.e): drain and rollback. A drained sidecar pulls no new
//! mail (it stays on the hub, the cursor does not move), hands nothing over, and
//! keeps every stored message not yet handed over; the kept count is compared
//! with the listed one, and only a match says the old receiver may start.
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::deliver::{Deliverer, Pulled};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{PassResult, ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

static HUB_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn roster(e: &Estate) -> Roster {
    let mut r = Roster::new();
    r.add(e.identity("a").unwrap().verifying_key());
    r.add(e.identity("b").unwrap().verifying_key());
    r
}

fn side(e: &Estate, who: &str, name: &str, inst: &str, hub_id: &str) -> Side {
    let store = Arc::new(Store::open(&e.store_root(), name).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: name.into(),
                instance: inst.into(),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: hub_id.into(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(15),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: name.into(),
            instance: inst.into(),
            hub: e.hub_addr(),
            declared_hub_id: hub_id.into(),
            roster: roster(e),
            timeout: Duration::from_secs(15),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster(e))),
    )
    .unwrap();
    Side { store, sender, rx }
}

fn call(api: &Api, scope: Scope, req: Value) -> Value {
    let mut s = UnixStream::connect(api.path(scope)).unwrap();
    writeln!(s, "{req}").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap_or_else(|e| panic!("bad reply {line:?}: {e}"))
}

fn handled(r: PassResult) -> usize {
    match r {
        PassResult::CaughtUp { handled } => handled,
        other => panic!("expected CaughtUp, got {other:?}"),
    }
}

#[test]
fn drain_keeps_what_is_stored_leaves_new_mail_on_the_hub_and_undrain_resumes() {
    let _s = HUB_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let a = side(&e, "a", "agent-a", "inst-a", &id);
    let b = side(&e, "b", "agent-b", "inst-b", &id);
    let d = Arc::new(Deliverer::new("agent-b", b.store.clone(), Box::new(BuiltIn::new(roster(&e))), Some(b.sender.clone())));
    let mut cfg = ApiConfig::new("agent-b", &e.runtime_dir());
    cfg.deliverer = Some(d.clone());
    let api = Api::start(cfg, b.store.clone()).unwrap();

    a.sender.send("agent-b", "inst-b", Some("rb"), "before the drain", None).unwrap();
    assert_eq!(handled(b.rx.pass()), 1);
    let cursor = b.rx.cursor();

    let r = call(&api, Scope::Admin, json!({"op": "drain", "why": "W1 rollback drill"}));
    assert_eq!((r["ok"].clone(), r["listed"].clone(), r["kept"].clone()), (json!(true), json!(1), json!(1)), "{r}");
    let st = call(&api, Scope::Status, json!({"op": "status"}));
    assert_eq!(st["receiving"], "drained", "{st}");

    // New mail stays on the hub: nothing read, the cursor does not move.
    a.sender.send("agent-b", "inst-b", Some("rb"), "during the drain", None).unwrap();
    assert!(matches!(b.rx.pass(), PassResult::Drained));
    assert_eq!(b.rx.cursor(), cursor);
    // Nothing is handed over while drained.
    assert!(matches!(d.pull("s1", "post-tool-use", None).unwrap(), Pulled::Nothing { why: "drained" }));
    // The drain is on the admin chain with its count.
    let admin = b.store.chain_lines("_admin").unwrap();
    assert!(admin.iter().any(|l| l.event == "DRAIN" && l.reason.as_deref().is_some_and(|r| r.starts_with("listed=1 kept=1"))), "{admin:?}");

    // Undrain: the waiting message is read and stored; two are kept now.
    let u = call(&api, Scope::Admin, json!({"op": "undrain", "why": "drill over"}));
    assert_eq!(u["ok"], true, "{u}");
    assert_eq!(handled(b.rx.pass()), 1);
    assert_eq!(d.drain_inventory().kept.len(), 2);
    assert_eq!(call(&api, Scope::Status, json!({"op": "status"}))["receiving"], "on");
}

#[test]
fn drain_count_mismatch_is_refused_loudly() {
    let _s = HUB_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let a = side(&e, "a", "agent-a", "inst-a", &id);
    let b = side(&e, "b", "agent-b", "inst-b", &id);
    let d = Arc::new(Deliverer::new("agent-b", b.store.clone(), Box::new(BuiltIn::new(roster(&e))), Some(b.sender.clone())));
    let mut cfg = ApiConfig::new("agent-b", &e.runtime_dir());
    cfg.deliverer = Some(d.clone());
    let api = Api::start(cfg, b.store.clone()).unwrap();
    a.sender.send("agent-b", "inst-b", Some("mm"), "one", None).unwrap();
    a.sender.send("agent-b", "inst-b", Some("mm"), "two", None).unwrap();
    assert_eq!(handled(b.rx.pass()), 2);
    // Damage one stored record on disk.
    let msgs = b.store.dir().join("msgs");
    let victim = walk(&msgs).into_iter().next().expect("a stored record");
    std::fs::write(&victim, b"not the record").unwrap();
    let r = call(&api, Scope::Admin, json!({"op": "drain", "why": "drill"}));
    assert_eq!((r["ok"].clone(), r["code"].clone(), r["listed"].clone(), r["kept"].clone()), (json!(false), json!("count-mismatch"), json!(2), json!(1)), "{r}");
    assert!(r["error"].as_str().unwrap().contains("do not start the old receiver"), "{r}");
}

fn walk(p: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    for e in std::fs::read_dir(p).unwrap().flatten() {
        let path = e.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out.sort();
    out
}
