//! Send path: signs a message, puts it in the durable outbox, records it as sent and returns its id.
//! C-5 send path (architecture note section 3.2 `send`, 5.1, 5.5; D-3, D-4).
//!
//! `Sender::send` signs the message (wire.rs), puts it in the durable outbox (the
//! session crate's `OfflineQueue`), records SENT in the sender's chain, and only
//! then returns `{msg_id, stage: SENT}`. It never says "delivered" (FX-7).
//!
//! A pump posts the outbox to the recipient inbox topic on the hub, oldest
//! first. Before every pass it reads `hub_id` from the hub (`hub.version`) and
//! compares it with the declared id (FX-2): a mismatch, or a hub that states no
//! id, posts nothing, leaves the messages in the outbox and writes a
//! needs-attention entry. A hub that cannot be reached leaves the outbox alone;
//! the message shows WAITING and the pump retries.
//!
//! Chain events written here, besides SENT: `WAITING` (a post attempt found the
//! hub unreachable, once per message), `POSTED` (the hub accepted it), and
//! `DEAD_LETTER` (the hub rejected it `POISON_THRESHOLD` times). The post carries
//! the message id as `client_msg_id`, so a re-post after a crash between the hub
//! accepting and the outbox row being removed is absorbed by the hub's dedupe.
//!
//! Only a stored message counts as accepted (operator ruling 2026-10-18, T-3461).
//! When the hub accepts a post the outbox row goes, but the sender keeps the
//! signed post in a durable awaiting record (`<outbox>.awaiting/<msg_id>.json`)
//! until the recipient reports STORED (or a later stage). `resend_overdue`
//! re-queues a message with no such report past the R-44.e accept deadline
//! (deadline.rs), with the same msg id as `client_msg_id`: the hub's dedupe and
//! the recipient store's Duplicate outcome keep it from arriving twice. Each
//! resend writes `RESENT` and doubles the next gap; after `MAX_RESENDS` the
//! record is dropped, `NOT_ACCEPTED` is written and a needs-attention entry
//! names the message.

use crate::store::{Store, StoreError};
use crate::wire::{self, Msg};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use termlink_protocol::TransportAddr;
use termlink_protocol::control::method;
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::agent_identity::Identity;
use termlink_session::bus_client::POISON_THRESHOLD;
use termlink_session::client::{rpc_call_addr_with_timeout, unwrap_result};
use termlink_session::offline_queue::{OfflineQueue, PendingPost, QueueError};

pub const WAITING: &str = "WAITING";
pub const POSTED: &str = "POSTED";
pub const DEAD_LETTER: &str = "DEAD_LETTER";
/// A message re-queued because no STORED report came back in time.
pub const RESENT: &str = "RESENT";
/// The sender gave up resending: the recipient never reported STORED.
pub const NOT_ACCEPTED: &str = "NOT_ACCEPTED";
/// Needs-attention kind written when the sender gives up.
pub const ATTN_NOT_ACCEPTED: &str = "not-accepted";
/// Needs-attention kind for an awaiting record (or the awaiting dir) that cannot be read.
pub const ATTN_AWAITING_UNREADABLE: &str = "awaiting-unreadable";
/// Suffix an unreadable awaiting record is moved to, so it stays on disk unre-read.
const UNREADABLE_SUFFIX: &str = ".unreadable";
/// Resends per message before the sender gives up (bounded: no flooding).
pub const MAX_RESENDS: u32 = 4;

pub struct SendConfig {
    /// This agent's name (`from_agent`).
    pub agent: String,
    /// This agent's exact instance (`from_instance`): replies come back to it.
    pub instance: String,
    /// The agent's own signing key (R-75).
    pub identity: Identity,
    /// Where the hub listens.
    pub hub: TransportAddr,
    /// The hub id this sidecar is configured to talk to (FX-2).
    pub declared_hub_id: String,
    /// The durable outbox file.
    pub queue_path: PathBuf,
    /// Bound on each hub call.
    pub timeout: Duration,
    /// How often the background pump retries. `None` disables it (tests drive `pump`).
    pub pump_interval: Option<Duration>,
}

#[derive(Debug)]
pub enum SendError {
    /// A request field is unusable.
    BadRequest(String),
    /// The hub answered with an id other than the declared one, or with none.
    HubId { declared: String, found: Option<String> },
    /// The outbox is full or unreadable; nothing was accepted.
    Outbox(String),
    /// The chain could not record SENT; the message was taken back out of the outbox.
    Store(String),
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(m) => write!(f, "{m}"),
            Self::HubId { declared, found } => write!(
                f,
                "hub id mismatch: declared {declared}, hub says {}",
                found.as_deref().unwrap_or("nothing")
            ),
            Self::Outbox(m) => write!(f, "outbox: {m}"),
            Self::Store(m) => write!(f, "store: {m}"),
        }
    }
}

impl std::error::Error for SendError {}

impl SendError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad-request",
            Self::HubId { .. } => "hub-id-mismatch",
            Self::Outbox(_) => "outbox",
            Self::Store(_) => "store",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sent {
    pub msg_id: String,
    pub conv: String,
    pub seq: u64,
    pub ts_ms: u64,
}

/// What one pump pass ended on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PumpResult {
    /// The outbox is empty.
    Drained,
    /// The hub could not be reached (or asked us to back off); messages remain.
    Waiting,
    /// The hub is not the declared one; nothing was posted.
    Held(String),
    /// The hub rejected the head message; it is retried, then dead-lettered.
    Rejected(String),
}

/// What one `resend_overdue` pass did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResendReport {
    /// Messages the recipient reported STORED: their awaiting record is gone.
    pub released: Vec<String>,
    /// Messages re-queued for another post.
    pub resent: Vec<String>,
    /// Messages given up on: needs-attention written.
    pub gave_up: Vec<String>,
}

/// A posted message the sender is still responsible for (durable, one file each).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Awaiting {
    conv: String,
    msg_id: String,
    urgent: bool,
    /// When it was last posted or re-queued (ms since the epoch).
    since_ms: u64,
    resends: u32,
    post: PendingPost,
}

/// The awaiting dir as read: parseable records, and the ids of quarantined ones.
#[derive(Default)]
struct AwaitingScan {
    records: Vec<Awaiting>,
    quarantined: Vec<String>,
}

/// Why a pin check did not pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinError {
    Unreachable(String),
    Mismatch { declared: String, found: Option<String> },
}

struct Inner {
    cfg: SendConfig,
    store: Arc<Store>,
    queue: OfflineQueue,
    rt: tokio::runtime::Runtime,
    /// Serialises pump passes and holds what they remember.
    pump: Mutex<PumpMemory>,
    stop: AtomicBool,
    /// Last time the background loop came round (ms since the epoch); the
    /// component heartbeat (K-3, FP-1 failure 4).
    beat: AtomicU64,
}

#[derive(Default)]
struct PumpMemory {
    topics: BTreeSet<String>,
    held: Option<String>,
}

pub struct Sender {
    inner: Arc<Inner>,
    thread: Option<JoinHandle<()>>,
}

pub fn inbox_topic(hub_id: &str, to_agent: &str) -> String {
    format!("inbox:{hub_id}/{to_agent}")
}

fn clamp_priority(p: Option<&str>) -> Result<String, SendError> {
    match p {
        None => Ok("0".into()),
        Some(s) => s
            .parse::<i64>()
            .map(|n| n.clamp(-9, 9).to_string())
            .map_err(|_| SendError::BadRequest("priority must be an integer".into())),
    }
}

fn post_params(p: &PendingPost) -> Value {
    use base64::Engine as _;
    let mut v = json!({
        "topic": p.topic,
        "msg_type": p.msg_type,
        "payload_b64": base64::engine::general_purpose::STANDARD.encode(&p.payload),
        "artifact_ref": p.artifact_ref,
        "ts": p.ts_unix_ms,
        "sender_id": p.sender_id,
        "sender_pubkey_hex": p.sender_pubkey_hex,
        "signature_hex": p.signature_hex,
    });
    if !p.metadata.is_empty() {
        v["metadata"] = json!(p.metadata);
    }
    if let Some(c) = &p.client_msg_id {
        v["client_msg_id"] = json!(c);
    }
    v
}

impl Sender {
    pub fn new(cfg: SendConfig, store: Arc<Store>) -> Result<Sender, SendError> {
        let queue = OfflineQueue::open(&cfg.queue_path).map_err(|e| SendError::Outbox(e.to_string()))?;
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(|e| SendError::Outbox(format!("runtime: {e}")))?;
        let interval = cfg.pump_interval;
        let inner = Arc::new(Inner {
            cfg,
            store,
            queue,
            rt,
            pump: Mutex::new(PumpMemory::default()),
            stop: AtomicBool::new(false),
            beat: AtomicU64::new(crate::store::now_ms()),
        });
        let thread = interval.map(|iv| {
            let i = inner.clone();
            std::thread::spawn(move || {
                let mut waited = Duration::ZERO;
                while !i.stop.load(Ordering::SeqCst) {
                    i.beat.store(crate::store::now_ms(), Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(50));
                    waited += Duration::from_millis(50);
                    if waited >= iv {
                        waited = Duration::ZERO;
                        i.run_pump();
                        i.resend_overdue(crate::store::now_ms(), &crate::deadline::Deadlines::R44E);
                    }
                }
            })
        });
        Ok(Sender { inner, thread })
    }

    /// When the background pump last came round (ms since the epoch).
    pub fn last_beat_ms(&self) -> u64 {
        self.inner.beat.load(Ordering::SeqCst)
    }

    /// Sign, enqueue durably, record SENT, try one post. Returns after SENT is
    /// recorded; the hub may still be unreachable.
    pub fn send(
        &self,
        to: &str,
        to_instance: &str,
        conv: Option<&str>,
        body: &str,
        priority: Option<&str>,
    ) -> Result<Sent, SendError> {
        self.inner.send(to, to_instance, conv, body, priority)
    }

    /// Queue a signed stage record (B7) for the original sender's inbox. It rides
    /// the same durable outbox and pump as messages, but writes no chain events
    /// of its own: the stage line is already in this agent's chain.
    pub fn post_stage(&self, to_agent: &str, st: &wire::Stage) -> Result<(), SendError> {
        self.inner.post_stage(to_agent, st)
    }

    /// The agent's own fingerprint.
    pub fn fingerprint(&self) -> &str {
        self.inner.cfg.identity.fingerprint()
    }

    /// Reply to a message this agent stored (R-27, BV-6): the same path as
    /// `send`, addressed to the original's sender agent and exact instance
    /// (SI-26). Once the reply is SENT, REPLIED is recorded on the original and
    /// reported to its sender as a signed stage.
    pub fn reply(&self, conv: &str, in_reply_to: &str, body: &str) -> Result<Sent, SendError> {
        self.inner.reply(conv, in_reply_to, body)
    }

    /// One pump pass (the background thread runs the same thing).
    pub fn pump(&self) -> PumpResult {
        self.inner.run_pump()
    }

    /// Release messages the recipient stored; re-queue the overdue ones; give up
    /// after `MAX_RESENDS`. `now_ms` is passed in so tests can move the clock.
    pub fn resend_overdue(&self, now_ms: u64, dl: &crate::deadline::Deadlines) -> ResendReport {
        self.inner.resend_overdue(now_ms, dl)
    }

    /// Msg ids the sender still holds awaiting a STORED report.
    pub fn awaiting(&self) -> Vec<String> {
        self.inner.awaiting_ids()
    }

    /// Messages still in the outbox.
    pub fn outbox_len(&self) -> u64 {
        self.inner.queue.size().unwrap_or(0)
    }

    /// Why posting is currently held back (hub id mismatch), if it is.
    pub fn held(&self) -> Option<String> {
        self.inner.pump.lock().ok().and_then(|m| m.held.clone())
    }

    /// Read the hub's id and compare it with the declared one.
    pub fn check_pin(&self) -> Result<(), PinError> {
        self.inner.check_pin()
    }

    pub fn declared_hub_id(&self) -> &str {
        &self.inner.cfg.declared_hub_id
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.inner.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Inner {
    fn rpc(&self, m: &str, params: Value) -> Result<RpcResponse, String> {
        self.rt
            .block_on(rpc_call_addr_with_timeout(&self.cfg.hub, m, params, self.cfg.timeout))
            .map_err(|e| e.to_string())
    }

    fn check_pin(&self) -> Result<(), PinError> {
        let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
        let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
        let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
        if found.as_deref() == Some(self.cfg.declared_hub_id.as_str()) {
            Ok(())
        } else {
            Err(PinError::Mismatch { declared: self.cfg.declared_hub_id.clone(), found })
        }
    }

    fn send(
        &self,
        to: &str,
        to_instance: &str,
        conv: Option<&str>,
        body: &str,
        priority: Option<&str>,
    ) -> Result<Sent, SendError> {
        if to.is_empty() || to_instance.is_empty() {
            return Err(SendError::BadRequest("to and to_instance are required".into()));
        }
        let priority = clamp_priority(priority)?;
        // A hub known to be the wrong one is refused before anything is queued.
        match self.check_pin() {
            Ok(()) | Err(PinError::Unreachable(_)) => {}
            Err(PinError::Mismatch { declared, found }) => return Err(SendError::HubId { declared, found }),
        }
        let me = &self.cfg.identity;
        let conv = conv.map(str::to_owned).unwrap_or_else(|| format!("c-{}", &wire::mint_msg_id()[..16]));
        let seq = self
            .store
            .chain_lines(&conv)
            .map_err(|e| SendError::Store(e.to_string()))?
            .iter()
            .filter(|l| l.event == "SENT" && l.sender == me.fingerprint())
            .count() as u64;
        let ts_ms = crate::store::now_ms();
        let msg = Msg {
            from: me.fingerprint().to_owned(),
            from_agent: self.cfg.agent.clone(),
            from_instance: self.cfg.instance.clone(),
            to_agent: to.to_owned(),
            to_instance: to_instance.to_owned(),
            conv: conv.clone(),
            msg_id: wire::mint_msg_id(),
            seq,
            priority,
            ts_ms: ts_ms as i64,
            body: body.to_owned(),
        };
        let topic = inbox_topic(&self.cfg.declared_hub_id, to);
        let signed = wire::sign(me, &topic, ts_ms as i64, &msg);
        let digest = msg.digest();
        let post = PendingPost {
            topic,
            msg_type: wire::MSG_TYPE.to_owned(),
            payload: signed.payload,
            artifact_ref: None,
            ts_unix_ms: signed.ts_ms,
            sender_id: me.fingerprint().to_owned(),
            sender_pubkey_hex: signed.signer_pk.clone(),
            signature_hex: signed.sig.clone(),
            metadata: BTreeMap::from([
                ("conversation_id".to_owned(), conv.clone()),
                ("sidecar_msg_id".to_owned(), msg.msg_id.clone()),
                ("sidecar_digest".to_owned(), digest.clone()),
                (wire::META_PK.to_owned(), signed.signer_pk),
                (wire::META_SIG.to_owned(), signed.sig),
            ]),
            client_msg_id: Some(msg.msg_id.clone()),
        };
        // Durable outbox first, then the SENT line. If the line cannot be written
        // the message is taken back out: SENT is never claimed without an outbox row.
        let qid = self.queue.enqueue(&post).map_err(|e| match e {
            QueueError::QueueFull { cap } => SendError::Outbox(format!("outbox full (cap {cap})")),
            other => SendError::Outbox(other.to_string()),
        })?;
        // The SENT line carries the priority, so the sender's own view can apply
        // the urgent deadlines later without keeping the payload (PH-2 row P2).
        let extra = crate::store::LineExtra { reason: Some(format!("priority={}", msg.priority)), ..Default::default() };
        if let Err(e) = self.store.append_event_unless(&conv, "SENT", &msg.msg_id, me.fingerprint(), &digest, extra, |_| false) {
            let _ = self.queue.pop(qid);
            return Err(SendError::Store(e.to_string()));
        }
        let sent = Sent { msg_id: msg.msg_id.clone(), conv: conv.clone(), seq, ts_ms };
        // Best effort now; the pump keeps trying. If it could not post, say WAITING.
        let r = self.run_pump();
        if !matches!(r, PumpResult::Drained) {
            let _ = self.mark_waiting(&conv, &msg.msg_id, &digest);
        }
        Ok(sent)
    }

    fn post_stage(&self, to_agent: &str, st: &wire::Stage) -> Result<(), SendError> {
        let me = &self.cfg.identity;
        let topic = inbox_topic(&self.cfg.declared_hub_id, to_agent);
        let signed = wire::sign_stage(me, &topic, st.ts_ms, st);
        let post = PendingPost {
            topic,
            msg_type: wire::STAGE_TYPE.to_owned(),
            payload: signed.payload,
            artifact_ref: None,
            ts_unix_ms: signed.ts_ms,
            sender_id: me.fingerprint().to_owned(),
            sender_pubkey_hex: signed.signer_pk.clone(),
            signature_hex: signed.sig.clone(),
            metadata: BTreeMap::from([
                ("conversation_id".to_owned(), st.conv.clone()),
                (wire::META_PK.to_owned(), signed.signer_pk),
                (wire::META_SIG.to_owned(), signed.sig),
            ]),
            client_msg_id: Some(wire::mint_msg_id()),
        };
        self.queue.enqueue(&post).map_err(|e| match e {
            QueueError::QueueFull { cap } => SendError::Outbox(format!("outbox full (cap {cap})")),
            other => SendError::Outbox(other.to_string()),
        })?;
        let _ = self.run_pump();
        Ok(())
    }

    fn reply(&self, conv: &str, in_reply_to: &str, body: &str) -> Result<Sent, SendError> {
        let lines = self.store.chain_lines(conv).map_err(|e| SendError::Store(e.to_string()))?;
        // The original: stored here (an inbound record, not a peer's report) and not refused.
        let st = lines
            .iter()
            .find(|l| l.event == "STORED" && l.peer_ts_ms.is_none() && l.msg_id == in_reply_to)
            .ok_or_else(|| SendError::BadRequest(format!("{conv}/{in_reply_to} is not a message stored here")))?;
        if lines.iter().any(|l| l.event == "REFUSED" && l.peer_ts_ms.is_none() && l.msg_id == in_reply_to) {
            return Err(SendError::BadRequest(format!("{conv}/{in_reply_to} was refused; it cannot be answered")));
        }
        let (_, payload) = self
            .store
            .read_record(conv, st.seq, in_reply_to)
            .map_err(|e| SendError::Store(e.to_string()))?;
        let orig = wire::parse_payload(&payload).map_err(|e| SendError::Store(format!("stored original: {e}")))?;
        let sent = self.send(&orig.from_agent, &orig.from_instance, Some(conv), body, None)?;
        let me = self.cfg.identity.fingerprint().to_owned();
        let extra = crate::store::LineExtra { reason: Some(format!("reply={}", sent.msg_id)), ..Default::default() };
        let id = in_reply_to.to_owned();
        let sender = st.sender.clone();
        self.store
            .append_event_unless(conv, "REPLIED", in_reply_to, &st.sender, &st.digest, extra, move |ls| {
                ls.iter().any(|l| l.event == "REPLIED" && l.peer_ts_ms.is_none() && l.msg_id == id && l.sender == sender)
            })
            .map_err(|e| SendError::Store(format!("reply SENT but REPLIED not recorded: {e}")))?;
        let at = self
            .store
            .chain_lines(conv)
            .ok()
            .and_then(|ls| ls.into_iter().find(|l| l.event == "REPLIED" && l.peer_ts_ms.is_none() && l.msg_id == in_reply_to))
            .and_then(|l| l.ts_ms)
            .unwrap_or_else(crate::store::now_ms);
        let stage = wire::Stage {
            from: me,
            from_agent: self.cfg.agent.clone(),
            to_agent: orig.from_agent.clone(),
            conv: conv.to_owned(),
            msg_id: in_reply_to.to_owned(),
            stage: "REPLIED".into(),
            ts_ms: at as i64,
            chain_head: self.store.chain_head(conv).unwrap_or_default(),
        };
        self.post_stage(&orig.from_agent, &stage)?;
        Ok(sent)
    }

    fn events_for(&self, conv: &str, msg_id: &str) -> Vec<String> {
        self.store
            .chain_lines(conv)
            .map(|ls| ls.into_iter().filter(|l| l.msg_id == msg_id).map(|l| l.event).collect())
            .unwrap_or_default()
    }

    /// Once per message, and never after it was posted.
    fn mark_waiting(&self, conv: &str, msg_id: &str, digest: &str) -> Result<(), StoreError> {
        if msg_id.is_empty() {
            // A stage record: it has no chain events of its own.
            return Ok(());
        }
        self.store
            .append_event_unless(conv, WAITING, msg_id, self.cfg.identity.fingerprint(), digest, Default::default(), |ls| {
                ls.iter().any(|l| l.msg_id == msg_id && (l.event == WAITING || l.event == POSTED))
            })
            .map(|_| ())
    }

    fn run_pump(&self) -> PumpResult {
        let mut mem = match self.pump.lock() {
            Ok(m) => m,
            Err(p) => p.into_inner(),
        };
        self.pump_locked(&mut mem)
    }

    fn pump_locked(&self, mem: &mut PumpMemory) -> PumpResult {
        if matches!(self.queue.peek_head_id(), Ok(None)) {
            mem.held = None;
            return PumpResult::Drained;
        }
        match self.check_pin() {
            Ok(()) => mem.held = None,
            Err(PinError::Unreachable(_)) => {
                self.wait_head();
                return PumpResult::Waiting;
            }
            Err(PinError::Mismatch { declared, found }) => {
                let why = format!(
                    "hub id mismatch: declared {declared}, hub says {}",
                    found.as_deref().unwrap_or("nothing")
                );
                if mem.held.as_deref() != Some(why.as_str()) {
                    let _ = self.store.attention("hub-id-mismatch", &why);
                }
                mem.held = Some(why.clone());
                return PumpResult::Held(why);
            }
        }
        loop {
            let (qid, post, attempts) = match self.queue.peek_oldest_with_attempts() {
                Ok(Some(t)) => t,
                Ok(None) => return PumpResult::Drained,
                Err(e) => {
                    let _ = self.store.attention("outbox-unreadable", &e.to_string());
                    return PumpResult::Rejected(format!("outbox unreadable: {e}"));
                }
            };
            if !mem.topics.contains(&post.topic) {
                match self.rpc(method::CHANNEL_CREATE, json!({"name": post.topic})) {
                    Err(_) => {
                        self.wait_head();
                        return PumpResult::Waiting;
                    }
                    Ok(RpcResponse::Success(_)) => {
                        mem.topics.insert(post.topic.clone());
                    }
                    Ok(RpcResponse::Error(e)) => {
                        // An already-existing topic is not an error worth stopping for.
                        if e.error.message.to_lowercase().contains("exist") {
                            mem.topics.insert(post.topic.clone());
                        } else {
                            return PumpResult::Rejected(format!("channel.create: {}", e.error.message));
                        }
                    }
                }
            }
            let conv = post.metadata.get("conversation_id").cloned().unwrap_or_default();
            let msg_id = post.metadata.get("sidecar_msg_id").cloned().unwrap_or_default();
            let digest = post.metadata.get("sidecar_digest").cloned().unwrap_or_default();
            match self.rpc(method::CHANNEL_POST, post_params(&post)) {
                Err(_) => {
                    let _ = self.mark_waiting(&conv, &msg_id, &digest);
                    return PumpResult::Waiting;
                }
                Ok(RpcResponse::Success(_)) => {
                    // POSTED before the row is removed: a crash in between re-posts
                    // with the same client_msg_id, which the hub absorbs.
                    if !msg_id.is_empty()
                        && !self.events_for(&conv, &msg_id).iter().any(|e| e == POSTED)
                        && let Err(e) = self.store.append_event(&conv, POSTED, &msg_id, self.cfg.identity.fingerprint(), &digest)
                    {
                        let _ = self.store.attention("posted-not-recorded", &format!("{msg_id}: {e}"));
                        return PumpResult::Rejected(format!("cannot record POSTED: {e}"));
                    }
                    // Responsibility stays here until the recipient reports STORED:
                    // the awaiting record is written before the outbox row goes.
                    if let Err(e) = self.keep_awaiting(&conv, &msg_id, &post) {
                        let _ = self.store.attention("awaiting-not-recorded", &format!("{msg_id}: {e}"));
                        return PumpResult::Rejected(format!("cannot keep the message awaiting STORED: {e}"));
                    }
                    let _ = self.queue.pop(qid);
                }
                Ok(RpcResponse::Error(e)) => {
                    use termlink_protocol::control::error_code;
                    if e.error.code == error_code::RATE_LIMITED || e.error.code == error_code::HUB_AT_CAPACITY {
                        let _ = self.mark_waiting(&conv, &msg_id, &digest);
                        return PumpResult::Waiting;
                    }
                    let reason = format!("hub rejected the post (code {}): {}", e.error.code, e.error.message);
                    if attempts + 1 >= POISON_THRESHOLD {
                        if self.queue.dead_letter(qid, &reason).is_ok() {
                            if msg_id.is_empty() {
                                let _ = self.store.attention("dead-letter", &format!("stage record for {conv}: {reason}"));
                                return PumpResult::Rejected(reason);
                            }
                            let _ = self.store.append_event(&conv, DEAD_LETTER, &msg_id, self.cfg.identity.fingerprint(), &digest);
                            let _ = self.store.attention("dead-letter", &format!("{msg_id}: {reason}"));
                        }
                    } else {
                        let _ = self.queue.bump_attempts(qid);
                    }
                    return PumpResult::Rejected(reason);
                }
            }
        }
    }

    fn awaiting_dir(&self) -> PathBuf {
        self.cfg.queue_path.with_extension("awaiting")
    }

    fn awaiting_path(&self, msg_id: &str) -> PathBuf {
        self.awaiting_dir().join(format!("{msg_id}.json"))
    }

    fn write_awaiting(&self, a: &Awaiting) -> std::io::Result<()> {
        use std::io::Write as _;
        let dir = self.awaiting_dir();
        std::fs::create_dir_all(&dir)?;
        let tmp = dir.join(format!(".{}.tmp", a.msg_id));
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(&serde_json::to_vec(a).map_err(std::io::Error::other)?)?;
        f.sync_all()?;
        std::fs::rename(&tmp, self.awaiting_path(&a.msg_id))?;
        std::fs::File::open(&dir)?.sync_all()
    }

    /// Every held record. A record that cannot be read or parsed is a held message
    /// too: it is moved aside (kept, under `.unreadable`) and raised as attention,
    /// never dropped. An awaiting dir that cannot be listed is an `Err`, not an
    /// empty list: that reads as "nothing held" otherwise. Attention is raised here.
    fn awaiting_records(&self) -> Result<AwaitingScan, String> {
        let dir = self.awaiting_dir();
        let rd = match std::fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(AwaitingScan::default()),
            Err(e) => {
                let why = format!("{}: cannot list the awaiting dir: {e}", dir.display());
                let _ = self.store.attention(ATTN_AWAITING_UNREADABLE, &why);
                return Err(why);
            }
        };
        let mut scan = AwaitingScan::default();
        for entry in rd {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    let _ = self.store.attention(ATTN_AWAITING_UNREADABLE, &format!("{}: cannot read a dir entry: {e}", dir.display()));
                    continue;
                }
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(UNREADABLE_SUFFIX) {
                scan.quarantined.push(name.trim_end_matches(UNREADABLE_SUFFIX).to_owned());
                continue;
            }
            if !name.ends_with(".json") {
                continue;
            }
            let path = entry.path();
            let parsed = std::fs::read(&path)
                .map_err(|e| e.to_string())
                .and_then(|bytes| serde_json::from_slice::<Awaiting>(&bytes).map_err(|e| e.to_string()));
            match parsed {
                Ok(a) => scan.records.push(a),
                Err(why) => {
                    scan.quarantined.push(name.trim_end_matches(".json").to_owned());
                    self.quarantine_awaiting(&path, &why);
                }
            }
        }
        scan.records.sort_by_key(|a| a.since_ms);
        Ok(scan)
    }

    /// Move an unreadable record aside so it is not re-read on every pump, and say so.
    fn quarantine_awaiting(&self, path: &std::path::Path, why: &str) {
        let aside = path.with_extension(UNREADABLE_SUFFIX.trim_start_matches('.'));
        let detail = match std::fs::rename(path, &aside) {
            Ok(()) => format!("{}: {why}; moved aside to {}", path.display(), aside.display()),
            Err(e) => format!("{}: {why}; could not move it aside: {e}", path.display()),
        };
        let _ = self.store.attention(ATTN_AWAITING_UNREADABLE, &detail);
    }

    /// Every message id still held: parseable records plus quarantined ones.
    fn awaiting_ids(&self) -> Vec<String> {
        match self.awaiting_records() {
            Ok(scan) => scan.records.into_iter().map(|a| a.msg_id).chain(scan.quarantined).collect(),
            Err(_) => vec![],
        }
    }

    /// At POSTED: start holding the message, unless it is held already (a resend
    /// keeps its count), stored already, or given up on.
    fn keep_awaiting(&self, conv: &str, msg_id: &str, post: &PendingPost) -> std::io::Result<()> {
        if msg_id.is_empty() || self.awaiting_path(msg_id).exists() {
            return Ok(());
        }
        let lines = self.store.chain_lines(conv).unwrap_or_default();
        let mine: Vec<_> = lines.iter().filter(|l| l.msg_id == msg_id).collect();
        if mine.iter().any(|l| l.event == NOT_ACCEPTED) || self.accepted(conv, msg_id) {
            return Ok(());
        }
        // Urgent per the SENT line's recorded priority, as the status view reads it.
        let urgent = mine
            .iter()
            .filter(|l| l.event == "SENT")
            .filter_map(|l| l.reason.as_deref()?.strip_prefix("priority=")?.parse::<i64>().ok())
            .any(|p| p >= 5);
        self.write_awaiting(&Awaiting {
            conv: conv.to_owned(),
            msg_id: msg_id.to_owned(),
            urgent,
            since_ms: crate::store::now_ms(),
            resends: 0,
            post: post.clone(),
        })
    }

    /// The recipient reported STORED, or a later stage, for this message.
    fn accepted(&self, conv: &str, msg_id: &str) -> bool {
        let stored = wire::STAGE_ORDER.iter().position(|s| *s == "STORED").unwrap_or(1);
        let me = self.cfg.identity.fingerprint();
        self.store.chain_lines(conv).unwrap_or_default().iter().any(|l| {
            l.msg_id == msg_id
                && l.peer_ts_ms.is_some()
                && l.sender != me
                && wire::STAGE_ORDER.iter().position(|s| *s == l.event).is_some_and(|r| r >= stored)
        })
    }

    fn resend_overdue(&self, now_ms: u64, dl: &crate::deadline::Deadlines) -> ResendReport {
        let mut rep = ResendReport::default();
        let mut mem = match self.pump.lock() {
            Ok(m) => m,
            Err(p) => p.into_inner(),
        };
        let me = self.cfg.identity.fingerprint().to_owned();
        let Ok(scan) = self.awaiting_records() else { return rep };
        for mut a in scan.records {
            let path = self.awaiting_path(&a.msg_id);
            let digest = a.post.metadata.get("sidecar_digest").cloned().unwrap_or_default();
            if self.accepted(&a.conv, &a.msg_id) {
                if std::fs::remove_file(&path).is_ok() {
                    rep.released.push(a.msg_id);
                }
                continue;
            }
            // Dead-lettered by the hub: that path already raised its own alarm.
            if self.events_for(&a.conv, &a.msg_id).iter().any(|e| e == DEAD_LETTER) {
                let _ = std::fs::remove_file(&path);
                continue;
            }
            // The gap doubles after each resend: deadline, 2x, 4x, ...
            let gap = dl.accept(a.urgent).saturating_mul(1u64 << a.resends.min(32));
            if now_ms.saturating_sub(a.since_ms) <= gap {
                continue;
            }
            if a.resends >= MAX_RESENDS {
                let why = format!(
                    "{}/{}: no STORED report after {} resends; the recipient has not accepted it",
                    a.conv, a.msg_id, a.resends
                );
                let extra = crate::store::LineExtra { reason: Some(format!("resends={}", a.resends)), ..Default::default() };
                let id = a.msg_id.clone();
                let written = self.store.append_event_unless(&a.conv, NOT_ACCEPTED, &a.msg_id, &me, &digest, extra, |ls| {
                    ls.iter().any(|l| l.msg_id == id && l.event == NOT_ACCEPTED)
                });
                if written.is_ok() && self.store.attention(ATTN_NOT_ACCEPTED, &why).is_ok() {
                    let _ = std::fs::remove_file(&path);
                    rep.gave_up.push(a.msg_id);
                }
                continue;
            }
            // Same post, same client_msg_id: the hub's dedupe and the recipient's
            // Duplicate outcome absorb a copy that did arrive. Posted directly, never
            // queued: a queued copy piles up behind a hub that is down (T-3511).
            if !self.resend_reached_hub(&mut mem, &a.post) {
                // Not an attempt the hub could take, so it does not count toward
                // giving up. The timer restarts, so a dead hub is retried on the gap.
                a.since_ms = now_ms;
                if let Err(e) = self.write_awaiting(&a) {
                    let _ = self.store.attention("awaiting-not-recorded", &format!("{}: {e}", a.msg_id));
                }
                continue;
            }
            a.resends += 1;
            a.since_ms = now_ms;
            let extra = crate::store::LineExtra { reason: Some(format!("attempt={}", a.resends)), ..Default::default() };
            let _ = self.store.append_event_unless(&a.conv, RESENT, &a.msg_id, &me, &digest, extra, |_| false);
            if let Err(e) = self.write_awaiting(&a) {
                let _ = self.store.attention("awaiting-not-recorded", &format!("{}: {e}", a.msg_id));
            }
            rep.resent.push(a.msg_id);
        }
        if !rep.resent.is_empty() {
            let _ = self.pump_locked(&mut mem);
        }
        rep
    }

    /// One resend attempt. True when the hub answered it, whether it accepted the post
    /// or refused it, since a refusal is what counts toward giving up. False when the
    /// hub could not take it: no connection, or RATE_LIMITED / HUB_AT_CAPACITY, which
    /// say "not now" rather than "no" (T-3511).
    fn resend_reached_hub(&self, mem: &mut PumpMemory, post: &PendingPost) -> bool {
        use termlink_protocol::control::error_code;
        if !mem.topics.contains(&post.topic) {
            match self.rpc(method::CHANNEL_CREATE, json!({"name": post.topic})) {
                Err(_) => return false,
                Ok(RpcResponse::Success(_)) => {
                    mem.topics.insert(post.topic.clone());
                }
                Ok(RpcResponse::Error(e)) => {
                    if e.error.message.to_lowercase().contains("exist") {
                        mem.topics.insert(post.topic.clone());
                    }
                }
            }
        }
        match self.rpc(method::CHANNEL_POST, post_params(post)) {
            Err(_) => false,
            Ok(RpcResponse::Error(e))
                if e.error.code == error_code::RATE_LIMITED || e.error.code == error_code::HUB_AT_CAPACITY =>
            {
                false
            }
            Ok(_) => true,
        }
    }

    /// The hub is out of reach: the head message shows WAITING.
    fn wait_head(&self) {
        if let Ok(Some((_, post, _))) = self.queue.peek_oldest_with_attempts() {
            let conv = post.metadata.get("conversation_id").cloned().unwrap_or_default();
            let msg_id = post.metadata.get("sidecar_msg_id").cloned().unwrap_or_default();
            let digest = post.metadata.get("sidecar_digest").cloned().unwrap_or_default();
            let _ = self.mark_waiting(&conv, &msg_id, &digest);
        }
    }
}
