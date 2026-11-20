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
pub const MAX_TOPICS: usize = if cfg!(test) { 64 } else { 4096 };
/// A read proof's timestamp must be this close to the hub clock to prove (P-23 skew bound).
pub const PROOF_WINDOW_MS: i64 = 120_000;
/// `last_ms` is persisted when it moved by at least this much since the last write (design 3.2.2).
pub const PERSIST_EVERY_MS: u64 = 60_000;
/// Longest inbox topic recorded or bound (r1 CC 3 / OC 2).
pub const MAX_TOPIC_LEN: usize = 256;
/// Admin rows of one kind written per minute; the rest are counted and reported on the next row (r1 CC 13).
pub const ROWS_PER_MINUTE: u32 = 10;
/// V-11: the default consumer window.
pub const DEFAULT_WINDOW_S: u64 = 86_400;
/// The instance recorded for an ack (a receipt post carries no instance).
pub const ACK_INSTANCE: &str = "-";
/// The agent recorded for a caller that names no reader.
pub const UNNAMED: &str = "(unnamed)";
/// Domain prefix of the read-proof bytes (shared with the sidecar through `termlink-session`).
pub use termlink_session::inbox_read::{READ_DOMAIN, read_sign_bytes};

/// Topics this module applies to.
pub fn applies(topic: &str) -> bool {
    topic.starts_with("inbox:")
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

/// The params field carrying a read proof. NOT `reader`: that name already means a plain reader NAME on
/// `channel.subscribe` (T-3309 read activity) and old clients send it as a string (regression found live
/// 2026-11-20, fixed T-3700).
pub const PROOF_FIELD: &str = "read_proof";

/// Parse `params.read_proof`. `Ok(None)` when absent; `Err` when present but malformed (refused: a caller that
/// sends a proof object must send a valid one).
pub fn parse_reader(params: &serde_json::Value) -> Result<Option<ReadProof>, String> {
    let Some(r) = params.get(PROOF_FIELD) else { return Ok(None) };
    if r.is_null() {
        return Ok(None);
    }
    if !r.is_object() {
        return Err(format!("{PROOF_FIELD} must be an object"));
    }
    let s = |k: &str| r.get(k).and_then(|v| v.as_str()).map(str::to_string);
    let agent = s("agent").ok_or("reader.agent is missing")?;
    if !crate::key_card::valid_name(&agent) {
        return Err(format!("reader.agent {:?} is not a plain agent name", agent.chars().take(64).collect::<String>()));
    }
    let instance = s("instance").unwrap_or_default();
    // r1 CC 1 / OC 1: `-` is the ack sentinel and an empty instance names nobody; neither may be claimed
    if instance.is_empty() || instance == ACK_INSTANCE || instance.len() > 128 || instance.chars().any(|c| c.is_control()) {
        return Err("reader.instance is missing, is \"-\", is longer than 128 bytes or holds a control character".into());
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
    if now_ms.abs_diff(p.ts_ms) > PROOF_WINDOW_MS as u64 {
        return Ok(claimed(&p.agent, &p.instance, "clock skew"));
    }
    Ok(Reader { agent: p.agent.clone(), instance: p.instance.clone(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) })
}

/// The reader of a verified receipt post: proven when the signer's key is on a card, not revoked, and the post's
/// `ts` is within the proof window (a replayed old receipt is only claimed, r1 CC 7). An uncarded ack is listed
/// under the payload's claimed `from_agent`, else `(unnamed)` (r1 OC 6).
pub fn assess_ack(
    signer: &VerifyingKey,
    claimed_agent: Option<&str>,
    post_ts_ms: i64,
    now_ms: i64,
    cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>,
    l: &Lookups<'_>,
) -> Reader {
    let fp = termlink_session::signed_state::key_fingerprint(signer);
    let short = fp.chars().skip(8).take(16).collect::<String>();
    let named = claimed_agent.filter(|a| crate::key_card::valid_name(a)).unwrap_or(UNNAMED).to_string();
    let agent = match cards() {
        Ok(cs) => cs.into_iter().find(|(_, k)| k == signer.as_bytes()).map(|(a, _)| a),
        Err(e) => return claimed(&named, ACK_INSTANCE, &format!("key cards unreadable: {e} (key {short})")),
    };
    let Some(agent) = agent else {
        return claimed(&named, ACK_INSTANCE, &format!("no key card (key {short})"));
    };
    match (l.revoked)() {
        Ok(keys) if keys.contains(&fp) => claimed(&agent, ACK_INSTANCE, "key revoked"),
        Ok(_) if now_ms.abs_diff(post_ts_ms) > PROOF_WINDOW_MS as u64 => claimed(&agent, ACK_INSTANCE, "stale ack"),
        Ok(_) => Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) },
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
    /// Set when an unreadable table was moved aside: readers before this time are lost (r1 CC 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    incomplete_since_ms: Option<u64>,
    /// Last time a new inbox went unrecorded because the table was full (r2 CC N1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dropped_since_ms: Option<u64>,
    /// First drop of the current run of drops, kept while it is inside the 24 h window (r3 CC 1): an inbox that
    /// entered the table before it was there through every drop since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dropped_first_ms: Option<u64>,
    /// When each inbox entered the table (r3 CC 1): one entering after a drop may be the dropped one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    topic_since: BTreeMap<String, u64>,
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
    incomplete_since_ms: Option<u64>,
    dropped_since_ms: Option<u64>,
    dropped_first_ms: Option<u64>,
    topic_since: BTreeMap<String, u64>,
    /// The last persist of the consumer table failed: `inbox.consumers` answers an error (FCW-125).
    persist_error: Option<String>,
    /// The consumer file could not be read nor moved aside: nothing is saved over it, and
    /// `inbox.consumers` answers an error.
    load_error: Option<String>,
    /// Admin rows found at load (a corrupt table moved aside), written by the next [`record`].
    pending_rows: Vec<serde_json::Value>,
}

static STATES: Mutex<Option<HashMap<PathBuf, State>>> = Mutex::new(None);

fn with_state<R>(dir: &Path, f: impl FnOnce(&mut State) -> R) -> R {
    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
    f(st)
}

fn now_wall_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Load the table. An unreadable or unknown-schema file is never saved over: it is moved to
/// `<name>.corrupt-<ms>` (admin row on the next record) and the table restarts marked incomplete (r1 CC 4).
fn load_state(dir: &Path) -> State {
    let path = consumers_path(dir);
    let why = match std::fs::read(&path) {
        Ok(b) => match serde_json::from_slice::<ConsumersFile>(&b) {
            Ok(f) if f.schema == CONSUMERS_SCHEMA => {
                let mut topics = f.topics;
                for v in topics.values_mut() {
                    for e in v.iter_mut() {
                        e.persisted_last_ms = e.last_ms;
                    }
                }
                return State {
                    topics,
                    incomplete_since_ms: f.incomplete_since_ms,
                    dropped_since_ms: f.dropped_since_ms,
                    dropped_first_ms: f.dropped_first_ms,
                    topic_since: f.topic_since,
                    ..Default::default()
                };
            }
            Ok(f) => format!("unknown schema {:?}", f.schema),
            Err(e) => format!("unparseable: {e}"),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return State::default(),
        Err(e) => format!("unreadable: {e}"),
    };
    let now = now_wall_ms();
    let aside = path.with_file_name(format!("inbox-consumers.json.corrupt-{now}"));
    crate::key_card::guard_test_write(dir);
    match std::fs::rename(&path, &aside) {
        Ok(()) => {
            let mut st = State {
                incomplete_since_ms: Some(now),
                pending_rows: vec![serde_json::json!({
                    "event": "inbox-consumers-reset", "attention": true, "why": why, "moved_to": aside.display().to_string(),
                })],
                ..Default::default()
            };
            // r2 CC N4: the marker reaches disk at once, so a restart before any record keeps it
            if let Err(e) = save_consumers(dir, &st) {
                st.persist_error = Some(format!("consumer list not persisted: {e}"));
            }
            st
        }
        Err(e) => State { load_error: Some(format!("consumer table {why}; not moved aside: {e}")), ..Default::default() },
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

fn save_consumers(dir: &Path, st: &State) -> Result<(), String> {
    let f = ConsumersFile {
        schema: CONSUMERS_SCHEMA.into(),
        topics: st.topics.clone(),
        incomplete_since_ms: st.incomplete_since_ms,
        dropped_since_ms: st.dropped_since_ms,
        dropped_first_ms: st.dropped_first_ms,
        topic_since: st.topic_since.clone(),
    };
    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
    crate::key_card::atomic_write(&consumers_path(dir), &text, 0o600).map_err(|e| e.to_string())
}

/// Admin-log writer, injectable for tests.
pub type AdminLog<'a> = &'a mut dyn FnMut(serde_json::Value) -> Result<(), String>;

/// Write a capped admin row of one kind: at most [`ROWS_PER_MINUTE`] a minute; the rest are counted and the
/// count rides on the next written row as `suppressed` (r1 CC 3, CC 13).
fn capped_row(kind: &'static str, mut row: serde_json::Value, now_ms: u64, log: AdminLog<'_>) {
    static WIN: Mutex<Option<HashMap<&'static str, (u64, u32, u64)>>> = Mutex::new(None);
    let mut g = WIN.lock().unwrap_or_else(|p| p.into_inner());
    let w = g.get_or_insert_with(HashMap::new).entry(kind).or_insert((0, 0, 0));
    let minute = now_ms / 60_000;
    if w.0 != minute {
        w.0 = minute;
        w.1 = 0;
    }
    w.1 += 1;
    if w.1 > ROWS_PER_MINUTE {
        w.2 += 1;
        return;
    }
    if w.2 > 0 {
        row["suppressed"] = serde_json::json!(w.2);
    }
    // r2 CC N5: the suppressed count is cleared only once a row carrying it is written
    if log(row).is_ok() {
        w.2 = 0;
    }
}

/// Whether `reader` passes the binding of `topic` (proven, the bound agent, the bound key).
fn passes(b: &Binding, reader: &Reader) -> bool {
    reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str())
}

fn bound_refusal(topic: &str, b: &Binding, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Refusal {
    let shown = if reader.strength == Strength::Proven {
        format!("{} (proven)", reader.agent)
    } else {
        format!("{} (claimed: {})", reader.agent, reader.why)
    };
    capped_row(
        "inbox-bound-refused",
        serde_json::json!({
            "event": "inbox-bound-refused", "attention": true, "topic": topic, "bound": b.agent,
            "reader": reader.agent, "instance": reader.instance, "strength": reader.strength, "action": action.name(),
        }),
        now_ms,
        log,
    );
    Refusal { code: error_code::INBOX_BOUND, fcw: "FCW-124", reason: format!("inbox bound to {}; {} by {shown} refused", b.agent, action.name()) }
}

fn bindings_refusal(e: String) -> Refusal {
    Refusal { code: error_code::INBOX_BOUND, fcw: "FCW-124", reason: format!("{e}; inbox reads are refused until it is repaired") }
}

/// Whether `topic` has a binding; unreadable bindings count as bound (fail closed, FCW-124).
pub fn is_bound(dir: &Path, topic: &str) -> bool {
    load_bindings(dir).map(|b| b.contains_key(topic)).unwrap_or(true)
}

/// Re-check only the binding (no recording): a long-poll that waited across a bind must not deliver (r1 CC 6).
pub fn check_bound(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let bindings = load_bindings(dir).map_err(bindings_refusal)?;
    match bindings.get(topic) {
        Some(b) if !passes(b, reader) => Err(bound_refusal(topic, b, reader, action, now_ms, log)),
        _ => Ok(()),
    }
}

/// Check the bind and record the reader in one locked step (2.14.f(4)). `Err` refuses the call.
/// The caller records only topics that exist on the bus (r1 CC 3 / OC 2).
pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
    // r2 CC N4: a load-time row stays queued until it is written
    st.pending_rows.retain(|row| log(row.clone()).is_err());
    // the bind is read under the same lock the bind verb writes under
    let bindings = load_bindings(dir).map_err(bindings_refusal)?;
    if let Some(b) = bindings.get(topic)
        && !passes(b, reader)
    {
        return Err(bound_refusal(topic, b, reader, action, now_ms, log));
    }
    if topic.len() > MAX_TOPIC_LEN || st.load_error.is_some() {
        // nothing is recorded over an unreadable table (it stays reported unreadable)
        return Ok(());
    }
    let mut dirty = false;
    if !st.topics.contains_key(topic) && st.topics.len() >= MAX_TOPICS {
        // evict the inbox whose newest reader is oldest, but never one with a reader inside the window
        let window_start = now_ms.saturating_sub(DEFAULT_WINDOW_S * 1000);
        let oldest = st
            .topics
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(|e| e.last_ms).max().unwrap_or(0)))
            .min_by_key(|(_, last)| *last);
        match oldest {
            Some((old, last)) if last < window_start => {
                st.topics.remove(&old);
                st.topic_since.remove(&old);
                capped_row("inbox-consumer-topic-evicted", serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}), now_ms, log);
                dirty = true;
            }
            _ => {
                capped_row(
                    "inbox-consumer-table-full",
                    serde_json::json!({"event": "inbox-consumer-table-full", "attention": true, "topic": topic}),
                    now_ms,
                    log,
                );
                // r2 CC N1: an inbox missing from a full table is answered as an error, never a clean list.
                // r3 CC 2 / OC 1: the marker moves on every drop; only the save is rate-limited.
                let saved = st.dropped_since_ms;
                if st.dropped_since_ms.is_none_or(|t| t < window_start) {
                    st.dropped_first_ms = Some(now_ms); // a new run of drops
                }
                st.dropped_since_ms = Some(now_ms);
                if saved.is_none_or(|t| now_ms >= t + PERSIST_EVERY_MS)
                    && let Err(e) = save_consumers(dir, st)
                {
                    st.persist_error = Some(format!("consumer list not persisted: {e}"));
                }
                return Ok(());
            }
        }
    }
    if !st.topics.contains_key(topic) {
        st.topic_since.insert(topic.to_string(), now_ms);
    }
    let list = st.topics.entry(topic.to_string()).or_default();
    // r1 CC 1 / OC 4: keyed by strength and key too, so a claimed reader never merges into a proven entry
    match list
        .iter_mut()
        .find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)
    {
        Some(e) => {
            // r2 CC N6: a changed reason alone is persisted on the PERSIST_EVERY_MS cadence, not per call
            e.why.clone_from(&reader.why);
            if e.actions.insert(action.name().to_string()) {
                dirty = true;
            }
            e.last_ms = e.last_ms.max(now_ms);
            if e.last_ms >= e.persisted_last_ms + PERSIST_EVERY_MS {
                dirty = true;
            }
        }
        None => {
            if list.len() >= MAX_PER_TOPIC
                && let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i)
            {
                let gone = list.remove(i);
                capped_row(
                    "inbox-consumer-evicted",
                    serde_json::json!({
                        "event": "inbox-consumer-evicted", "topic": topic, "agent": gone.agent,
                        "instance": gone.instance, "last_ms": gone.last_ms,
                    }),
                    now_ms,
                    log,
                );
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
        match save_consumers(dir, st) {
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
/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one". A table restarted after a corrupt
/// one stays an error until the window no longer reaches back before the restart (r1 CC 4).
pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<serde_json::Value, String> {
    let bound = load_bindings(dir)?.get(topic).cloned();
    with_state(dir, |st| {
        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {
            return Err(e.clone());
        }
        let since = now_ms.saturating_sub(window_s.saturating_mul(1000));
        if let Some(t) = st.incomplete_since_ms
            && t > since
        {
            return Err(format!("consumer history incomplete since {t} (an unreadable table was moved aside)"));
        }
        // r3 CC 1: an inbox absent from the table, or one that entered it after a drop inside the window (it may be
        // the one whose reader was dropped), is answered as an error, never as a clean list
        let first = st.dropped_first_ms.unwrap_or(0);
        let entered_after_first_drop = st.topic_since.get(topic).is_some_and(|entered| *entered >= first);
        if let Some(t) = st.dropped_since_ms
            && t > since
            && (!st.topics.contains_key(topic) || entered_after_first_drop)
        {
            return Err(format!("consumer table full since {t}: this inbox's readers are not recorded"));
        }
        let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
        Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
    })
}

/// `hub.inbox_bind`: admin row first, then the durable binding (2.14.c). The bound key is the agent's
/// card key now; an agent without a card, or whose card key is revoked (r1 CC 14), cannot be bound.
pub fn bind(
    dir: &Path,
    topic: &str,
    agent: &str,
    card_key: Option<[u8; 32]>,
    revoked: &dyn Fn() -> Result<BTreeSet<String>, String>,
    now_ms: u64,
    log: AdminLog<'_>,
) -> Result<Binding, String> {
    if !applies(topic) || topic.len() > MAX_TOPIC_LEN {
        return Err(format!("{topic:?} is not an inbox topic of at most {MAX_TOPIC_LEN} bytes"));
    }
    if !crate::key_card::valid_name(agent) {
        return Err(format!("{agent:?} is not a plain agent name"));
    }
    let Some(k) = card_key else { return Err(format!("agent {agent:?} has no key card; bind refused")) };
    let vk = VerifyingKey::from_bytes(&k).map_err(|e| format!("agent {agent:?}'s card key is invalid: {e}"))?;
    let b = Binding { agent: agent.into(), key_fp: termlink_session::signed_state::key_fingerprint(&vk), at_ms: now_ms };
    if revoked().map_err(|e| format!("revocation list unreadable: {e}"))?.contains(&b.key_fp) {
        return Err(format!("agent {agent:?}'s card key {} is revoked; bind refused", b.key_fp));
    }
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
