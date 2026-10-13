//! B12: the fail-closed rows not covered by an earlier step's tests
//! (architecture note 9.1): FC-1 hub half, FC-2 ENOSPC, FC-8 hub outage,
//! FC-17 secret scan. Hermetic: testkit estate only.

use base64::Engine as _;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::api::{self, Scope};
use termlink_sidecar::deadline::Deadlines;
use termlink_sidecar::daemon::Config;
use termlink_sidecar::hook;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{PassResult, ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender, inbox_topic};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::{self, Roster};

const BIN: &str = env!("CARGO_BIN_EXE_termlink-sidecar");

static HUB_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    HUB_SERIAL.lock().unwrap_or_else(|p| p.into_inner())
}

fn wait_for(secs: u64, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    false
}

fn roster_of(ids: &[&Identity]) -> Roster {
    let mut r = Roster::new();
    for i in ids {
        r.add(i.verifying_key());
    }
    r
}

fn sender(e: &Estate, store: &Arc<Store>, who: &str, name: &str, inst: &str, hub_id: &str) -> Arc<Sender> {
    Arc::new(
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
    )
}

fn receiver(e: &Estate, store: &Arc<Store>, s: &Arc<Sender>, name: &str, inst: &str, hub_id: &str, roster: &Roster) -> Receiver {
    Receiver::new(
        ReceiveConfig {
            agent: name.into(),
            instance: inst.into(),
            hub: e.hub_addr(),
            declared_hub_id: hub_id.into(),
            roster: roster.clone(),
            timeout: Duration::from_secs(15),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        s.clone(),
        Box::new(BuiltIn::new(roster.clone())),
    )
    .unwrap()
}

fn events(s: &Store, conv: &str, id: &str) -> Vec<String> {
    s.chain_lines(conv).unwrap().into_iter().filter(|l| l.msg_id == id).map(|l| l.event).collect()
}

fn rpc(e: &Estate, m: &str, params: Value) -> Result<Value, String> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), m, params, Duration::from_secs(15)))
        .map_err(|x| x.to_string())?;
    termlink_session::client::unwrap_result(resp)
}

// ---- FC-1, hub half ---------------------------------------------------------------

#[test]
fn fc_1_hub_refuses_a_post_whose_signature_does_not_verify() {
    let _s = serial();
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let ka = e.identity("a").unwrap();
    let topic = inbox_topic(&id, "agent-b");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    let payload = b"{\"v\":1}".to_vec();
    let sig = ka.sign(&canonical_sign_bytes(&topic, wire::MSG_TYPE, &payload, None, 7));
    let hex = |b: &[u8]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
    let post = |body: &[u8], sig_hex: &str, pk: &str, sender_id: &str| {
        rpc(
            &e,
            "channel.post",
            json!({
                "topic": topic, "msg_type": wire::MSG_TYPE,
                "payload_b64": base64::engine::general_purpose::STANDARD.encode(body),
                "ts": 7, "sender_id": sender_id, "sender_pubkey_hex": pk, "signature_hex": sig_hex,
            }),
        )
    };
    // Altered after signing.
    assert!(post(b"{\"v\":2}", &hex(&sig.to_bytes()), &ka.public_key_hex(), ka.fingerprint()).is_err());
    // Signed by another key than the one presented.
    let other = e.identity("host").unwrap();
    let osig = other.sign(&canonical_sign_bytes(&topic, wire::MSG_TYPE, &payload, None, 7));
    assert!(post(&payload, &hex(&osig.to_bytes()), &ka.public_key_hex(), ka.fingerprint()).is_err());
    // Missing signature.
    assert!(post(&payload, "", &ka.public_key_hex(), ka.fingerprint()).is_err());
    // Control: the honest post is accepted.
    assert!(post(&payload, &hex(&sig.to_bytes()), &ka.public_key_hex(), ka.fingerprint()).is_ok());
}

// ---- FC-2: no space left ------------------------------------------------------------

struct Mount(PathBuf);
impl Drop for Mount {
    fn drop(&mut self) {
        let _ = std::process::Command::new("umount").arg(&self.0).status();
    }
}

#[test]
fn fc_2_a_full_disk_never_reports_stored_and_the_message_waits() {
    // SAFETY: geteuid has no preconditions.
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("fc_2: SKIPPED: mounting a size-limited tmpfs needs root");
        return;
    }
    let _s = serial();
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let (ka, kb) = (e.identity("a").unwrap(), e.identity("b").unwrap());
    let roster = roster_of(&[&ka, &kb]);
    // B's store on a 256 KiB tmpfs inside the estate.
    let small = e.root().join("small");
    std::fs::create_dir_all(&small).unwrap();
    let st = std::process::Command::new("mount").args(["-t", "tmpfs", "-o", "size=256k,mode=0700", "tmpfs"]).arg(&small).status().unwrap();
    assert!(st.success(), "mount failed");
    let _m = Mount(small.clone());
    let store_a = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let store_b = Arc::new(Store::open(&small, "agent-b").unwrap());
    let sa = sender(&e, &store_a, "a", "agent-a", "inst-a", &id);
    let sb = sender(&e, &store_b, "b", "agent-b", "inst-b", &id);
    let rx_a = receiver(&e, &store_a, &sa, "agent-a", "inst-a", &id, &roster);
    let rx_b = receiver(&e, &store_b, &sb, "agent-b", "inst-b", &id, &roster);
    let big = "x".repeat(512 * 1024);
    let s = sa.send("agent-b", "inst-b", Some("full"), &big, None).unwrap();
    let r = rx_b.pass();
    assert!(matches!(r, PassResult::Stalled { .. }), "{r:?}");
    assert!(!events(&store_b, "full", &s.msg_id).contains(&"STORED".to_string()));
    // The partial temp file is gone, so a retry starts from a clean disk...
    let tmp_left = || std::fs::read_dir(store_b.dir().join("tmp")).unwrap().count();
    assert_eq!(tmp_left(), 0, "a failed write leaked its temp file");
    // ...and fails the same way again, still without STORED and without a leak.
    assert!(matches!(rx_b.pass(), PassResult::Stalled { .. }));
    assert_eq!(tmp_left(), 0);
    rx_a.pass();
    let ev = events(&store_a, "full", &s.msg_id);
    assert!(!ev.contains(&"STORED".to_string()), "STORED reported on a full disk: {ev:?}");
    // The message is not lost: the cursor did not move past it.
    assert_eq!(rx_b.cursor(), 0);
    // The sender sees it: received, never stored, STUCK past the accept deadline
    // (R-44.e): the recipient is demonstrably alive, so the step is named.
    let later = termlink_sidecar::store::now_ms() + Deadlines::R44E.accept(false) + 60_000;
    let v = api::status_view(&json!({"op": "status", "msg_id": s.msg_id}), &store_a, "agent-a", Some(&id), later, &Deadlines::R44E);
    assert_eq!((v["messages"][0]["stage"].clone(), v["messages"][0]["label"].clone()), (json!("RECEIVED"), json!("STUCK")), "{v}");
    assert_eq!(v["messages"][0]["stuck_step"], "not accepted", "{v}");
}

// ---- FC-8: hub outage ----------------------------------------------------------------

#[test]
fn fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss() {
    let _s = serial();
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let (ka, kb) = (e.identity("a").unwrap(), e.identity("b").unwrap());
    let roster = roster_of(&[&ka, &kb]);
    let store_a = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sa = sender(&e, &store_a, "a", "agent-a", "inst-a", &id);
    let view = |now: u64, msg: &str| {
        api::status_view(&json!({"op": "status", "msg_id": msg}), &store_a, "agent-a", Some(&id), now, &Deadlines::R44E)["messages"][0].clone()
    };
    e.stop_hub();
    let s = sa.send("agent-b", "inst-b", Some("outage"), "while the hub is down", None).unwrap();
    assert_eq!(view(termlink_sidecar::store::now_ms(), &s.msg_id)["label"], "WAITING");
    assert_eq!(sa.outbox_len(), 1);
    e.restart_hub().unwrap();
    assert!(wait_for(30, || { sa.pump(); sa.outbox_len() == 0 }), "never posted after the hub came back");
    // Posted, but nobody has reported a stage: past the deadline it is UNKNOWN.
    let later = termlink_sidecar::store::now_ms() + Deadlines::R44E.accept(false) + 60_000;
    assert_eq!(view(later, &s.msg_id)["label"], "UNKNOWN");
    // B comes up: the message is received and stored, nothing lost.
    let store_b = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
    let sb = sender(&e, &store_b, "b", "agent-b", "inst-b", &id);
    let rx_b = receiver(&e, &store_b, &sb, "agent-b", "inst-b", &id, &roster);
    let rx_a = receiver(&e, &store_a, &sa, "agent-a", "inst-a", &id, &roster);
    rx_b.pass();
    rx_a.pass();
    let ev = events(&store_a, "outage", &s.msg_id);
    assert!(ev.ends_with(&["RECEIVED".to_string(), "STORED".to_string()]), "{ev:?}");
    assert_eq!(ev.iter().filter(|x| *x == "SENT").count(), 1);
}

// ---- FC-17: the secret never leaves the environment ------------------------------------------

struct Proc(Child);
impl Drop for Proc {
    fn drop(&mut self) {
        // SAFETY: plain kill(2) on our own child.
        unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) };
        let _ = self.0.wait();
    }
}

fn start(e: &Estate, who: &str, name: &str, inst: &str, hub_id: &str) -> Proc {
    let roster: Vec<String> = ["a", "b"].iter().map(|w| e.identity(w).unwrap().public_key_hex()).collect();
    let cfg = Config {
        agent: name.into(),
        instance: inst.into(),
        hub: e.hub_addr(),
        hub_id: hub_id.into(),
        key_file: e.root().join("keys").join(format!("{who}.key")),
        roster,
        queue_path: e.root().join(format!("outbox-{who}.sqlite")),
        poll_ms: 200,
        timeout_ms: 15_000,
        claude_projects: None,
        key_register: None,
        book: Default::default(),
    };
    let path = e.root().join(format!("sidecar-{who}.json"));
    std::fs::write(&path, serde_json::to_string(&cfg).unwrap()).unwrap();
    let log = std::fs::File::create(e.root().join(format!("sidecar-{who}.log"))).unwrap();
    let child = e.command(BIN).args(["start", "--config"]).arg(&path).stdin(Stdio::null()).stdout(log.try_clone().unwrap()).stderr(log).spawn().unwrap();
    assert!(wait_for(30, || api::socket_path(&e.runtime_dir(), name, Scope::Send).exists()));
    Proc(child)
}

fn sock(e: &Estate, name: &str, scope: Scope) -> PathBuf {
    api::socket_path(&e.runtime_dir(), name, scope)
}

fn run_hook(e: &Estate, event: &str, socket: &Path, input: &Value) -> String {
    let mut c = e
        .command(BIN)
        .args(["hook", event])
        .env(hook::SOCKET_ENV, socket)
        .env(hook::FINALIZE_SECS_ENV, "20")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    String::from_utf8_lossy(&c.wait_with_output().unwrap().stdout).into_owned()
}

#[test]
fn fc_17_the_canary_appears_in_nothing_the_estate_produced() {
    let _s = serial();
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let canary = e.canary().to_owned();
    let mut outputs: Vec<String> = vec![];
    {
        let _a = start(&e, "a", "agent-a", "inst-a", &id);
        let _b = start(&e, "b", "agent-b", "inst-b", &id);
        let sent = hook::call(
            &sock(&e, "agent-a", Scope::Send),
            &json!({"op": "send", "to": "agent-b", "to_instance": "inst-b", "conv": "secret", "body": "a plain message"}),
            Duration::from_secs(20),
        )
        .unwrap();
        outputs.push(sent.to_string());
        let id_m = sent["msg_id"].as_str().unwrap().to_owned();
        let st = |name: &str| hook::call(&sock(&e, name, Scope::Status), &json!({"op": "status"}), Duration::from_secs(10)).unwrap();
        assert!(wait_for(30, || st("agent-a")["messages"].as_array().is_some_and(|m| m.iter().any(|x| x["msg_id"] == id_m.as_str() && x["stage"] == "STORED"))));
        let session = "fc17-session";
        let out = run_hook(&e, "post-tool-use", &sock(&e, "agent-b", Scope::Send), &json!({"session_id": session, "tool_use_id": "toolu_fc17"}));
        outputs.push(out.clone());
        let printed: Value = serde_json::from_str(out.trim()).unwrap();
        let frame = printed["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
        let proj = e.home().join(".claude/projects/-fc17");
        std::fs::create_dir_all(&proj).unwrap();
        let rec = json!({"type": "attachment", "attachment": {"type": "hook_additional_context", "content": [frame], "toolUseID": "toolu_fc17"}});
        std::fs::write(proj.join(format!("{session}.jsonl")), format!("{rec}\n")).unwrap();
        assert!(wait_for(40, || st("agent-a")["messages"].as_array().is_some_and(|m| m.iter().any(|x| x["msg_id"] == id_m.as_str() && x["stage"] == "HANDED_OVER"))));
        outputs.push(st("agent-a").to_string());
        outputs.push(st("agent-b").to_string());
        let r = hook::call(&sock(&e, "agent-b", Scope::Send), &json!({"op": "reply", "conv": "secret", "in_reply_to": id_m, "body": "ok"}), Duration::from_secs(20)).unwrap();
        outputs.push(r.to_string());
    }
    // Every file the run left in the estate (logs, stores, chains, records,
    // claims, outboxes, hub data) and every answer it gave.
    let hits = e.secret_scan(e.root());
    assert!(hits.is_empty(), "the canary leaked into {hits:?}");
    for o in &outputs {
        assert!(!o.contains(&canary), "the canary leaked into an output: {o}");
    }
    assert!(e.root().join("sidecar-a.log").exists(), "the scan must cover the sidecar logs");
    let _ = BTreeMap::<String, String>::new();
}
