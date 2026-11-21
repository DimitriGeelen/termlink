//! Liveness of the pinned main (T-3646; T-3619 slice 2, operator ruling B 2026-11-01).
//!
//! The main's sidecar (the agent whose name is the address book's `main`) posts a small signed
//! note on the hub every [`NOTE_EVERY_MS`]: its agent, instance, the sample time and how far it
//! has drained its inbox. Every sidecar with a book reads the newest note from the hub, so any
//! host can tell whether the main is alive and draining, and `status` carries the verdict as
//! `main.liveness`.
//!
//! Verdicts, fail-closed (nothing that is not proven alive is ever reported alive):
//! * `alive`: a note from the pinned main, signed by a known key, sampled within
//!   [`STALE_AFTER_MS`], and no inbox mail it had left waiting longer than that.
//! * `stale`: the newest note is older than [`STALE_AFTER_MS`].
//! * `stuck`: notes are fresh, but at the note's sample time the main's inbox held a message at
//!   or past `drained_up_to` that had already been on the hub longer than [`STALE_AFTER_MS`].
//!   That is "N stopped advancing while mail waits": a draining main would have passed it.
//! * `unknown`: no book or no pinned main, no note yet, the hub could not be read, the note is
//!   unreadable, from another agent or instance, signed by an unknown key, dated in the future,
//!   or this sidecar's own check has not run recently.
//!
//! The needs-attention alarm is a latch persisted in the store (state [`ALARM_STATE`]): one entry
//! on entering `stale` or `stuck`, one on recovery to `alive`, nothing per cycle. `unknown` never
//! raises or clears it: it is shown in `status`, not alarmed (a main whose sidecar predates this
//! note would otherwise alarm forever).

use crate::book::Book;
use crate::send::inbox_topic;
use crate::store::Store;
use crate::wire::Roster;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termlink_protocol::TransportAddr;
use termlink_protocol::control::{error_code, method};
use termlink_protocol::jsonrpc::RpcResponse;

/// How often the main posts its note, and how often every sidecar reads it.
pub const NOTE_EVERY_MS: u64 = 60_000;
/// A note older than this is stale; a message waiting longer than this makes the main stuck.
/// Five intervals: a main that misses four notes in a row (hub blips, a slow pass) is still alive.
pub const STALE_AFTER_MS: u64 = 5 * NOTE_EVERY_MS;
/// The note's `msg_type` on the hub.
pub const NOTE_TYPE: &str = "sidecar.liveness.v1";
/// The hub keeps the newest notes only (100 notes, about 100 minutes at the default interval).
pub const NOTE_KEEP: u64 = 100;
/// Store state holding the alarm latch: `ok`, `stale` or `stuck`.
pub const ALARM_STATE: &str = "main-liveness-alarm";
/// Needs-attention kinds.
pub const ALERT_STALE: &str = "main-stale";
pub const ALERT_STUCK: &str = "main-stuck";
pub const ALERT_RECOVERED: &str = "main-recovered";

/// Test seams, honoured only in a test estate (never in production mode), the same rule as
/// `beat::test_overrides`: `TERMLINK_SIDECAR_TEST_LIVENESS_EVERY_MS` and
/// `TERMLINK_SIDECAR_TEST_LIVENESS_STALE_MS`. Returns `(every_ms, stale_after_ms)`.
pub fn test_overrides(mode: &crate::estate::Mode) -> (u64, u64) {
    let defaults = (NOTE_EVERY_MS, STALE_AFTER_MS);
    if !matches!(mode, crate::estate::Mode::Estate(_)) {
        return defaults;
    }
    let num = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u64>().ok()).filter(|n| *n > 0);
    (
        num("TERMLINK_SIDECAR_TEST_LIVENESS_EVERY_MS").unwrap_or(defaults.0),
        num("TERMLINK_SIDECAR_TEST_LIVENESS_STALE_MS").unwrap_or(defaults.1),
    )
}

/// The hub topic a main posts its notes on, named like its inbox (`inbox:<hub_id>/<agent>`).
pub fn liveness_topic(hub_id: &str, agent: &str) -> String {
    format!("liveness:{hub_id}/{agent}")
}

/// The note. `drained_up_to` is the main's inbox cursor: every inbox offset below it is handled.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Note {
    pub agent: String,
    pub instance: String,
    pub sample_ms: u64,
    pub drained_up_to: u64,
    /// `on`, or `drained` when the operator has stopped receiving (the cursor then holds still on purpose).
    pub receiving: String,
    /// T-3832 (design 3.6, E-61): the reachability card. Optional and additive: a note from a sidecar that
    /// predates P-24 has none, and a pre-P-24 reader ignores it (serde skips unknown fields).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<Card>,
}

/// PN-9: a reachability card older than this has expired; its agent is treated as offline.
pub const CARD_EXPIRE_MS: u64 = 600_000;
/// A valid adapter report within this long means the adapter is present (design 3.6.2).
pub const ADAPTER_PRESENT_MS: u64 = 24 * 3_600_000;
/// Store state holding the last read of each recipient's card (one JSON object, agent -> read).
pub const CARDS_STATE: &str = "recipient-cards";
/// At most this many recipients (the most recently sent to) have their card read each cycle.
pub const CARDS_MAX: usize = 32;

/// T-3832 (design 3.6, E-61): what a sender may know about whether its recipient can be reached. No secret
/// content (3.6.4): the PH-3 read scope that limits other projects to yes/no is recorded as a PH-3 item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Card {
    /// This sidecar's own heartbeat is fresh and no component is stale.
    pub receiver_up: bool,
    /// The hub it reaches is the hub it declares (`hub.match`).
    pub right_hub: bool,
    /// A valid adapter report within [`ADAPTER_PRESENT_MS`] (the meter's report log).
    pub adapter_present: bool,
    /// The last real hand-over to its agent (3.3's value), never a heartbeat time.
    pub last_surface_ms: Option<u64>,
}

/// Test seam (estate only, as [`test_overrides`]): `TERMLINK_SIDECAR_TEST_CARD_EXPIRE_MS`.
pub fn card_expire_ms(mode: &crate::estate::Mode) -> u64 {
    if !matches!(mode, crate::estate::Mode::Estate(_)) {
        return CARD_EXPIRE_MS;
    }
    std::env::var("TERMLINK_SIDECAR_TEST_CARD_EXPIRE_MS").ok().and_then(|v| v.parse::<u64>().ok()).filter(|n| *n > 0).unwrap_or(CARD_EXPIRE_MS)
}

/// Wall time of the newest adapter report the meter's log holds that was not ignored (applied or
/// dropped: the adapter ran and spoke the protocol). Reads the current log, then the rotated one.
pub fn last_valid_report_ms(dir: &std::path::Path) -> Option<u64> {
    let log = dir.join(crate::meter::REPORT_LOG);
    for path in [log.clone(), log.with_extension("log.1")] {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let newest = text
            .lines()
            .rev()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .find(|v| matches!(v["outcome"].as_str(), Some("applied" | "dropped")))
            .and_then(|v| v["ts_ms"].as_u64());
        if newest.is_some() {
            return newest;
        }
    }
    None
}

/// This sidecar's card at `now`, from its own state only.
pub fn own_card(store: &Store, agent: &str, hub_id: &str, now: u64) -> Card {
    let hb = crate::beat::view(store.dir(), now);
    let receiver_up = hb["heartbeat_age_ms"].as_u64().is_some_and(|a| a <= crate::fields::BOUND_HEARTBEAT_MS)
        && hb["stale"].as_array().is_some_and(|s| s.is_empty());
    let right_hub = crate::hub_key_pin::hub_status(store, Some(hub_id))["match"] == Value::Bool(true);
    let adapter_present = last_valid_report_ms(store.dir()).is_some_and(|t| now.saturating_sub(t) <= ADAPTER_PRESENT_MS);
    let st = crate::api::status_view(&json!({}), store, agent, Some(hub_id), now, &crate::deadline::Deadlines::R44E);
    let last_surface_ms = crate::fields::own_handovers(&crate::fields::rows_of(&st["rows"])).max();
    Card { receiver_up, right_hub, adapter_present, last_surface_ms }
}

/// The agents this sidecar sent to most recently, newest first, at most `max`.
pub fn recent_recipients(store: &Store, max: usize) -> Vec<String> {
    let text = store.read_state(crate::revocation::SENT_TO_STATE).ok().flatten().unwrap_or_default();
    let mut out: Vec<String> = vec![];
    for (_, to) in text.lines().rev().filter_map(|l| l.split_once(' ')) {
        let to = to.trim();
        if !to.is_empty() && !out.iter().any(|a| a == to) {
            out.push(to.to_string());
            if out.len() >= max {
                break;
            }
        }
    }
    out
}

/// One read of a recipient's newest note, as stored: `{sample_ms, card}` from a note its own agent posted
/// and a known key signed, or `{error}` naming why there is no card to show.
pub fn read_card(hub: &dyn Hub, hub_id: &str, agent: &str, signers: &dyn Fn(&str) -> bool, now: u64) -> Value {
    let (note, signer) = match read_note(hub, &liveness_topic(hub_id, agent)) {
        Read::Unreadable(why) => return json!({"read_ms": now, "error": format!("hub unreadable: {why}")}),
        Read::NoNote => return json!({"read_ms": now, "error": "no liveness note from this agent"}),
        Read::Note { note, signer, .. } => (note, signer),
    };
    let note = match note {
        Ok(n) => n,
        Err(why) => return json!({"read_ms": now, "error": format!("newest note does not parse: {why}")}),
    };
    if !signers(&signer) {
        return json!({"read_ms": now, "error": format!("newest note is signed by an unknown key {}", crate::pull::shown(&signer))});
    }
    if note.agent != agent {
        return json!({"read_ms": now, "error": format!("newest note is from {}, not {agent}", note.agent)});
    }
    match note.card {
        None => json!({"read_ms": now, "sample_ms": note.sample_ms, "error": "its note carries no card (its sidecar predates P-24)"}),
        Some(c) => json!({"read_ms": now, "sample_ms": note.sample_ms, "card": c}),
    }
}

/// The card a sender shows for `agent` at `now` (design 3.6.3), from the stored read:
/// * `fresh`: a card sampled within `expire_ms` (PN-9), its fields shown;
/// * `expired`: sampled more than PN-9 ago and at most 2 x PN-9: the recipient is offline;
/// * `unknown`: no read, an unreadable hub, no card, or a card older than 2 x PN-9. Never `dead`.
pub fn card_view(read: Option<&Value>, now: u64, expire_ms: u64) -> Value {
    let Some(r) = read else {
        return json!({"state": "unknown", "why": "this sidecar has not read the recipient's card yet"});
    };
    if let Some(e) = r["error"].as_str() {
        return json!({"state": "unknown", "why": e, "read_ms": r["read_ms"]});
    }
    let Some(sample) = r["sample_ms"].as_u64() else {
        return json!({"state": "unknown", "why": "the stored read has no sample time"});
    };
    let age = now.saturating_sub(sample);
    let mut v = json!({"age_ms": age, "sample_ms": sample, "read_ms": r["read_ms"]});
    if age <= expire_ms {
        v["state"] = json!("fresh");
        v["card"] = r["card"].clone();
    } else if age <= 2 * expire_ms {
        v["state"] = json!("expired");
        v["why"] = json!(format!("no card for {age} ms (expires after {expire_ms} ms): the recipient is offline"));
    } else {
        v["state"] = json!("unknown");
        v["why"] = json!(format!("the newest card is {age} ms old, more than 2 x {expire_ms} ms"));
    }
    v
}

/// The stored reads (agent -> read).
pub fn stored_cards(store: &Store) -> serde_json::Map<String, Value> {
    store
        .read_state(CARDS_STATE)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

/// What a read of the hub found.
#[derive(Debug, Clone, PartialEq)]
pub enum Read {
    /// The hub could not be read (unreachable, an error, an unparseable reply).
    Unreadable(String),
    /// No note has ever been posted for this main.
    NoNote,
    /// The newest note: its parsed body (or why it does not parse), the signer's fingerprint as
    /// the hub verified it, and the hub time of the oldest inbox message at or past
    /// `drained_up_to`, if one exists.
    Note { note: Result<Note, String>, signer: String, oldest_waiting_ms: Option<u64> },
}

/// The verdict, rendered for `status`.
#[derive(Debug, Clone, PartialEq)]
pub struct Liveness {
    pub state: &'static str,
    pub note_age_ms: Option<u64>,
    pub drained_up_to: Option<u64>,
    pub reason: String,
}

impl Liveness {
    fn unknown(reason: impl Into<String>) -> Liveness {
        Liveness { state: "unknown", note_age_ms: None, drained_up_to: None, reason: reason.into() }
    }

    pub fn to_json(&self) -> Value {
        json!({"state": self.state, "note_age_ms": self.note_age_ms, "drained_up_to": self.drained_up_to, "reason": self.reason})
    }
}

/// Who the note must come from: the pinned main's name, its provisioned instance (`None` when the
/// main is this agent, whose own instance is checked instead) and the keys allowed to sign it.
pub struct Expect<'a> {
    pub agent: &'a str,
    pub instance: &'a str,
    pub signers: &'a dyn Fn(&str) -> bool,
}

/// Judge a read at `now`. Pure: every state is reachable from a test.
pub fn judge(expect: &Expect, read: &Read, now: u64, stale_after: u64) -> Liveness {
    let (note, signer, oldest) = match read {
        Read::Unreadable(why) => return Liveness::unknown(format!("hub unreadable: {why}")),
        Read::NoNote => return Liveness::unknown("no liveness note from the main yet"),
        Read::Note { note, signer, oldest_waiting_ms } => (note, signer, *oldest_waiting_ms),
    };
    let note = match note {
        Ok(n) => n,
        Err(why) => return Liveness::unknown(format!("newest note does not parse: {why}")),
    };
    if !(expect.signers)(signer) {
        return Liveness::unknown(format!("newest note is signed by an unknown key {}", crate::pull::shown(signer)));
    }
    if note.agent != expect.agent || note.instance != expect.instance {
        return Liveness::unknown(format!(
            "newest note is from {}/{}, not the pinned main {}/{}",
            note.agent, note.instance, expect.agent, expect.instance
        ));
    }
    if note.sample_ms > now.saturating_add(stale_after) {
        return Liveness::unknown("newest note is dated in the future (clocks disagree)");
    }
    let age = now.saturating_sub(note.sample_ms);
    let seen = |state: &'static str, reason: String| Liveness {
        state,
        note_age_ms: Some(age),
        drained_up_to: Some(note.drained_up_to),
        reason,
    };
    if age > stale_after {
        return seen("stale", format!("no liveness note for {age} ms (threshold {stale_after} ms)"));
    }
    if let Some(t) = oldest {
        let waited = note.sample_ms.saturating_sub(t);
        if waited > stale_after {
            let why = if note.receiving == "drained" { " (receiving is drained by the operator)" } else { "" };
            return seen(
                "stuck",
                format!("inbox offset {} had waited {waited} ms undrained when the main last reported{why}", note.drained_up_to),
            );
        }
    }
    seen("alive", String::new())
}

/// One JSON-RPC call to the hub, returning its result or an error message; the error code
/// rides along so `CHANNEL_TOPIC_UNKNOWN` can be told apart.
pub trait Hub {
    fn call(&self, method: &str, params: Value) -> HubResult;
}

/// A hub call's result, or its error code (when the hub sent one) and message.
pub type HubResult = Result<Value, (Option<i64>, String)>;

/// The newest note on `topic` and the oldest waiting message on `inbox` (see [`Read`]).
pub fn read(hub: &dyn Hub, topic: &str, inbox: &str) -> Read {
    let (note, signer) = match read_note(hub, topic) {
        Read::Note { note, signer, .. } => (note, signer),
        other => return other,
    };
    let oldest_waiting_ms = match &note {
        Err(_) => None,
        // T-3700: the timestamp only, so this check is not a consumer of the main's inbox (AL-30)
        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({"topic": inbox, "from_offset": n.drained_up_to})) {
            Ok(v) => v.get("ts").and_then(Value::as_u64).filter(|_| v.get("offset").and_then(Value::as_u64).is_some()),
            Err((Some(code), _)) if code == error_code::CHANNEL_TOPIC_UNKNOWN => None,
            // r1 CC 9 / OC 3: only a hub that predates channel.first_at falls back to the one-message read. A
            // T-3700 hub's own errors name the method; an old hub routes the unknown method to a session
            // (-32001 "Missing 'target'"). A transport error (no code) is Unreadable, never a content read.
            // r2 CC N3: exactly the old hub's answer (the unknown method routed to a session), nothing looser
            Err((Some(code), m)) if code == error_code::SESSION_NOT_FOUND && m.contains("Missing 'target'") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
                Ok(t) => t,
                Err(e) => return Read::Unreadable(e),
            },
            Err((_, e)) => return Read::Unreadable(format!("channel.first_at {inbox}: {e}")),
        },
    };
    Read::Note { note, signer, oldest_waiting_ms }
}

/// The newest note on `topic` only (no inbox read; `oldest_waiting_ms` is always `None`).
pub fn read_note(hub: &dyn Hub, topic: &str) -> Read {
    let listed = match hub.call(method::CHANNEL_LIST, json!({"prefix": topic})) {
        Ok(v) => v,
        Err((_, e)) => return Read::Unreadable(format!("channel.list: {e}")),
    };
    let Some(topics) = listed.get("topics").and_then(Value::as_array) else {
        return Read::Unreadable("channel.list: no topics array".into());
    };
    let Some(entry) = topics.iter().find(|t| t.get("name").and_then(Value::as_str) == Some(topic)) else {
        return Read::NoNote;
    };
    let Some(latest) = entry.get("latest_offset").and_then(Value::as_u64) else {
        return Read::NoNote;
    };
    let msg = match hub.call(method::CHANNEL_SUBSCRIBE, json!({"topic": topic, "cursor": latest, "limit": 1})) {
        Ok(v) => v.get("messages").and_then(Value::as_array).and_then(|m| m.first().cloned()),
        Err((_, e)) => return Read::Unreadable(format!("channel.subscribe {topic}: {e}")),
    };
    let Some(msg) = msg else {
        return Read::Unreadable(format!("{topic}: latest offset {latest} returned no message"));
    };
    let signer = msg.get("sender_id").and_then(Value::as_str).unwrap_or("").to_owned();
    let note = if msg.get("msg_type").and_then(Value::as_str) != Some(NOTE_TYPE) {
        Err(format!("msg_type is not {NOTE_TYPE}"))
    } else {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(msg.get("payload_b64").and_then(Value::as_str).unwrap_or(""))
            .map_err(|e| e.to_string())
            .and_then(|b| serde_json::from_slice::<Note>(&b).map_err(|e| e.to_string()))
    };
    Read::Note { note, signer, oldest_waiting_ms: None }
}

/// The pre-T-3700 way to find the oldest waiting message: read one message at `from`.
fn oldest_by_subscribe(hub: &dyn Hub, inbox: &str, from: u64) -> Result<Option<u64>, String> {
    Ok(match hub.call(method::CHANNEL_SUBSCRIBE, json!({"topic": inbox, "cursor": from, "limit": 1})) {
        Ok(v) => v
            .get("messages")
            .and_then(Value::as_array)
            .and_then(|m| m.iter().find(|e| e.get("offset").and_then(Value::as_u64).is_some_and(|o| o >= from)))
            .map(|e| e.get("ts").and_then(Value::as_u64).unwrap_or(0)),
        Err((Some(code), _)) if code == error_code::CHANNEL_TOPIC_UNKNOWN => None,
        Err((_, e)) => return Err(format!("channel.subscribe {inbox}: {e}")),
    })
}

/// Move the alarm latch for a verdict; returns the needs-attention entry written, if any.
/// The latch lives in the store, so a restart neither repeats nor forgets an alarm.
pub fn alarm(store: &Store, main: &str, v: &Liveness) -> Option<&'static str> {
    let last = store.read_state(ALARM_STATE).ok().flatten().map(|s| s.trim().to_owned()).unwrap_or_else(|| "ok".into());
    let (kind, next) = match v.state {
        "stale" if last != "stale" => (ALERT_STALE, "stale"),
        "stuck" if last != "stuck" => (ALERT_STUCK, "stuck"),
        "alive" if last == "stale" || last == "stuck" => (ALERT_RECOVERED, "ok"),
        _ => return None,
    };
    let detail = match kind {
        ALERT_RECOVERED => format!("pinned main {main} is alive again (was {last})"),
        _ => format!("pinned main {main} is {}: {}", v.state, v.reason),
    };
    // Latch first: a failed latch write must not turn into an entry per cycle.
    if let Err(e) = store.write_state(ALARM_STATE, next) {
        eprintln!("termlink-sidecar: main liveness alarm latch not written: {e}");
        return None;
    }
    if let Err(e) = store.attention(kind, &detail) {
        eprintln!("termlink-sidecar: needs attention ({kind}): {detail} [could not record it: {e}]");
    }
    Some(kind)
}

/// The note this sidecar would post now, if it is the pinned main.
pub fn own_note(store: &Store, agent: &str, instance: &str, now: u64) -> Note {
    let drained_up_to = store
        .read_state(crate::pull::CURSOR_STATE)
        .ok()
        .flatten()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
    let receiving = if crate::policy::receiving_enabled(store) { "on" } else { "drained" };
    Note { agent: agent.into(), instance: instance.into(), sample_ms: now, drained_up_to, receiving: receiving.into(), card: None }
}

/// The last verdict and when it was reached; `status` reads it.
#[derive(Default)]
pub struct Shared {
    last: Mutex<Option<(u64, Liveness)>>,
}

impl Shared {
    pub fn new() -> Arc<Shared> {
        Arc::new(Shared::default())
    }

    pub fn set(&self, at: u64, v: Liveness) {
        if let Ok(mut l) = self.last.lock() {
            *l = Some((at, v));
        }
    }

    /// The `main.liveness` object at `now`. A verdict older than three checks means this
    /// sidecar's own monitor is not running: unknown, never the old answer.
    pub fn view(&self, now: u64, every: u64) -> Value {
        let last = self.last.lock().ok().and_then(|l| l.clone());
        match last {
            None => Liveness::unknown("not checked yet").to_json(),
            Some((at, _)) if now.saturating_sub(at) > 3 * every => {
                Liveness::unknown(format!("this sidecar's liveness check last ran {} ms ago", now.saturating_sub(at))).to_json()
            }
            Some((at, mut v)) => {
                // Age the note by the time since the check, so the number is current.
                v.note_age_ms = v.note_age_ms.map(|a| a + now.saturating_sub(at));
                let mut j = v.to_json();
                j["checked_age_ms"] = json!(now.saturating_sub(at));
                j
            }
        }
    }
}

/// `main.liveness` for `status`: the monitor's verdict, or `unknown` naming why there is none.
pub fn status_view(book: Option<&Book>, monitor: Option<(&Shared, u64)>, now: u64) -> Value {
    match (book, monitor) {
        (None, _) => Liveness::unknown("no address book").to_json(),
        (Some(b), _) if b.main.is_none() => Liveness::unknown("no main is pinned in the address book").to_json(),
        (Some(_), None) => Liveness::unknown("this sidecar runs no liveness check").to_json(),
        (Some(_), Some((sh, every))) => sh.view(now, every),
    }
}

/// The real hub, one call at a time on a private runtime.
pub struct RpcHub {
    pub addr: TransportAddr,
    pub timeout: Duration,
    rt: tokio::runtime::Runtime,
}

impl RpcHub {
    pub fn new(addr: TransportAddr, timeout: Duration) -> std::io::Result<RpcHub> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        Ok(RpcHub { addr, timeout, rt })
    }
}

impl Hub for RpcHub {
    fn call(&self, m: &str, params: Value) -> HubResult {
        let resp = self
            .rt
            .block_on(termlink_session::client::rpc_call_addr_with_timeout(&self.addr, m, params, self.timeout))
            .map_err(|e| (None, e.to_string()))?;
        match resp {
            RpcResponse::Success(s) => Ok(s.result),
            RpcResponse::Error(e) => Err((Some(e.error.code), e.error.message)),
        }
    }
}

/// Everything one cycle needs.
pub struct Monitor {
    pub book: Book,
    pub agent: String,
    pub instance: String,
    pub hub_id: String,
    pub roster: Roster,
    pub own_fingerprint: String,
    pub store: Arc<Store>,
    pub sender: Arc<crate::send::Sender>,
    pub shared: Arc<Shared>,
    pub every_ms: u64,
    pub stale_after_ms: u64,
}

impl Monitor {
    /// One cycle: post this sidecar's own note with its card (every sidecar, T-3832 design 3.6.1), read the
    /// cards of the agents it sent to most recently, then read and judge the main's newest note and move the
    /// alarm latch. Returns the main's verdict.
    pub fn cycle(&self, hub: &dyn Hub, now: u64) -> Liveness {
        let mut note = own_note(&self.store, &self.agent, &self.instance, now);
        note.card = Some(own_card(&self.store, &self.agent, &self.hub_id, now));
        let body = serde_json::to_vec(&note).unwrap_or_default();
        let topic = liveness_topic(&self.hub_id, &self.agent);
        if let Err(e) = self.sender.post_note(&topic, NOTE_TYPE, &body, json!({"kind": "messages", "value": NOTE_KEEP})) {
            eprintln!("termlink-sidecar: liveness note not posted: {e}");
        }
        self.read_cards(hub, now);
        let Some(main) = self.book.main.as_deref() else {
            let v = Liveness::unknown("no main is pinned in the address book");
            self.shared.set(now, v.clone());
            return v;
        };
        let is_self = main == self.agent;
        let instance = if is_self { self.instance.as_str() } else { self.book.peers.get(main).map(String::as_str).unwrap_or("") };
        let signers = |fp: &str| fp == self.own_fingerprint || self.roster.get(fp).is_some();
        let expect = Expect { agent: main, instance, signers: &signers };
        let r = read(hub, &liveness_topic(&self.hub_id, main), &inbox_topic(&self.hub_id, main));
        // T-3708 (FCW-120): the main's liveness verdict is fresh only from a read the hub answered
        if !matches!(r, Read::Unreadable(_)) {
            crate::fields::touch(crate::fields::LIVENESS);
        }
        let v = judge(&expect, &r, now, self.stale_after_ms);
        alarm(&self.store, main, &v);
        self.shared.set(now, v.clone());
        v
    }

    /// Read and store the card of each agent this sidecar sent to most recently (design 3.6.3: recipients are
    /// addressed on this sidecar's declared hub, so that hub holds their notes). Each read replaces the last
    /// one, so an unreadable hub shows as `unknown`, never as an expired (offline) recipient.
    pub fn read_cards(&self, hub: &dyn Hub, now: u64) {
        let recipients = recent_recipients(&self.store, CARDS_MAX);
        if recipients.is_empty() {
            return;
        }
        let signers = |fp: &str| fp == self.own_fingerprint || self.roster.get(fp).is_some();
        let mut out = serde_json::Map::new();
        for to in recipients {
            let r = read_card(hub, &self.hub_id, &to, &signers, now);
            out.insert(to, r);
        }
        if let Err(e) = self.store.write_state(CARDS_STATE, &Value::Object(out).to_string()) {
            eprintln!("termlink-sidecar: recipient cards not written: {e}");
        }
    }

    /// Run [`Monitor::cycle`] every `every_ms` on its own thread until `stop` is set.
    pub fn spawn(self, hub: RpcHub, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let every = Duration::from_millis(self.every_ms);
            let mut waited = every; // first cycle at once
            // Not a component beat: a hub outage slows a cycle to its timeouts, and that must
            // not restart the sidecar. A hung check shows instead as `unknown` in `status`.
            while !stop.load(Ordering::SeqCst) {
                if waited >= every {
                    waited = Duration::ZERO;
                    self.cycle(&hub, crate::store::now_ms());
                }
                std::thread::sleep(Duration::from_millis(100));
                waited += Duration::from_millis(100);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    const NOW: u64 = 10_000_000;
    const STALE: u64 = STALE_AFTER_MS;

    fn note(sample_ms: u64, drained: u64) -> Note {
        Note { agent: "main-a".into(), instance: "main-a.w1".into(), sample_ms, drained_up_to: drained, receiving: "on".into(), card: None }
    }

    fn known(fp: &str) -> bool {
        fp == "fp-main"
    }

    fn expect() -> Expect<'static> {
        Expect { agent: "main-a", instance: "main-a.w1", signers: &known }
    }

    fn seen(n: Note, oldest: Option<u64>) -> Read {
        Read::Note { note: Ok(n), signer: "fp-main".into(), oldest_waiting_ms: oldest }
    }

    #[test]
    fn a_fresh_note_with_no_waiting_mail_is_alive() {
        let v = judge(&expect(), &seen(note(NOW - 30_000, 7), None), NOW, STALE);
        assert_eq!(v.state, "alive");
        assert_eq!(v.note_age_ms, Some(30_000));
        assert_eq!(v.drained_up_to, Some(7));
        // Mail that had only just arrived when the note was sampled is not stuck.
        let v = judge(&expect(), &seen(note(NOW - 30_000, 7), Some(NOW - 60_000)), NOW, STALE);
        assert_eq!(v.state, "alive");
    }

    #[test]
    fn an_old_note_is_stale() {
        let v = judge(&expect(), &seen(note(NOW - STALE - 1, 7), None), NOW, STALE);
        assert_eq!(v.state, "stale");
        assert_eq!(v.note_age_ms, Some(STALE + 1));
        // Stale wins over stuck: a silent main says nothing about its cursor now.
        let v = judge(&expect(), &seen(note(NOW - STALE - 1, 7), Some(0)), NOW, STALE);
        assert_eq!(v.state, "stale");
    }

    #[test]
    fn a_fresh_note_with_mail_waiting_past_the_threshold_is_stuck() {
        let sample = NOW - 10_000;
        let v = judge(&expect(), &seen(note(sample, 7), Some(sample - STALE - 1)), NOW, STALE);
        assert_eq!(v.state, "stuck");
        assert_eq!(v.drained_up_to, Some(7));
        assert!(v.reason.contains("offset 7"), "{}", v.reason);
        let mut drained = note(sample, 7);
        drained.receiving = "drained".into();
        let v = judge(&expect(), &seen(drained, Some(0)), NOW, STALE);
        assert_eq!(v.state, "stuck");
        assert!(v.reason.contains("drained by the operator"));
    }

    #[test]
    fn anything_not_proven_is_unknown_never_alive() {
        let fresh = || note(NOW - 1_000, 3);
        let cases: Vec<(Read, &str)> = vec![
            (Read::Unreadable("connection refused".into()), "hub unreadable"),
            (Read::NoNote, "no liveness note"),
            (Read::Note { note: Err("eof".into()), signer: "fp-main".into(), oldest_waiting_ms: None }, "does not parse"),
            (Read::Note { note: Ok(fresh()), signer: "fp-stranger".into(), oldest_waiting_ms: None }, "unknown key"),
            (seen(Note { agent: "someone-else".into(), ..fresh() }, None), "not the pinned main"),
            (seen(Note { instance: "main-a.old".into(), ..fresh() }, None), "not the pinned main"),
            (seen(note(NOW + STALE + 1, 3), None), "future"),
        ];
        for (r, why) in cases {
            let v = judge(&expect(), &r, NOW, STALE);
            assert_eq!(v.state, "unknown", "{r:?}");
            assert!(v.reason.contains(why), "{why}: {}", v.reason);
            assert_eq!(v.note_age_ms, None);
        }
    }

    /// T-3700 (AL-30): with `channel.first_at` the main's inbox is never read for content, so this
    /// check is not a consumer of it; the subscribe path is only the fallback for older hubs.
    #[test]
    fn read_uses_first_at_and_never_reads_the_inbox_content() {
        let n = note(NOW, 2);
        let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 999)]})));
        f.replies.insert(("channel.first_at".into(), "I".into()), Ok(json!({"offset": 2, "ts": 1234})));
        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n.clone()), signer: "fp-main".into(), oldest_waiting_ms: Some(1234) });
        assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"), "{:?}", f.calls.borrow());
        // nothing waiting at or after the offset
        let mut f = fake(&n, Ok(json!({"messages": []})));
        f.replies.insert(("channel.first_at".into(), "I".into()), Ok(json!({"offset": null, "ts": null})));
        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n), signer: "fp-main".into(), oldest_waiting_ms: None });
        assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"));
    }

    /// r1 CC 9 / OC 3: only an old hub (-32001, the unknown method routed to a session) falls back to the
    /// content read; a transport error or a T-3700 hub's own first_at error is Unreadable.
    #[test]
    fn read_falls_back_only_for_an_old_hub() {
        let n = note(NOW, 2);
        let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
        f.replies.insert(("channel.first_at".into(), "I".into()), Err((Some(-32001), "Missing 'target' in params".into())));
        assert!(matches!(read(&f, "L", "I"), Read::Note { oldest_waiting_ms: Some(1234), .. }));
        for err in [(None, "connection reset".to_string()), (Some(-32603), "channel.first_at: io".to_string()), (Some(-32603), "bus unavailable".to_string())] {
            let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
            f.replies.insert(("channel.first_at".into(), "I".into()), Err(err.clone()));
            assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)), "{err:?}");
            assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"), "{err:?}");
        }
    }

    /// A fake hub: canned replies per (method, topic).
    struct Fake {
        replies: BTreeMap<(String, String), HubResult>,
        calls: RefCell<Vec<String>>,
    }

    impl Hub for Fake {
        fn call(&self, m: &str, params: Value) -> HubResult {
            let t = params.get("topic").or_else(|| params.get("prefix")).and_then(Value::as_str).unwrap_or("").to_owned();
            self.calls.borrow_mut().push(format!("{m} {t}"));
            self.replies.get(&(m.to_owned(), t)).cloned().unwrap_or(Err((None, "no reply".into())))
        }
    }

    fn envelope(offset: u64, msg_type: &str, payload: &[u8], ts: u64) -> Value {
        json!({"offset": offset, "msg_type": msg_type, "sender_id": "fp-main", "ts": ts,
               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload)})
    }

    fn fake(n: &Note, inbox: HubResult) -> Fake {
        let body = serde_json::to_vec(n).unwrap();
        Fake {
            replies: BTreeMap::from([
                (("channel.list".into(), "L".into()), Ok(json!({"topics": [{"name": "L", "latest_offset": 4}, {"name": "L2"}]}))),
                (("channel.subscribe".into(), "L".into()), Ok(json!({"messages": [envelope(4, NOTE_TYPE, &body, n.sample_ms)]}))),
                (("channel.subscribe".into(), "I".into()), inbox),
                // an old hub by default: the unknown method is routed to a session (T-3700 fallback)
                (("channel.first_at".into(), "I".into()), Err((Some(-32001), "Missing 'target' in params".into()))),
            ]),
            calls: RefCell::new(vec![]),
        }
    }

    #[test]
    fn read_takes_the_newest_note_and_the_first_waiting_message() {
        let n = note(NOW, 2);
        let f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n.clone()), signer: "fp-main".into(), oldest_waiting_ms: Some(1234) });
        let f = fake(&n, Ok(json!({"messages": []})));
        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n.clone()), signer: "fp-main".into(), oldest_waiting_ms: None });
        let f = fake(&n, Err((Some(error_code::CHANNEL_TOPIC_UNKNOWN), "unknown topic".into())));
        assert!(matches!(read(&f, "L", "I"), Read::Note { oldest_waiting_ms: None, .. }));
    }

    #[test]
    fn read_fails_closed_on_any_hub_error() {
        let n = note(NOW, 2);
        // The inbox read fails for a reason other than "no such topic": unreadable, not "no mail".
        let f = fake(&n, Err((None, "timed out".into())));
        assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)));
        let mut f = fake(&n, Ok(json!({"messages": []})));
        f.replies.insert(("channel.list".into(), "L".into()), Err((None, "refused".into())));
        assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)));
        f.replies.insert(("channel.list".into(), "L".into()), Ok(json!({"nope": 1})));
        assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)));
        f.replies.insert(("channel.list".into(), "L".into()), Ok(json!({"topics": []})));
        assert_eq!(read(&f, "L", "I"), Read::NoNote);
        // And every one of them judges unknown.
        f.replies.insert(("channel.list".into(), "L".into()), Err((None, "refused".into())));
        assert_eq!(judge(&expect(), &read(&f, "L", "I"), NOW, STALE).state, "unknown");
    }

    #[test]
    fn a_note_of_another_type_is_not_a_note() {
        let n = note(NOW, 2);
        let mut f = fake(&n, Ok(json!({"messages": []})));
        f.replies.insert(("channel.subscribe".into(), "L".into()), Ok(json!({"messages": [envelope(4, "sidecar.msg.v1", b"{}", NOW)]})));
        let r = read(&f, "L", "I");
        assert!(matches!(&r, Read::Note { note: Err(_), .. }));
        assert_eq!(judge(&expect(), &r, NOW, STALE).state, "unknown");
    }

    /// A store inside a throwaway estate (no hub); the estate is torn down when dropped.
    fn temp_store() -> (crate::testkit::Estate, std::path::PathBuf, Store) {
        let e = crate::testkit::Estate::create_without_hub().unwrap();
        let root = e.store_root();
        let s = Store::open(&root, "agent-x").unwrap();
        (e, root, s)
    }

    fn kinds(s: &Store) -> Vec<String> {
        s.attention_entries()
            .iter()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter_map(|v| v["kind"].as_str().map(str::to_owned))
            .collect()
    }

    #[test]
    fn the_alarm_fires_once_on_entry_and_once_on_recovery() {
        let (_e, root, s) = temp_store();
        let lv = |state: &'static str| Liveness { state, note_age_ms: Some(1), drained_up_to: Some(1), reason: "r".into() };
        for st in ["alive", "unknown", "stale", "stale", "stale", "unknown", "stale", "stuck", "stuck", "alive", "alive", "unknown"] {
            alarm(&s, "main-a", &lv(st));
        }
        assert_eq!(kinds(&s), vec![ALERT_STALE, ALERT_STUCK, ALERT_RECOVERED]);
        // The latch survives a restart: a new store handle does not fire again.
        drop(s);
        let s = Store::open(&root, "agent-x").unwrap();
        alarm(&s, "main-a", &lv("alive"));
        assert_eq!(kinds(&s).len(), 3);
        alarm(&s, "main-a", &lv("stuck"));
        drop(s);
        let s = Store::open(&root, "agent-x").unwrap();
        alarm(&s, "main-a", &lv("stuck"));
        assert_eq!(kinds(&s), vec![ALERT_STALE, ALERT_STUCK, ALERT_RECOVERED, ALERT_STUCK]);
    }

    #[test]
    fn status_view_is_unknown_until_checked_and_when_the_check_stops() {
        let sh = Shared::new();
        assert_eq!(sh.view(NOW, NOTE_EVERY_MS)["state"], "unknown");
        sh.set(NOW, Liveness { state: "alive", note_age_ms: Some(1_000), drained_up_to: Some(5), reason: String::new() });
        let v = sh.view(NOW + 2_000, NOTE_EVERY_MS);
        assert_eq!(v["state"], "alive");
        assert_eq!(v["note_age_ms"], 3_000);
        assert_eq!(v["drained_up_to"], 5);
        assert_eq!(sh.view(NOW + 3 * NOTE_EVERY_MS + 1, NOTE_EVERY_MS)["state"], "unknown");
    }

    #[test]
    fn own_note_reports_the_inbox_cursor_and_receiving() {
        let (_e, _root, s) = temp_store();
        assert_eq!(own_note(&s, "a", "a.w1", 5).drained_up_to, 0);
        s.write_state(crate::pull::CURSOR_STATE, "42").unwrap();
        let n = own_note(&s, "a", "a.w1", 5);
        assert_eq!((n.drained_up_to, n.sample_ms, n.receiving.as_str()), (42, 5, "on"));
    }
}
