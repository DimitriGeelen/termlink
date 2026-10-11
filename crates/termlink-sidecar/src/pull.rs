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
use std::sync::atomic::{AtomicBool, Ordering};
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
/// The store state file holding the next inbox offset to read.
pub const CURSOR_STATE: &str = "inbox-cursor";
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
}

struct Inner {
    cfg: ReceiveConfig,
    store: Arc<Store>,
    sender: Arc<Sender>,
    policy: Box<dyn Policy>,
    rt: tokio::runtime::Runtime,
    pass: Mutex<PassMemory>,
    stop: AtomicBool,
}

#[derive(Default)]
struct PassMemory {
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
        let inner = Arc::new(Inner {
            cfg,
            store,
            sender,
            policy,
            rt,
            pass: Mutex::new(PassMemory::default()),
            stop: AtomicBool::new(false),
        });
        let thread = interval.map(|iv| {
            let i = inner.clone();
            std::thread::spawn(move || {
                let mut waited = Duration::ZERO;
                while !i.stop.load(Ordering::SeqCst) {
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

    /// The next inbox offset this receiver will read.
    pub fn cursor(&self) -> u64 {
        self.inner.cursor()
    }

    pub fn topic(&self) -> String {
        inbox_topic(&self.inner.cfg.declared_hub_id, &self.inner.cfg.agent)
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

    fn cursor(&self) -> u64 {
        self.store
            .read_state(CURSOR_STATE)
            .ok()
            .flatten()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0)
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
        match self.check_pin() {
            Ok(()) => mem.held = None,
            Err(PinError::Unreachable(_)) => return PassResult::Waiting,
            Err(PinError::Mismatch { declared, found }) => {
                let why = format!(
                    "hub id mismatch: declared {declared}, hub says {}; inbox not read",
                    found.as_deref().unwrap_or("nothing")
                );
                if mem.held.as_deref() != Some(why.as_str()) {
                    let _ = self.store.attention("hub-id-mismatch", &why);
                }
                mem.held = Some(why.clone());
                return PassResult::Held(why);
            }
        }
        let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
        let mut handled = 0;
        loop {
            let cursor = self.cursor();
            let resp = match self.rpc(
                method::CHANNEL_SUBSCRIBE,
                json!({"topic": topic, "cursor": cursor, "limit": self.cfg.page}),
            ) {
                Ok(r) => r,
                Err(_) => return PassResult::Waiting,
            };
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
                    let _ = self.store.attention("inbox-unreadable", "envelope without an offset");
                    return PassResult::Stalled { handled, reason: "envelope without an offset".into() };
                };
                if env.offset < cursor {
                    continue;
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
                            let _ = self.store.attention("receive-stalled", &format!("inbox offset {key}"));
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
                let _ = self.store.attention(
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
            let why = format!("priority {} from a {} sender: urgent needs the allow-list", a.priority, a.trust.as_str());
            if let Err(e) = once(policy::DOWNGRADED, Some(&why)) {
                return Outcome::Retry(format!("DOWNGRADED not recorded: {e}"));
            }
        }
        // A sender outside the allow-list is never answered: its name for itself
        // (`from_agent`) is not bound to a key we know.
        let answer = a.withheld.is_none();
        if answer && let Err(e) = self.report(&msg, RECEIVED) {
            return Outcome::Retry(e);
        }
        match self.store.put(from, conv, id, &env.payload) {
            Ok(PutOutcome::Stored { .. }) | Ok(PutOutcome::Duplicate { .. }) => {
                if let Some(code) = a.withheld {
                    // Stored and shown, never delivered (7.2).
                    match once(REFUSED, Some(code)) {
                        Ok(Some(_)) => {
                            let _ = self.store.attention(
                                code,
                                &format!(
                                    "{conv}/{id} from {from} ({}): stored, not delivered: sender is not on the allow-list",
                                    a.trust.as_str()
                                ),
                            );
                        }
                        Ok(None) => {}
                        Err(e) => return Outcome::Retry(format!("REFUSED not recorded: {e}")),
                    }
                    return Outcome::Done;
                }
                match self.report(&msg, STORED) {
                    Ok(()) => Outcome::Done,
                    Err(e) => Outcome::Retry(e),
                }
            }
            Ok(PutOutcome::Conflict { first_seq, conflict_seq }) => {
                let _ = self.store.attention(
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

    /// Queue a signed stage record for the sender. The chain head is read after
    /// the stage line, so the sender holds an anchor for this agent's chain.
    fn report(&self, msg: &Msg, stage: &str) -> Result<(), String> {
        let head = self.store.chain_head(&msg.conv).map_err(|e| format!("chain head: {e}"))?;
        let st = Stage {
            from: self.me().to_owned(),
            from_agent: self.cfg.agent.clone(),
            to_agent: msg.from_agent.clone(),
            conv: msg.conv.clone(),
            msg_id: msg.msg_id.clone(),
            stage: stage.to_owned(),
            ts_ms: self.stage_time(msg, stage),
            chain_head: head,
        };
        self.sender
            .post_stage(&msg.from_agent, &st)
            .map_err(|e| format!("{stage} stage not queued: {e}"))
    }

    /// The time of this agent's own chain line for the stage, so a re-sent stage
    /// carries the same time as the first.
    fn stage_time(&self, msg: &Msg, stage: &str) -> i64 {
        self.store
            .chain_lines(&msg.conv)
            .ok()
            .and_then(|ls| {
                ls.into_iter()
                    .find(|l| l.event == stage && l.msg_id == msg.msg_id && l.sender == msg.from)
                    .and_then(|l| l.ts_ms)
            })
            .map(|t| t as i64)
            .unwrap_or_else(|| crate::store::now_ms() as i64)
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
            let _ = self.store.attention(
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
            Some(s) => wire::verify_stage(s, &self.cfg.roster),
        };
        let st = match verified {
            Ok(s) => s,
            Err(r) => return self.ignore_stage(env, &r.to_string()),
        };
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
        // Forward-only: a stage at or behind one already recorded from this peer
        // is a repeat (or arrived out of order) and changes nothing.
        match self.store.append_event_unless(&st.conv, &st.stage, &id, &from, &sent.digest, LineExtra { peer_ts_ms: Some(ts), ..Default::default() }, |ls| {
            ls.iter()
                .filter(|l| l.msg_id == id && l.sender == from)
                .filter_map(|l| rank(&l.event))
                .any(|r| Some(r) >= new_rank)
        }) {
            Ok(_) => Outcome::Done,
            Err(e) => Outcome::Retry(format!("stage not recorded: {e}")),
        }
    }

    fn ignore_stage(&self, env: &Envelope, why: &str) -> Outcome {
        let _ = self.store.attention(
            "stage-ignored",
            &format!("inbox offset {} from {}: {why}", env.offset, env.sender_id),
        );
        Outcome::Done
    }
}
