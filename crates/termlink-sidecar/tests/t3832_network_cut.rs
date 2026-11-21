//! T-3832 (P-24 slice b, design 3.8.3, E-60): a transport cut between a sender and a real, throwaway hub,
//! and the restore. The hub stays up; only the link fails. A proxy on the hub's socket plays the cut:
//!
//! 1. `EatPostReplies`: a `channel.post` reaches the hub, the hub stores it, and the reply never comes back
//!    (the case that produces duplicates: the sender cannot tell it was taken, so it posts again);
//! 2. `Down`: every connection is dropped at once (the link is gone);
//! 3. `Pass`: the link is restored.
//!
//! On restore the pump must leave each message on the hub exactly once by its identity triple (sender,
//! conversation, message id), with nothing missing. The estate hub listens on a unix socket, so the cut is
//! made at the transport (the socket), not by a firewall rule: a test never touches host network state.
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use termlink_sidecar::send::{PumpResult, SendConfig, Sender, inbox_topic};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

const PASS: u8 = 0;
const EAT_POST_REPLIES: u8 = 1;
const DOWN: u8 = 2;

/// A unix-socket proxy in front of the hub with a switchable fault.
struct Proxy {
    path: PathBuf,
    mode: Arc<AtomicU8>,
    /// `channel.post` requests the proxy passed to the hub and whose reply it dropped.
    eaten: Arc<AtomicUsize>,
}

impl Proxy {
    fn start(listen: PathBuf, hub: PathBuf) -> Proxy {
        let _ = std::fs::remove_file(&listen);
        let l = UnixListener::bind(&listen).unwrap();
        let mode = Arc::new(AtomicU8::new(PASS));
        let eaten = Arc::new(AtomicUsize::new(0));
        let (m, ea) = (mode.clone(), eaten.clone());
        std::thread::spawn(move || {
            for c in l.incoming().flatten() {
                let (m, ea, hub) = (m.clone(), ea.clone(), hub.clone());
                std::thread::spawn(move || serve(c, &hub, &m, &ea));
            }
        });
        Proxy { path: listen, mode, eaten }
    }

    fn set(&self, mode: u8) {
        self.mode.store(mode, Ordering::SeqCst);
    }
}

/// One client connection: read its request, then pass it on or fail per the mode.
fn serve(mut client: UnixStream, hub: &Path, mode: &AtomicU8, eaten: &AtomicUsize) {
    if mode.load(Ordering::SeqCst) == DOWN {
        return; // dropped: the client sees the connection close with no reply
    }
    let Ok(mut upstream) = UnixStream::connect(hub) else { return };
    let _ = client.set_read_timeout(Some(Duration::from_millis(300)));
    // The client writes its request and then waits: read until it pauses.
    let mut req = vec![];
    let mut buf = [0u8; 8192];
    loop {
        match client.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                req.extend_from_slice(&buf[..n]);
                if req.contains(&b'\n') {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let is_post = String::from_utf8_lossy(&req).contains("\"channel.post\"");
    if upstream.write_all(&req).is_err() {
        return;
    }
    let _ = upstream.set_read_timeout(Some(Duration::from_secs(10)));
    let mut reply = vec![];
    loop {
        match upstream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                reply.extend_from_slice(&buf[..n]);
                if reply.contains(&b'\n') {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    if is_post && mode.load(Ordering::SeqCst) == EAT_POST_REPLIES {
        // The hub has answered, so it has taken the post; the reply is lost in the cut.
        eaten.fetch_add(1, Ordering::SeqCst);
        return;
    }
    let _ = client.write_all(&reply);
}

fn rpc_direct(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    termlink_session::client::unwrap_result(resp).unwrap()
}

/// Every message on the recipient's inbox on the hub, as (sender, conversation, message id) triples.
/// Read directly from the throwaway hub (not through the proxy): this is the test's own inbox.
fn triples_on_hub(e: &Estate, topic: &str) -> Vec<(String, String, String)> {
    let v = rpc_direct(e, "channel.subscribe", json!({"topic": topic, "cursor": 0, "limit": 1000}));
    v["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|m| {
            let md = &m["metadata"];
            (
                m["sender_id"].as_str().unwrap_or("").to_owned(),
                md["conversation_id"].as_str().unwrap_or("").to_owned(),
                md["sidecar_msg_id"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect()
}

#[test]
fn t3832_after_a_network_cut_the_pump_posts_only_what_is_missing() {
    let e = Estate::create().unwrap();
    let hub_id = e.hub_id().unwrap();
    let proxy = Proxy::start(e.root().join("cut-proxy.sock"), e.runtime_dir().join("hub.sock"));
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sender = Sender::new(
        SendConfig {
            agent: "agent-a".into(),
            instance: "agent-a.w1".into(),
            identity: e.identity("a").unwrap(),
            hub: termlink_protocol::TransportAddr::unix(proxy.path.clone()),
            declared_hub_id: hub_id.clone(),
            queue_path: e.root().join("outbox-a.sqlite"),
            timeout: Duration::from_secs(3),
            pump_interval: None,
        },
        store.clone(),
    )
    .unwrap();
    let topic = inbox_topic(&hub_id, "agent-b");

    // The link works: m1 is posted.
    let m1 = sender.send("agent-b", "agent-b.w1", None, "one", None).unwrap().msg_id;
    assert_eq!(sender.pump(), PumpResult::Drained);

    // The cut, first half: m2 reaches the hub but its reply is lost, so the sender keeps it in the outbox.
    proxy.set(EAT_POST_REPLIES);
    let m2 = sender.send("agent-b", "agent-b.w1", None, "two", None).unwrap().msg_id;
    let m3 = sender.send("agent-b", "agent-b.w1", None, "three", None).unwrap().msg_id;
    assert_eq!(sender.pump(), PumpResult::Waiting);
    assert!(proxy.eaten.load(Ordering::SeqCst) >= 1, "the hub took a post whose reply was lost");
    // The link is gone: nothing passes, and the message sent meanwhile waits too.
    proxy.set(DOWN);
    let m4 = sender.send("agent-b", "agent-b.w1", None, "four", None).unwrap().msg_id;
    assert_eq!(sender.pump(), PumpResult::Waiting);
    assert_eq!(sender.outbox_len(), 3, "m2, m3 and m4 are still the sender's to post");

    // Before the restore the hub already holds m2 (it took the post): re-posting it is the duplicate risk.
    let before = triples_on_hub(&e, &topic);
    assert!(before.iter().any(|t| t.2 == m2), "m2 reached the hub during the cut: {before:?}");
    assert!(!before.iter().any(|t| t.2 == m3 || t.2 == m4), "{before:?}");

    // Restore: the pump drains, and every message is on the hub exactly once, none missing.
    proxy.set(PASS);
    assert_eq!(sender.pump(), PumpResult::Drained);
    assert_eq!(sender.outbox_len(), 0);
    let after = triples_on_hub(&e, &topic);
    for m in [&m1, &m2, &m3, &m4] {
        let n = after.iter().filter(|t| &t.2 == m).count();
        assert_eq!(n, 1, "{m} is on the hub {n} times: {after:?}");
    }
    let mut uniq = after.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), after.len(), "no identity triple twice: {after:?}");
    assert_eq!(after.len(), 4, "{after:?}");
}
