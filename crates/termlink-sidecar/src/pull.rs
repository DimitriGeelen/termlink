//! Receive path: reads the agent's inbox topic from a durable cursor, verifies each message and records it in the store.
//! C-6 receive path (architecture note sections 4.3, 5.2, 5.3, 5.5; table 13 B7).
//!
//! A `Receiver` reads this agent's own inbox topic from a durable cursor. For a
//! message (`sidecar.msg.v1`) it re-verifies the signature (`wire::verify`:
//! roster key, signed `from`, own instance id), asks the policy hook, records
//! RECEIVED, stores the record (`Store::put`, steps 2 to 7 of 4.3) and only then
//! reports STORED. Each stage goes back to the sender as a signed stage record
//! (`sidecar.stage.v1`) through the same durable outbox the send path uses.
//!
//! For a stage record addressed to this agent it does the sender's half: verify
//! against the roster, accept it only for a message this agent SENT, keep the
//! stage forward-only, and record it with the peer's own time.
//!
//! The hub verifies a post's signature but does not return it on subscribe, so
//! the sender carries signer key and signature in the post metadata
//! (`wire::META_PK`, `wire::META_SIG`). Metadata is outside the signed bytes:
//! changing it can only make verification fail, never pass.
//!
//! The cursor moves past a message only once it is fully handled (stored and
//! both stages queued, or refused and recorded). A failure leaves the cursor, so
//! the next pass retries; every step is idempotent, so a retry or a restart adds
//! no second RECEIVED or STORED line, and a re-sent stage record is absorbed by
//! the sender's forward-only check.

use crate::policy::{self, Policy};
use crate::send::{PinError, Sender, inbox_topic};
use crate::store::{ChainLine, LineExtra, PutOutcome, Store};
use crate::wire::{self, Msg, Refusal, Roster, Signed, Stage};
use base64::Engine as _;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use termlink_protocol::TransportAddr;
use termlink_protocol::control::{error_code, method};
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_session::client::{rpc_call_addr_with_timeout, unwrap_result};

pub const RECEIVED: &str = "RECEIVED";
pub const STORED: &str = "STORED";
pub const REFUSED: &str = "REFUSED";
/// T-3703 (r3 OC 3): reason for a verified message whose signer key fingerprint cannot be derived.
pub const BAD_SIGNER_KEY: &str = "bad-signer-key";
/// Chain event and notice: held for this side's operator (OQ-2 ruling E).
pub const HELD: &str = "HELD";
/// Notice: the operator refused the sender.
pub const DENIED: &str = "DENIED";
/// Notice: the operator admitted the sender.
pub const ADMITTED: &str = "ADMITTED";
/// Needs-attention kind for a held message.
pub const HELD_ALERT: &str = "held-for-operator";
/// Store state prefix: `held-key-<fingerprint>` holds the knocking key (64 hex).
pub const HELD_KEY_PREFIX: &str = "held-key-";
/// The store state file holding the next inbox offset to read.
pub const CURSOR_STATE: &str = "inbox-cursor";
/// T-3823: the key cards this sidecar has seen ("agent:seq:signing key" per line) and the check setting it last saw.
const CARDS_SEEN_STATE: &str = "key-cards-seen";
const CARD_MODE_SEEN_STATE: &str = "key-card-mode-seen";
/// T-3704: the last refusal count the hub reported for this agent (impersonation attention).
pub const IMPERSONATION_STATE: &str = "key-card-refusals-seen";
/// Conversation name used for a refused message whose own conv is unusable.
pub const UNPARSED_CONV: &str = "_refused";

pub struct ReceiveConfig {
    /// This agent's name: its inbox is `inbox:<hub_id>/<agent>`.
    pub agent: String,
    /// This agent's exact instance id (SI-26).
    pub instance: String,
    pub hub: TransportAddr,
    /// The hub id this sidecar is configured to talk to (FX-2).
    pub declared_hub_id: String,
    /// Senders whose keys are accepted (PH-1: filled by the harness).
    pub roster: Roster,
    pub timeout: Duration,
    /// Messages read per hub call.
    pub page: u64,
    /// How often the background loop reads. `None` disables it (tests drive `pass`).
    pub poll_interval: Option<Duration>,
}

/// What one pass ended on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassResult {
    /// Read to the end of the inbox; `handled` envelopes processed this pass.
    CaughtUp { handled: usize },
    /// The hub could not be reached.
    Waiting,
    /// The hub is not the declared one; nothing was read.
    Held(String),
    /// A message could not be finished (e.g. the store failed); retried next pass.
    Stalled { handled: usize, reason: String },
    /// Drained by the operator (PH-2 row P4): nothing was read; new mail stays on
    /// the hub and the cursor does not move.
    Drained,
}

struct Inner {
    cfg: ReceiveConfig,
    store: Arc<Store>,
    sender: Arc<Sender>,
    policy: Box<dyn Policy>,
    rt: tokio::runtime::Runtime,
    pass: Mutex<PassMemory>,
    stop: AtomicBool,
    /// Last time the background loop came round (ms since the epoch); the
    /// component heartbeat (K-3, FP-1 failure 4).
    beat: AtomicU64,
    /// T-3703 (AL-22): the revocation pull, run at the start of every pass.
    revocation: Mutex<crate::revocation::Tracker>,
}

#[derive(Default)]
struct PassMemory {
    /// T-3700: when the consumer list was last checked (ms).
    consumers_checked_ms: u64,
    held: Option<String>,
    /// The last per-message failure written to attention, so a stuck message
    /// is reported once rather than every pass.
    stalled: Option<String>,
}

pub struct Receiver {
    inner: Arc<Inner>,
    thread: Option<JoinHandle<()>>,
}

/// One envelope as `channel.subscribe` returns it.
struct Envelope {
    offset: u64,
    msg_type: String,
    sender_id: String,
    signed: Option<Signed>,
    payload: Vec<u8>,
}

enum Outcome {
    /// Fully handled: the cursor may move past it.
    Done,
    /// Not finished; stop the pass and retry from here.
    Retry(String),
}

fn name_ok(n: &str) -> bool {
    !n.is_empty()
        && n != "."
        && n != ".."
        && n.len() <= 200
        && n.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':'))
}

fn parse_envelope(v: &Value) -> Option<Envelope> {
    let offset = v.get("offset")?.as_u64()?;
    let payload = base64::engine::general_purpose::STANDARD
        .decode(v.get("payload_b64").and_then(Value::as_str).unwrap_or(""))
        .unwrap_or_default();
    let meta = v.get("metadata");
    let m = |k: &str| meta.and_then(|m| m.get(k)).and_then(Value::as_str).map(str::to_owned);
    let signed = match (m(wire::META_PK), m(wire::META_SIG)) {
        (Some(pk), Some(sig)) => Some(Signed {
            topic: v.get("topic").and_then(Value::as_str).unwrap_or("").to_owned(),
            ts_ms: v.get("ts").and_then(Value::as_i64).unwrap_or(0),
            payload: payload.clone(),
            signer_pk: pk,
            sig,
        }),
        _ => None,
    };
    Some(Envelope {
        offset,
        msg_type: v.get("msg_type").and_then(Value::as_str).unwrap_or("").to_owned(),
        sender_id: v.get("sender_id").and_then(Value::as_str).unwrap_or("").to_owned(),
        signed,
        payload,
    })
}

impl Receiver {
    pub fn new(
        cfg: ReceiveConfig,
        store: Arc<Store>,
        sender: Arc<Sender>,
        policy: Box<dyn Policy>,
    ) -> std::io::Result<Receiver> {
        let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
        let interval = cfg.poll_interval;
        let revocation = Mutex::new(crate::revocation::Tracker::new(&store, &cfg.declared_hub_id, crate::revocation::PN13));
        // T-3700 (r1 CC 2): a consumers line from before this start is never shown
        crate::consumers::reset(&store);
        let inner = Arc::new(Inner {
            cfg,
            store,
            sender,
            policy,
            rt,
            pass: Mutex::new(PassMemory::default()),
            stop: AtomicBool::new(false),
            beat: AtomicU64::new(crate::store::now_ms()),
            revocation,
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
                        i.run_pass();
                    }
                }
            })
        });
        Ok(Receiver { inner, thread })
    }

    /// Read and handle everything new on the inbox (the background loop runs the same).
    pub fn pass(&self) -> PassResult {
        self.inner.run_pass()
    }

    /// When the background receive loop last came round (ms since the epoch).
    pub fn last_beat_ms(&self) -> u64 {
        self.inner.beat.load(Ordering::SeqCst)
    }

    /// The next inbox offset this receiver will read.
    pub fn cursor(&self) -> u64 {
        self.inner.cursor()
    }

    pub fn topic(&self) -> String {
        inbox_topic(&self.inner.cfg.declared_hub_id, &self.inner.cfg.agent)
    }

    /// T-3703: PN-13 for this receiver (tests shorten it; the daemon keeps 1 hour).
    pub fn set_revocation_stale_after(&self, d: std::time::Duration) {
        self.inner.revocation.lock().unwrap_or_else(|p| p.into_inner()).set_stale_after(d);
    }

    /// T-3703: the revocation pull's state (tests and status).
    pub fn revocation_state(&self) -> crate::revocation::State {
        self.inner.revocation.lock().unwrap_or_else(|p| p.into_inner()).state().clone()
    }

    /// T-3700: run the consumer check now, whatever the throttle (tests, and an operator's `status` refresh).
    pub fn check_consumers_now(&self) {
        let mut mem = self.inner.pass.lock().unwrap_or_else(|p| p.into_inner());
        mem.consumers_checked_ms = 0;
        self.inner.check_consumers(&mut mem);
    }
}

impl Drop for Receiver {
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

    fn check_pin(&self) -> Result<Value, PinError> {
        let before = crate::store::now_ms();
        let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
        let after = crate::store::now_ms();
        let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
        // T-3708 (V-5): every pass is a skew sample, also for an agent whose inbox has never had mail
        crate::host::record_round_trip(before, after, v.get("hub_now_ms").and_then(Value::as_u64));
        let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
        crate::hub_key_pin::observe_hub_id(&self.store, &self.cfg.declared_hub_id, found.as_deref());
        if found.as_deref() == Some(self.cfg.declared_hub_id.as_str()) {
            // T-3366 (AL-18): pin / compare the hub's signing key; never holds mail.
            let trusted = matches!(self.cfg.hub, termlink_protocol::TransportAddr::Unix { .. });
            crate::hub_key_pin::observe(&self.store, &self.cfg.declared_hub_id, &v, trusted);
            Ok(v)
        } else {
            Err(PinError::Mismatch { declared: self.cfg.declared_hub_id.clone(), found })
        }
    }

    /// T-3704 (B-6, design v2 C-12): the hub counts arc-011 posts refused (or, in log mode, that
    /// would be refused) for claiming to be this agent. When that count has grown since the last
    /// tick, raise one needs-attention entry per day naming the counts. Best effort: an old hub
    /// without `hub.key_card_list` or any read error changes nothing and never holds mail.
    fn observe_impersonation(&self) {
        // transport errors are the pin check's to report; this reads only an answering hub
        let Ok(resp) = self.rpc("hub.key_card_list", json!({})) else { return };
        let v = match resp {
            RpcResponse::Success(s) => s.result,
            // a hub that predates key cards: method not found, or (older hubs) forwarded and refused for a target
            RpcResponse::Error(e) if e.error.code == -32601 || e.error.message.contains("Missing 'target'") => return,
            RpcResponse::Error(e) => {
                // T-3823 (T-3704 r3): an unreadable card list is said, once an hour, never silently skipped
                let hour = crate::store::now_ms() / 3_600_000;
                let detail = format!("hub.key_card_list failed ({}): {}; card writes and refusals cannot be watched", e.error.code, e.error.message);
                let _ = self.store.attention_coalesced("key-card-list-unreadable", &hour.to_string(), &detail);
                return;
            }
        };
        let day = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / 86_400)
            .unwrap_or(0);
        let start = v["hub_start_ms"].as_u64().unwrap_or(0);
        // (1) refusals of posts claiming to be this agent, NOT signed by this agent's own key
        //     (its own posts refused during the P-31 roll-out are not an impersonation, r2 MINOR 11)
        let me = self.sender.fingerprint().to_string();
        let r = &v["refusals"][self.cfg.agent.as_str()];
        let total: u64 = r["signers"]
            .as_object()
            .map(|m| m.iter().filter(|(fp, _)| **fp != me).filter_map(|(_, n)| n.as_u64()).sum())
            .unwrap_or(0);
        let now = format!("{start}:{total}");
        let seen = self.store.read_state(IMPERSONATION_STATE).ok().flatten().unwrap_or_default();
        if now != seen.trim() {
            let _ = self.store.write_state(IMPERSONATION_STATE, &now);
            let (seen_start, seen_total) = seen
                .trim()
                .split_once(':')
                .map(|(a, b)| (a.parse::<u64>().unwrap_or(0), b.parse::<u64>().unwrap_or(0)))
                .unwrap_or((0, 0));
            // a new hub start resets its counts: compare against 0, so none are lost
            let base = if seen_start == start { seen_total } else { 0 };
            if total > base {
                let detail = format!(
                    "posts claiming to be agent {} were refused at the hub by its key card ({} since the hub started, signed by keys other than this agent's); see `termlink hub key-card list` and rpc-admin.jsonl",
                    self.cfg.agent, total
                );
                if let Err(e) = self.store.attention_coalesced("posts-claiming-to-be-you-refused", &day.to_string(), &detail) {
                    eprintln!("termlink-sidecar: needs attention (posts-claiming-to-be-you-refused): {detail} [could not record it: {e}]");
                }
            }
        }
        // (2) every card write is the operator's to know (ruling A): one entry per (agent, seq, signing key). The
        //     keys seen are persisted; a sidecar's first look records the existing cards as its baseline and raises
        //     nothing for them (T-3823, T-3704 r3).
        let seen_text = self.store.read_state(CARDS_SEEN_STATE).ok().flatten();
        let baseline = seen_text.is_none();
        let mut seen: std::collections::BTreeSet<String> = seen_text.unwrap_or_default().lines().map(str::to_string).collect();
        let mut grew = false;
        for c in v["cards"].as_array().into_iter().flatten() {
            let (Some(agent), Some(seq), Some(key)) = (c["agent"].as_str(), c["seq"].as_u64(), c["signing_key"].as_str()) else {
                continue;
            };
            let id = format!("{agent}:{seq}:{key}");
            if seen.contains(&id) {
                continue;
            }
            if !baseline {
                let detail = format!("key card for agent {agent} is at seq {seq} (key {key}); written via hub.key_card_put, see rpc-admin.jsonl for old and new values");
                let _ = self.store.attention_coalesced("key-card-written", &id, &detail);
            }
            seen.insert(id);
            grew = true;
        }
        if grew || baseline {
            let _ = self.store.write_state(CARDS_SEEN_STATE, &seen.into_iter().collect::<Vec<_>>().join("\n"));
        }
        // (3) a change of the check setting (off|log|enforce) is the operator's to know too (T-3823, T-3704 r3)
        if let Some(mode) = v["mode"].as_str() {
            let was = self.store.read_state(CARD_MODE_SEEN_STATE).ok().flatten().map(|s| s.trim().to_string());
            if was.as_deref() != Some(mode) {
                if let Some(w) = &was {
                    let _ = self.store.attention(
                        "key-card-mode-changed",
                        &format!("the hub's key-card check moved {w} -> {mode} (see rpc-admin.jsonl key-card-mode)"),
                    );
                }
                let _ = self.store.write_state(CARD_MODE_SEEN_STATE, mode);
            }
        }
    }

    /// T-3703 (r2 CC 1): the hub could not be asked this pass; the stale rule still counts it.
    fn revocation_outage(&self, why: &str) {
        let mut t = self.revocation.lock().unwrap_or_else(|p| p.into_inner());
        t.outage(&self.store, std::time::Instant::now(), why);
    }

    /// T-3703 (AL-22): one revocation pull. A failure never holds mail (the stale rule narrows, never stops).
    fn pull_revocations(&self, version: &Value) {
        let rpc = |m: &str| -> Result<Value, (i64, String)> {
            match self.rpc(m, json!({})) {
                Err(e) => Err((0, e)),
                Ok(RpcResponse::Success(s)) => Ok(s.result),
                Ok(RpcResponse::Error(e)) => Err((e.error.code, e.error.message)),
            }
        };
        let mut t = self.revocation.lock().unwrap_or_else(|p| p.into_inner());
        t.tick(&self.store, version, &rpc, std::time::Instant::now(), Some(&self.sender));
    }

    /// T-3700 (AL-30, E-66): ask the hub who consumed this inbox within the V-11 window and record it.
    fn check_consumers(&self, mem: &mut PassMemory) {
        let now = crate::store::now_ms();
        if mem.consumers_checked_ms != 0 && now < mem.consumers_checked_ms + crate::consumers::CHECK_EVERY_MS {
            return;
        }
        mem.consumers_checked_ms = now;
        let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
        let answer = self.rpc("inbox.consumers", json!({"topic": topic, "window_s": crate::consumers::WINDOW_S}));
        let fp = self.sender.key_fp();
        let (me, inst) = (&self.cfg.agent, &self.cfg.instance);
        let v = match &answer {
            Ok(RpcResponse::Success(s)) => crate::consumers::verdict(me, inst, &fp, Ok(&s.result)),
            Ok(RpcResponse::Error(e)) => crate::consumers::verdict(me, inst, &fp, Err((e.error.code, &e.error.message))),
            Err(e) => crate::consumers::verdict(me, inst, &fp, Err((0, &format!("hub unreachable: {e}")))),
        };
        crate::consumers::record(&self.store, &topic, &v, now);
        // T-3708 (FCW-120): the check ran; its value says when it could not see (`unknown (...)`)
        crate::fields::touch(crate::fields::CONSUMERS);
    }

    fn cursor(&self) -> u64 {
        self.store
            .read_state(CURSOR_STATE)
            .ok()
            .flatten()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
    }

    /// A needs-attention entry; when even that cannot be written (the store's
    /// disk is full, say), the entry goes to stderr, i.e. the sidecar's log, so
    /// the failure is never silent (H-17).
    fn alert(&self, kind: &str, detail: &str) {
        if let Err(e) = self.store.attention(kind, detail) {
            eprintln!("termlink-sidecar: needs attention ({kind}): {detail} [could not record it: {e}]");
        }
    }

    fn me(&self) -> &str {
        self.sender.fingerprint()
    }

    fn run_pass(&self) -> PassResult {
        let mut mem = match self.pass.lock() {
            Ok(m) => m,
            Err(p) => p.into_inner(),
        };
        self.pass_locked(&mut mem)
    }

    fn pass_locked(&self, mem: &mut PassMemory) -> PassResult {
        if !policy::receiving_enabled(&self.store) {
            return PassResult::Drained;
        }
        let version = match self.check_pin() {
            Ok(version) => {
                mem.held = None;
                // T-3708 (FCW-120): the hub was read; `hub` and `revocation` are fresh from this pass.
                crate::fields::touch(crate::fields::PULL);
                self.observe_impersonation();
                version
            }
            Err(PinError::Unreachable(e)) => {
                self.revocation_outage(&format!("hub unreachable: {e}"));
                return PassResult::Waiting;
            }
            Err(PinError::Mismatch { declared, found }) => {
                self.revocation_outage("hub id mismatch");
                let why = format!(
                    "hub id mismatch: declared {declared}, hub says {}; inbox not read",
                    found.as_deref().unwrap_or("nothing")
                );
                if mem.held.as_deref() != Some(why.as_str()) {
                    self.alert("hub-id-mismatch", &why);
                }
                mem.held = Some(why.clone());
                return PassResult::Held(why);
            }
        };
        // T-3703 (design 5.4): the revocation head is compared before any message is handled.
        self.pull_revocations(&version);
        // T-3700 (AL-30): who else consumes this inbox (at most every CHECK_EVERY_MS)
        self.check_consumers(mem);
        let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
        let mut handled = 0;
        loop {
            let cursor = self.cursor();
            let before = crate::store::now_ms();
            let resp = match self.rpc(
                method::CHANNEL_SUBSCRIBE,
                // T-3700 (AL-30, ruling E): a named, signed read of our own inbox (proven consumer)
                json!({"topic": topic, "cursor": cursor, "limit": self.cfg.page,
                       "read_proof": self.sender.read_proof(&topic, method::CHANNEL_SUBSCRIBE, cursor)}),
            ) {
                Ok(r) => r,
                Err(_) => return PassResult::Waiting,
            };
            // T-3708 (V-5): this round trip is a skew sample when the hub reports its time
            if let RpcResponse::Success(s) = &resp {
                crate::host::record_round_trip(before, crate::store::now_ms(), s.result.get("hub_now_ms").and_then(Value::as_u64));
            }
            let page = match resp {
                RpcResponse::Success(s) => s.result,
                RpcResponse::Error(e) if e.error.code == error_code::CHANNEL_TOPIC_UNKNOWN => {
                    // Nobody has written to this inbox yet.
                    return PassResult::CaughtUp { handled };
                }
                RpcResponse::Error(e) => {
                    return PassResult::Stalled { handled, reason: format!("channel.subscribe: {}", e.error.message) };
                }
            };
            let msgs = page.get("messages").and_then(Value::as_array).cloned().unwrap_or_default();
            if msgs.is_empty() {
                return PassResult::CaughtUp { handled };
            }
            for raw in &msgs {
                let Some(env) = parse_envelope(raw) else {
                    self.alert("inbox-unreadable", "envelope without an offset");
                    return PassResult::Stalled { handled, reason: "envelope without an offset".into() };
                };
                if env.offset < cursor {
                    continue;
                }
                // A drain set mid-pass stops before the next message: it stays on
                // the hub and the cursor does not move past it (PH-2 row P4).
                if !policy::receiving_enabled(&self.store) {
                    return PassResult::Drained;
                }
                match self.handle(&env) {
                    Outcome::Done => {
                        if let Err(e) = self.store.write_state(CURSOR_STATE, &(env.offset + 1).to_string()) {
                            return PassResult::Stalled { handled, reason: format!("cursor: {e}") };
                        }
                        mem.stalled = None;
                        handled += 1;
                    }
                    Outcome::Retry(reason) => {
                        let key = format!("{}: {reason}", env.offset);
                        if mem.stalled.as_deref() != Some(key.as_str()) {
                            self.alert("receive-stalled", &format!("inbox offset {key}"));
                        }
                        mem.stalled = Some(key);
                        return PassResult::Stalled { handled, reason };
                    }
                }
            }
        }
    }

    fn handle(&self, env: &Envelope) -> Outcome {
        match env.msg_type.as_str() {
            wire::MSG_TYPE => self.handle_msg(env),
            wire::STAGE_TYPE => self.handle_stage(env),
            other => {
                self.alert(
                    "inbox-unknown-type",
                    &format!("offset {}: msg_type {other:?} from {} skipped", env.offset, env.sender_id),
                );
                Outcome::Done
            }
        }
    }

    // ---- messages (receiver side) ------------------------------------------

    fn handle_msg(&self, env: &Envelope) -> Outcome {
        let verified = match &env.signed {
            None => Err(Refusal::Malformed("no signature in the post metadata")),
            Some(s) => wire::verify_open(s, &self.cfg.roster, &self.cfg.instance),
        };
        let msg = match verified {
            Ok((m, _signer)) => m,
            Err(r) => return self.refuse(env, None, wire::refusal_code(&r), &r.to_string()),
        };
        if !name_ok(&msg.conv) || !name_ok(&msg.msg_id) {
            return self.refuse(env, None, "bad-name", "conv or msg_id is not a valid name");
        }
        // T-3703 (FCW-99, E-39): a message signed by a key on the held revocation list is refused, not stored,
        // and the sender gets a signed REVOKED notice. The fingerprint is of the key whose signature `verify_open`
        // just checked (the envelope's signer key, never a roster entry looked up by the claimed name, r3 OC 1); one
        // that cannot be derived is refused under its own reason, never passed (r2 CC 2, r3 OC 3).
        let fp = env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk));
        let Some(fp) = fp else {
            return self.refuse(env, Some(&msg), BAD_SIGNER_KEY, "signer key fingerprint cannot be derived");
        };
        if crate::revocation::revoked(&self.store, &fp) {
            let out = self.refuse(env, Some(&msg), crate::revocation::REASON_REVOKED, &format!("signed by revoked key {fp}"));
            if matches!(out, Outcome::Done) {
                crate::revocation::count_refusal(&self.store);
                // best effort: a notice that cannot be queued never holds the inbox (r2 CC 8)
                if let Err(e) = self.report(&msg, crate::revocation::REVOKED) {
                    eprintln!("termlink-sidecar: REVOKED notice for {}/{} not queued: {e}", msg.conv, msg.msg_id);
                }
            }
            return out;
        }
        // Receiver checks (C-7). Their inputs are the verified sender and the
        // signed priority only.
        let a = self.policy.assess(&msg);
        let digest = msg.digest();
        let (conv, id, from) = (msg.conv.as_str(), msg.msg_id.as_str(), msg.from.as_str());
        let once = |event: &'static str, reason: Option<&str>| {
            let d = digest.clone();
            self.store.append_event_unless(
                conv,
                event,
                id,
                from,
                &digest,
                LineExtra { reason: reason.map(str::to_owned), ..Default::default() },
                move |ls| ls.iter().any(|l| l.event == event && l.msg_id == id && l.sender == from && l.digest == d),
            )
        };
        if let Err(e) = once(RECEIVED, None) {
            return Outcome::Retry(format!("RECEIVED not recorded: {e}"));
        }
        if a.downgraded {
            // Asked for urgent without the allow-list: normal, and said so (SI-18).
            let why = policy::downgrade_why(a.priority, a.trust, crate::revocation::narrowing(&self.store));
            if let Err(e) = once(policy::DOWNGRADED, Some(&why)) {
                return Outcome::Retry(format!("DOWNGRADED not recorded: {e}"));
            }
        }
        // Every sender hears back, including one outside the allow-list (OQ-2
        // ruling E): the knock tells it the receiver is alive and holding its
        // message for an operator, so silence never stands in for a decision.
        // The notices go to the inbox the message names; a forger can make this
        // receiver post at most one notice set per message it sends itself, and
        // the named agent's sidecar ignores (and shows) stage records for
        // messages it did not send.
        if let Err(e) = self.report(&msg, RECEIVED) {
            return Outcome::Retry(e);
        }
        match self.store.put(from, conv, id, &env.payload) {
            Ok(PutOutcome::Stored { .. }) | Ok(PutOutcome::Duplicate { .. }) => {
                if let Err(e) = self.report(&msg, STORED) {
                    return Outcome::Retry(e);
                }
                match a.withheld {
                    None => Outcome::Done,
                    Some(policy::AWAITING_OPERATOR) => self.hold(env, &msg),
                    Some(code) => {
                        // Refused by this side's operator: stored, shown, never delivered.
                        match once(REFUSED, Some(code)) {
                            Ok(Some(_)) => self.alert(
                                code,
                                &format!("{conv}/{id} from {from}: stored, not delivered: the operator refused this sender"),
                            ),
                            Ok(None) => {}
                            Err(e) => return Outcome::Retry(format!("REFUSED not recorded: {e}")),
                        }
                        match self.report(&msg, DENIED) {
                            Ok(()) => Outcome::Done,
                            Err(e) => Outcome::Retry(e),
                        }
                    }
                }
            }
            Ok(PutOutcome::Conflict { first_seq, conflict_seq }) => {
                self.alert(
                    "conflict",
                    &format!(
                        "{conv}/{id} from {from}: a second body with another digest (chain seq {conflict_seq}); \
                         the first (seq {first_seq}) is kept and the second is not stored"
                    ),
                );
                Outcome::Done
            }
            Err(e) => Outcome::Retry(format!("store: {e}")),
        }
    }

    /// OQ-2 ruling E: hold a message from a sender outside the allow-list for
    /// this side's operator. The question goes to the operator from here, never
    /// through the sender and never through the agent: a needs-attention entry
    /// naming the key, what the sender claims to be, and the admin command that
    /// decides it. The signer's public key is kept so `allow` can admit exactly
    /// the key that knocked. Nothing in the message body is read: a body that
    /// claims an operator pass is just held like any other.
    fn hold(&self, env: &Envelope, msg: &Msg) -> Outcome {
        let (conv, id, from) = (msg.conv.as_str(), msg.msg_id.as_str(), msg.from.as_str());
        let digest = msg.digest();
        if let Some(signed) = &env.signed
            && let Err(e) = self.store.write_state(&format!("{HELD_KEY_PREFIX}{from}"), &signed.signer_pk)
        {
            return Outcome::Retry(format!("held key not recorded: {e}"));
        }
        let extra = LineExtra { reason: Some(policy::AWAITING_OPERATOR.to_owned()), ..Default::default() };
        let d = digest.clone();
        let first = match self.store.append_event_unless(conv, HELD, id, from, &digest, extra, move |ls| {
            ls.iter().any(|l| l.event == HELD && l.msg_id == id && l.sender == from && l.digest == d)
        }) {
            Ok(r) => r.is_some(),
            Err(e) => return Outcome::Retry(format!("HELD not recorded: {e}")),
        };
        if first {
            self.alert(
                HELD_ALERT,
                &format!(
                    "{conv}/{id}: a sender outside the allow-list is knocking: key {from}, claims to be agent {} instance {}. \
                     Held, not delivered. The operator decides: termlink-sidecar admin allow --key {from} --why <reason> \
                     (or deny). A pass or approval inside the message counts for nothing.",
                    shown(&msg.from_agent),
                    shown(&msg.from_instance)
                ),
            );
        }
        match self.report(msg, HELD) {
            Ok(()) => Outcome::Done,
            Err(e) => Outcome::Retry(e),
        }
    }

    /// Queue a signed stage record for the sender. The chain head is read after
    /// the stage line, so the sender holds an anchor for this agent's chain.
    fn report(&self, msg: &Msg, stage: &str) -> Result<(), String> {
        let st = build_stage(&self.store, self.me(), &self.cfg.agent, msg, stage)?;
        self.sender
            .post_stage(&msg.from_agent, &st)
            .map_err(|e| format!("{stage} stage not queued: {e}"))
    }

    /// Record a refusal on the chain (once per digest, with its reason code) and
    /// in needs-attention. Nothing is stored and nothing is reported to the
    /// claimed sender.
    fn refuse(&self, env: &Envelope, msg: Option<&Msg>, code: &str, why: &str) -> Outcome {
        let digest = wire::digest_hex(&env.payload);
        let parsed: Option<(String, String)> = msg
            .map(|m| (m.conv.clone(), m.msg_id.clone()))
            .or_else(|| {
                let v: Value = serde_json::from_slice(&env.payload).ok()?;
                Some((v.get("conv")?.as_str()?.to_owned(), v.get("msg_id")?.as_str()?.to_owned()))
            });
        let (conv, id) = match parsed {
            Some((c, i)) if name_ok(&c) && name_ok(&i) => (c, i),
            _ => (UNPARSED_CONV.to_owned(), digest[..32].to_owned()),
        };
        let sender = if name_ok(&env.sender_id) { env.sender_id.clone() } else { "unknown".to_owned() };
        let extra = LineExtra { reason: Some(code.to_owned()), ..Default::default() };
        let first = match self.store.append_event_unless(&conv, REFUSED, &id, &sender, &digest, extra, |ls| {
            ls.iter().any(|l| l.event == REFUSED && l.msg_id == id && l.digest == digest)
        }) {
            Ok(r) => r.is_some(),
            Err(e) => return Outcome::Retry(format!("REFUSED not recorded: {e}")),
        };
        if first {
            self.alert(
                "refused",
                &format!("inbox offset {}: {conv}/{id} from {sender}: {why}", env.offset),
            );
        }
        Outcome::Done
    }

    // ---- stage records (sender side) ---------------------------------------

    fn handle_stage(&self, env: &Envelope) -> Outcome {
        let verified = match &env.signed {
            None => Err(Refusal::Malformed("no signature in the post metadata")),
            Some(s) => {
                // Peers this side's operator admitted report stages too (OQ-2 E).
                let mut roster = self.cfg.roster.clone();
                roster.extend(&policy::admitted_keys(&self.store));
                wire::verify_stage(s, &roster)
            }
        };
        let st = match verified {
            Ok(s) => s,
            Err(r) => return self.ignore_stage(env, &r.to_string()),
        };
        match env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk)) {
            None => return self.ignore_stage(env, "signer key fingerprint cannot be derived (FCW-99)"),
            Some(fp) if crate::revocation::revoked(&self.store, &fp) => {
                return self.ignore_stage(env, &format!("signed by revoked key {fp} (FCW-99)"));
            }
            Some(_) => {}
        }
        if st.to_agent != self.cfg.agent {
            return self.ignore_stage(env, &format!("addressed to {}, this is {}", st.to_agent, self.cfg.agent));
        }
        if !name_ok(&st.conv) || !name_ok(&st.msg_id) || !name_ok(&st.from) {
            return self.ignore_stage(env, "conv, msg_id or from is not a valid name");
        }
        let me = self.me().to_owned();
        let sent: Option<ChainLine> = match self.store.chain_lines(&st.conv) {
            Ok(ls) => ls.into_iter().find(|l| l.event == "SENT" && l.msg_id == st.msg_id && l.sender == me),
            Err(e) => return Outcome::Retry(format!("chain: {e}")),
        };
        let Some(sent) = sent else {
            return self.ignore_stage(env, &format!("{}/{} was not sent by this agent", st.conv, st.msg_id));
        };
        let rank = |e: &str| wire::STAGE_ORDER.iter().position(|s| *s == e);
        let new_rank = rank(&st.stage);
        let (from, id) = (st.from.clone(), st.msg_id.clone());
        let ts = st.ts_ms.max(0) as u64;
        let event = st.stage.clone();
        // Forward-only: a stage at or behind one already recorded from this peer
        // is a repeat (or arrived out of order) and changes nothing. A notice
        // (HELD, ADMITTED, DENIED) has no rank: it is recorded once.
        match self.store.append_event_unless(&st.conv, &st.stage, &id, &from, &sent.digest, LineExtra { peer_ts_ms: Some(ts), ..Default::default() }, |ls| {
            let mine = ls.iter().filter(|l| l.msg_id == id && l.sender == from);
            match new_rank {
                Some(n) => mine.filter_map(|l| rank(&l.event)).any(|r| r >= n),
                None => mine.into_iter().any(|l| l.event == event),
            }
        }) {
            Ok(_) => Outcome::Done,
            Err(e) => Outcome::Retry(format!("stage not recorded: {e}")),
        }
    }

    fn ignore_stage(&self, env: &Envelope, why: &str) -> Outcome {
        self.alert(
            "stage-ignored",
            &format!("inbox offset {} from {}: {why}", env.offset, env.sender_id),
        );
        Outcome::Done
    }
}

/// A stage record from this agent about `msg` (signed when posted), timed by
/// this agent's own chain line for that stage, so a re-sent stage carries the
/// first time, and carrying the conversation's chain head as the sender's anchor.
pub fn build_stage(store: &Store, me: &str, my_agent: &str, msg: &Msg, stage: &str) -> Result<Stage, String> {
    let head = store.chain_head(&msg.conv).map_err(|e| format!("chain head: {e}"))?;
    let ts_ms = store
        .chain_lines(&msg.conv)
        .ok()
        .and_then(|ls| {
            ls.into_iter()
                .find(|l| l.event == stage && l.msg_id == msg.msg_id && l.sender == msg.from)
                .and_then(|l| l.ts_ms)
        })
        .map(|t| t as i64)
        .unwrap_or_else(|| crate::store::now_ms() as i64);
    Ok(Stage {
        from: me.to_owned(),
        from_agent: my_agent.to_owned(),
        to_agent: msg.from_agent.clone(),
        conv: msg.conv.clone(),
        msg_id: msg.msg_id.clone(),
        stage: stage.to_owned(),
        ts_ms,
        chain_head: head,
    })
}

/// A sender-chosen name as it may appear in an operator notice; anything else is replaced.
pub fn shown(s: &str) -> String {
    if name_ok(s) { s.to_owned() } else { "(not shown)".to_owned() }
}
