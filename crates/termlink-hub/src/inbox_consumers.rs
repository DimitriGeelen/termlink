//! T-3700 (arc-011 P-12, B-2, 06a AL-30): who consumes each inbox, `proven` or `claimed`, and the bind.
//!
//! Every read (`channel.subscribe`), claim (`channel.claim`) and ack (`channel.post` of a `receipt`) on an
//! `inbox:*` topic records its reader. A reader names itself in an optional `reader` object; one with a key
//! card signs it, and is `proven` when the signature, the card and the revocation list all agree
//! (operator ruling E, 2026-11-19). A broken proof is refused, never downgraded. A bound inbox
//! (`hub.inbox_bind`, operator runme) admits only reads proven by the bound agent's bound key (FCW-124).
//! Design: `docs/reports/T-3700-design.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use termlink_protocol::control::error_code;

/// AL-30 2.14.c: entries kept per inbox; the oldest by `last_ms` is evicted with an admin row.
pub const MAX_PER_TOPIC: usize = 16;
/// Bound on the number of inboxes tracked, so caller-chosen topic names cannot grow the table without limit.
pub const MAX_TOPICS: usize = 4096;
/// A read proof's timestamp must be this close to the hub clock to prove (P-23 skew bound).
pub const PROOF_WINDOW_MS: i64 = 120_000;
/// `last_ms` is persisted when it moved by at least this much since the last write (design 3.2.2).
pub const PERSIST_EVERY_MS: u64 = 60_000;
/// V-11: the default consumer window.
pub const DEFAULT_WINDOW_S: u64 = 86_400;
/// The instance recorded for an ack (a receipt post carries no instance).
pub const ACK_INSTANCE: &str = "-";
/// The agent recorded for a caller that names no reader.
pub const UNNAMED: &str = "(unnamed)";
/// Domain prefix of the read-proof bytes; distinct from every post signature.
pub const READ_DOMAIN: &str = "termlink-inbox-read/v1";

/// Topics this module applies to.
pub fn applies(topic: &str) -> bool {
    topic.starts_with("inbox:")
}

/// The bytes a reader signs (design 2.1.2).
pub fn read_sign_bytes(hub_id: &str, topic: &str, method: &str, position: u64, ts_ms: i64, agent: &str, instance: &str) -> Vec<u8> {
    format!("{READ_DOMAIN}\n{hub_id}\n{topic}\n{method}\n{position}\n{ts_ms}\n{agent}\n{instance}").into_bytes()
}

/// The `reader` object of a read or claim.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadProof {
    pub agent: String,
    pub instance: String,
    pub ts_ms: i64,
    pub signer_pk: Option<String>,
    pub sig: Option<String>,
}

/// Parse `params.reader`. `Ok(None)` when absent; `Err` when present but malformed (refused: a caller that
/// sends a reader object must send a valid one).
pub fn parse_reader(params: &serde_json::Value) -> Result<Option<ReadProof>, String> {
    let Some(r) = params.get("reader") else { return Ok(None) };
    if r.is_null() {
        return Ok(None);
    }
    let s = |k: &str| r.get(k).and_then(|v| v.as_str()).map(str::to_string);
    let agent = s("agent").ok_or("reader.agent is missing")?;
    if !crate::key_card::valid_name(&agent) {
        return Err(format!("reader.agent {:?} is not a plain agent name", agent.chars().take(64).collect::<String>()));
    }
    let instance = s("instance").unwrap_or_default();
    if instance.len() > 128 || instance.chars().any(|c| c.is_control() || c == '\n') {
        return Err("reader.instance is longer than 128 bytes or holds a control character".into());
    }
    let ts_ms = r.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
    let signer_pk = s("signer_pk");
    let sig = s("sig");
    if signer_pk.is_some() != sig.is_some() {
        return Err("reader.signer_pk and reader.sig come together or not at all".into());
    }
    Ok(Some(ReadProof { agent, instance, ts_ms, signer_pk, sig }))
}

/// How strongly the hub knows who a reader is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Strength {
    Proven,
    Claimed,
}

/// A reader as the hub records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reader {
    pub agent: String,
    pub instance: String,
    pub strength: Strength,
    /// Why a reader is only `claimed` (empty when proven).
    pub why: String,
    /// The signing key fingerprint of a proven reader.
    pub key_fp: Option<String>,
}

/// A refusal: code, FCW id, reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: i64,
    pub fcw: &'static str,
    pub reason: String,
}

/// What [`assess`] needs from the hub, injectable for tests.
pub struct Lookups<'a> {
    /// The hub's id; `None` when the identity is not loaded (nothing can be proven).
    pub hub_id: Option<&'a str>,
    /// The signing key (raw 32 bytes) on an agent's card, `Ok(None)` without a card.
    pub card_key: &'a dyn Fn(&str) -> Result<Option<[u8; 32]>, String>,
    /// The fingerprints on the hub's revocation list.
    pub revoked: &'a dyn Fn() -> Result<BTreeSet<String>, String>,
}

fn claimed(agent: &str, instance: &str, why: &str) -> Reader {
    Reader { agent: agent.into(), instance: instance.into(), strength: Strength::Claimed, why: why.into(), key_fp: None }
}

/// Decide who a read or claim comes from (design 3.1). `transport` names a caller with no reader
/// (`local` or `tcp:<ip>`).
pub fn assess(
    topic: &str,
    method: &str,
    position: u64,
    proof: Option<&ReadProof>,
    transport: &str,
    now_ms: i64,
    l: &Lookups<'_>,
) -> Result<Reader, Refusal> {
    let Some(p) = proof else { return Ok(claimed(UNNAMED, transport, "no reader named")) };
    let (Some(pk_hex), Some(sig_hex)) = (p.signer_pk.as_deref(), p.sig.as_deref()) else {
        return Ok(claimed(&p.agent, &p.instance, "unsigned"));
    };
    let bad = |reason: String| Refusal { code: error_code::CHANNEL_SIGNATURE_INVALID, fcw: "FCW-124", reason };
    let Some(vk) = termlink_session::signed_state::verifying_key_of_hex(pk_hex) else {
        return Err(bad("reader.signer_pk is not a lower-case 32-byte ed25519 public key".into()));
    };
    let Some(sig) = parse_sig(sig_hex) else {
        return Err(bad("reader.sig is not a 64-byte ed25519 signature".into()));
    };
    let Some(hub_id) = l.hub_id else {
        return Ok(claimed(&p.agent, &p.instance, "the hub identity is not loaded"));
    };
    let bytes = read_sign_bytes(hub_id, topic, method, position, p.ts_ms, &p.agent, &p.instance);
    if vk.verify(&bytes, &sig).is_err() {
        return Err(bad(format!("the read signature of {:?} failed verification", p.agent)));
    }
    match (l.card_key)(&p.agent) {
        Ok(Some(k)) if &k == vk.as_bytes() => {}
        Ok(Some(_)) => {
            return Err(Refusal {
                code: error_code::CHANNEL_KEY_CARD_MISMATCH,
                fcw: "FCW-124",
                reason: format!("the read is not signed by the key on agent {:?}'s card", p.agent),
            })
        }
        Ok(None) => return Ok(claimed(&p.agent, &p.instance, "no key card")),
        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("key cards unreadable: {e}"))),
    }
    let fp = termlink_session::signed_state::key_fingerprint(&vk);
    match (l.revoked)() {
        Ok(keys) if keys.contains(&fp) => {
            return Err(Refusal {
                code: error_code::KEY_REVOKED,
                fcw: "FCW-99",
                reason: format!("agent {:?}'s card key {fp} is on the hub's revocation list", p.agent),
            })
        }
        Ok(_) => {}
        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("revocation list unreadable: {e}"))),
    }
    if (now_ms - p.ts_ms).abs() > PROOF_WINDOW_MS {
        return Ok(claimed(&p.agent, &p.instance, "clock skew"));
    }
    Ok(Reader { agent: p.agent.clone(), instance: p.instance.clone(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) })
}

/// The reader of a verified receipt post: proven when the signer's key is on a card and not revoked.
pub fn assess_ack(signer: &VerifyingKey, cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>, l: &Lookups<'_>) -> Reader {
    let fp = termlink_session::signed_state::key_fingerprint(signer);
    let agent = match cards() {
        Ok(cs) => cs.into_iter().find(|(_, k)| k == signer.as_bytes()).map(|(a, _)| a),
        Err(e) => return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, &format!("key cards unreadable: {e}")),
    };
    let Some(agent) = agent else {
        return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, "no key card");
    };
    match (l.revoked)() {
        Ok(keys) if !keys.contains(&fp) => {
            Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) }
        }
        Ok(_) => claimed(&agent, ACK_INSTANCE, "key revoked"),
        Err(e) => claimed(&agent, ACK_INSTANCE, &format!("revocation list unreadable: {e}")),
    }
}

fn parse_sig(hex: &str) -> Option<Signature> {
    let b = termlink_session::signed_state::unhex_lower(hex)?;
    let arr: [u8; 64] = b.try_into().ok()?;
    Some(Signature::from_bytes(&arr))
}

/// A consumer action (FCW-209: every one records).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Claim,
    Ack,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Claim => "claim",
            Action::Ack => "ack",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub agent: String,
    pub instance: String,
    pub strength: Strength,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_fp: Option<String>,
    pub first_ms: u64,
    pub last_ms: u64,
    pub actions: BTreeSet<String>,
    #[serde(skip)]
    persisted_last_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Binding {
    pub agent: String,
    pub key_fp: String,
    pub at_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConsumersFile {
    schema: String,
    topics: BTreeMap<String, Vec<Entry>>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct BindingsFile {
    schema: String,
    bindings: BTreeMap<String, Binding>,
}

const CONSUMERS_SCHEMA: &str = "termlink.inbox-consumers/v1";
const BINDINGS_SCHEMA: &str = "termlink.inbox-bindings/v1";

pub fn consumers_path(dir: &Path) -> PathBuf {
    dir.join("inbox-consumers.json")
}

pub fn bindings_path(dir: &Path) -> PathBuf {
    dir.join("inbox-bindings.json")
}

/// In-memory state of one runtime dir.
#[derive(Debug, Default)]
struct State {
    topics: BTreeMap<String, Vec<Entry>>,
    /// The last persist of the consumer table failed: `inbox.consumers` answers an error (FCW-125).
    persist_error: Option<String>,
    /// The consumer file could not be read at load: same.
    load_error: Option<String>,
}

static STATES: Mutex<Option<HashMap<PathBuf, State>>> = Mutex::new(None);

fn with_state<R>(dir: &Path, f: impl FnOnce(&mut State) -> R) -> R {
    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
    f(st)
}

fn load_state(dir: &Path) -> State {
    match std::fs::read(consumers_path(dir)) {
        Ok(b) => match serde_json::from_slice::<ConsumersFile>(&b) {
            Ok(f) if f.schema == CONSUMERS_SCHEMA => {
                let mut topics = f.topics;
                for v in topics.values_mut() {
                    for e in v.iter_mut() {
                        e.persisted_last_ms = e.last_ms;
                    }
                }
                State { topics, ..Default::default() }
            }
            Ok(f) => State { load_error: Some(format!("unknown schema {:?}", f.schema)), ..Default::default() },
            Err(e) => State { load_error: Some(format!("unparseable: {e}")), ..Default::default() },
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
        Err(e) => State { load_error: Some(e.to_string()), ..Default::default() },
    }
}

fn load_bindings(dir: &Path) -> Result<BTreeMap<String, Binding>, String> {
    match std::fs::read(bindings_path(dir)) {
        Ok(b) => {
            let f: BindingsFile = serde_json::from_slice(&b).map_err(|e| format!("inbox bindings unparseable: {e}"))?;
            if f.schema != BINDINGS_SCHEMA {
                return Err(format!("inbox bindings: unknown schema {:?}", f.schema));
            }
            Ok(f.bindings)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
        Err(e) => Err(format!("inbox bindings unreadable: {e}")),
    }
}

fn save_consumers(dir: &Path, topics: &BTreeMap<String, Vec<Entry>>) -> Result<(), String> {
    let f = ConsumersFile { schema: CONSUMERS_SCHEMA.into(), topics: topics.clone() };
    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
    crate::key_card::atomic_write(&consumers_path(dir), &text, 0o600).map_err(|e| e.to_string())
}

/// Admin-log writer, injectable for tests.
pub type AdminLog<'a> = &'a mut dyn FnMut(serde_json::Value) -> Result<(), String>;

/// Check the bind and record the reader in one locked step (2.14.f(4)). `Err` refuses the call.
pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
    // the bind is read under the same lock the bind verb writes under
    let bindings = load_bindings(dir).map_err(|e| Refusal {
        code: error_code::INBOX_BOUND,
        fcw: "FCW-124",
        reason: format!("{e}; inbox reads are refused until it is repaired"),
    })?;
    if let Some(b) = bindings.get(topic) {
        let ok = reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str());
        if !ok {
            let shown = if reader.strength == Strength::Proven {
                format!("{} (proven)", reader.agent)
            } else {
                format!("{} (claimed: {})", reader.agent, reader.why)
            };
            if refused_row_allowed(now_ms) {
                let _ = log(serde_json::json!({
                    "event": "inbox-bound-refused", "attention": true, "topic": topic, "bound": b.agent,
                    "reader": reader.agent, "instance": reader.instance, "strength": reader.strength,
                    "action": action.name(),
                }));
            }
            return Err(Refusal {
                code: error_code::INBOX_BOUND,
                fcw: "FCW-124",
                reason: format!("inbox bound to {}; {} by {shown} refused", b.agent, action.name()),
            });
        }
    }
    let mut dirty = false;
    if !st.topics.contains_key(topic) && st.topics.len() >= MAX_TOPICS {
        // evict the inbox whose newest reader is oldest
        if let Some(old) = st
            .topics
            .iter()
            .min_by_key(|(_, v)| v.iter().map(|e| e.last_ms).max().unwrap_or(0))
            .map(|(k, _)| k.clone())
        {
            st.topics.remove(&old);
            let _ = log(serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}));
            dirty = true;
        }
    }
    let list = st.topics.entry(topic.to_string()).or_default();
    match list.iter_mut().find(|e| e.agent == reader.agent && e.instance == reader.instance) {
        Some(e) => {
            if e.strength != reader.strength || e.key_fp != reader.key_fp || e.why != reader.why {
                e.strength = reader.strength.clone();
                e.key_fp = reader.key_fp.clone();
                e.why = reader.why.clone();
                dirty = true;
            }
            if e.actions.insert(action.name().to_string()) {
                dirty = true;
            }
            e.last_ms = e.last_ms.max(now_ms);
            if e.last_ms >= e.persisted_last_ms + PERSIST_EVERY_MS {
                dirty = true;
            }
        }
        None => {
            if list.len() >= MAX_PER_TOPIC {
                if let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i) {
                    let gone = list.remove(i);
                    let _ = log(serde_json::json!({
                        "event": "inbox-consumer-evicted", "topic": topic, "agent": gone.agent,
                        "instance": gone.instance, "last_ms": gone.last_ms,
                    }));
                }
            }
            list.push(Entry {
                agent: reader.agent.clone(),
                instance: reader.instance.clone(),
                strength: reader.strength.clone(),
                why: reader.why.clone(),
                key_fp: reader.key_fp.clone(),
                first_ms: now_ms,
                last_ms: now_ms,
                actions: [action.name().to_string()].into_iter().collect(),
                persisted_last_ms: 0,
            });
            dirty = true;
        }
    }
    if dirty {
        match save_consumers(dir, &st.topics) {
            Ok(()) => {
                st.persist_error = None;
                for v in st.topics.values_mut() {
                    for e in v.iter_mut() {
                        e.persisted_last_ms = e.last_ms;
                    }
                }
            }
            // the read itself goes on; the list is now reported unreadable (FCW-125), never "one"
            Err(e) => st.persist_error = Some(format!("consumer list not persisted: {e}")),
        }
    }
    Ok(())
}

/// `inbox.consumers`: readers seen within `window_s`, plus the binding. `Err` when the list cannot be
/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one".
pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<serde_json::Value, String> {
    let bound = load_bindings(dir)?.get(topic).cloned();
    with_state(dir, |st| {
        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {
            return Err(e.clone());
        }
        let since = now_ms.saturating_sub(window_s.saturating_mul(1000));
        let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
        Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
    })
}

/// `hub.inbox_bind`: admin row first, then the durable binding (2.14.c). The bound key is the agent's
/// card key now; an agent without a card cannot be bound.
pub fn bind(dir: &Path, topic: &str, agent: &str, card_key: Option<[u8; 32]>, now_ms: u64, log: AdminLog<'_>) -> Result<Binding, String> {
    if !applies(topic) {
        return Err(format!("{topic:?} is not an inbox topic"));
    }
    if !crate::key_card::valid_name(agent) {
        return Err(format!("{agent:?} is not a plain agent name"));
    }
    let Some(k) = card_key else { return Err(format!("agent {agent:?} has no key card; bind refused")) };
    let vk = VerifyingKey::from_bytes(&k).map_err(|e| format!("agent {agent:?}'s card key is invalid: {e}"))?;
    let b = Binding { agent: agent.into(), key_fp: termlink_session::signed_state::key_fingerprint(&vk), at_ms: now_ms };
    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let mut all = load_bindings(dir)?;
    log(serde_json::json!({"event": "inbox-bind", "topic": topic, "agent": agent, "key_fp": b.key_fp}))?;
    all.insert(topic.to_string(), b.clone());
    write_bindings(dir, all)?;
    Ok(b)
}

/// `hub.inbox_unbind`: admin row first, then remove the binding. Unbinding an unbound inbox is a no-op.
pub fn unbind(dir: &Path, topic: &str, log: AdminLog<'_>) -> Result<bool, String> {
    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let mut all = load_bindings(dir)?;
    if !all.contains_key(topic) {
        return Ok(false);
    }
    log(serde_json::json!({"event": "inbox-unbind", "topic": topic}))?;
    all.remove(topic);
    write_bindings(dir, all)?;
    Ok(true)
}

fn write_bindings(dir: &Path, bindings: BTreeMap<String, Binding>) -> Result<(), String> {
    let f = BindingsFile { schema: BINDINGS_SCHEMA.into(), bindings };
    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
    crate::key_card::atomic_write(&bindings_path(dir), &text, 0o600).map_err(|e| e.to_string())
}

/// At most 10 refused-read admin rows a minute (any caller can provoke them; r2 CC 12 of T-3703).
fn refused_row_allowed(now_ms: u64) -> bool {
    static WIN: Mutex<(u64, u32)> = Mutex::new((0, 0));
    let mut w = WIN.lock().unwrap_or_else(|p| p.into_inner());
    let minute = now_ms / 60_000;
    if w.0 != minute {
        *w = (minute, 0);
    }
    w.1 += 1;
    w.1 <= 10
}

/// Forget the cached state of `dir` (tests; a hub restart starts from disk anyway).
#[cfg(test)]
pub(crate) fn reset(dir: &Path) {
    if let Some(m) = STATES.lock().unwrap_or_else(|p| p.into_inner()).as_mut() {
        m.remove(dir);
    }
}

#[cfg(test)]
#[path = "inbox_consumers_tests.rs"]
mod tests;
