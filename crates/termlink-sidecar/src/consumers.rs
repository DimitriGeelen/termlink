//! T-3700 (arc-011 P-12, B-2, 06a AL-30): the sidecar's per-tick check of who consumes its inbox.
//!
//! The hub lists every reader of the inbox (`inbox.consumers`), `proven` (signed by its key-card key) or
//! `claimed` (operator ruling E, 2026-11-19). Any reader but this sidecar within the V-11 window (24 h) is
//! "inbox consumed by two identities: A, B" in status and a needs-attention entry, coalesced per
//! (inbox, reader, UTC day). The same agent at another instance is the V-12 second copy. A list that
//! cannot be read is `unknown`, never "one" (FCW-125).

use serde_json::{Value, json};

use crate::store::Store;

/// Store state holding the last status line (JSON `{consumers, bound}`).
pub const STATUS_STATE: &str = "inbox-consumers";
/// V-11: the window the hub is asked about.
pub const WINDOW_S: u64 = 86_400;
/// How often the pass loop asks (well inside E-66's 60 s).
pub const CHECK_EVERY_MS: u64 = 30_000;
/// The instance the hub records for an ack (a receipt post names no instance).
pub const ACK_INSTANCE: &str = "-";
/// Attention kind for a second consumer.
pub const ATTENTION_KIND: &str = "inbox-second-consumer";
/// JSON-RPC "method not found".
const METHOD_NOT_FOUND: i64 = -32601;

/// One reader as the hub reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub agent: String,
    pub instance: String,
    pub strength: String,
    pub why: String,
    pub key_fp: String,
}

impl Seen {
    fn shown(&self) -> String {
        let s = if self.why.is_empty() { self.strength.clone() } else { format!("{}: {}", self.strength, self.why) };
        format!("{} ({s})", self.agent)
    }
}

/// What one check concluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The `consumers` status line.
    pub line: String,
    /// Readers other than this sidecar (each is a needs-attention entry).
    pub others: Vec<Seen>,
    /// The agent the inbox is bound to, if any.
    pub bound: Option<String>,
}

/// Classify the hub's answer to `inbox.consumers` (`Err((code, message))` when the call failed). `me_fp` is this
/// sidecar's own key fingerprint: only an entry PROVEN by that key is this sidecar (r1 CC 1 / OC 1); a name, an
/// instance or the ack sentinel alone never is.
pub fn verdict(me_agent: &str, me_instance: &str, me_fp: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
    let unknown = |why: String| Verdict { line: format!("unknown ({why})"), others: vec![], bound: None };
    let v = match answer {
        Ok(v) => v,
        Err((METHOD_NOT_FOUND, _)) => return unknown("hub predates the consumer list".into()),
        Err((_, m)) => return unknown(m.to_string()),
    };
    let Some(list) = v.get("consumers").and_then(Value::as_array) else {
        // FCW-125: a malformed answer is not "one"
        return unknown("the hub's answer holds no consumer list".into());
    };
    let bound = v.get("bound").and_then(|b| b.get("agent")).and_then(Value::as_str).map(str::to_string);
    // r2 OC 2 / r3 CC 5, OC 2: without its own fingerprint the sidecar cannot recognise itself. Hub errors are
    // reported first (above); readers under another agent's name are still flagged; the line is unknown.
    if me_fp.is_empty() {
        let others: Vec<Seen> = list
            .iter()
            .filter(|e| e.get("agent").and_then(Value::as_str) != Some(me_agent))
            .map(|e| {
                let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
                Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why"), key_fp: s("key_fp") }
            })
            .collect();
        return Verdict { line: "unknown (this sidecar's own key fingerprint is unavailable)".into(), others, bound };
    }
    let mut mine: Vec<Seen> = Vec::new();
    let mut others: Vec<Seen> = Vec::new();
    for e in list {
        let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let seen = Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why"), key_fp: s("key_fp") };
        let is_me = seen.agent == me_agent
            && seen.strength == "proven"
            && seen.key_fp == me_fp
            && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);
        if is_me { mine.push(seen) } else { others.push(seen) }
    }
    let line = if !others.is_empty() {
        let me_shown = mine
            .iter()
            .find(|m| m.instance == me_instance)
            .or(mine.first())
            .map(Seen::shown)
            .unwrap_or_else(|| format!("{me_agent} (not seen)"));
        let n = others.len() + 1;
        let count = if n == 2 { "two".to_string() } else { n.to_string() };
        let names: Vec<String> = std::iter::once(me_shown)
            .chain(others.iter().map(|o| {
                if o.agent == me_agent && o.strength == "proven" {
                    // proven by a key that is not this sidecar's: another copy of this agent (V-12)
                    format!("{} [second copy, instance {}]", o.shown(), o.instance)
                } else if o.agent == me_agent {
                    format!("{} [claims this agent, instance {}]", o.shown(), o.instance)
                } else {
                    o.shown()
                }
            }))
            .collect();
        format!("inbox consumed by {count} identities: {}", names.join(", "))
    } else {
        match mine.iter().find(|m| m.instance == me_instance).or(mine.first()) {
            Some(m) if m.strength == "proven" => "one (proven)".to_string(),
            Some(m) => format!("one ({}: {})", m.strength, if m.why.is_empty() { "?" } else { &m.why }),
            None => "no reader in the window".to_string(),
        }
    };
    Verdict { line, others, bound }
}

/// How old a check may be before `status` shows `unknown` (r1 CC 2).
pub const STALE_AFTER_MS: u64 = 2 * CHECK_EVERY_MS;

/// Record a verdict: the status line, and one needs-attention entry per other reader per UTC day.
pub fn record(store: &Store, topic: &str, v: &Verdict, now_ms: u64) {
    let state = json!({"consumers": v.line, "bound": v.bound, "checked_ms": now_ms}).to_string();
    if let Err(e) = store.write_state(STATUS_STATE, &state) {
        eprintln!("termlink-sidecar: consumers status not written: {e}");
    }
    let day = now_ms / 86_400_000;
    for o in &v.others {
        let key = format!("{topic}|{}|{}|{day}", o.agent, o.instance);
        let detail = format!("{} (instance {}) also consumes {topic}: {}", o.shown(), o.instance, v.line);
        if let Err(e) = store.attention_coalesced(ATTENTION_KIND, &key, &detail) {
            eprintln!("termlink-sidecar: needs attention ({ATTENTION_KIND}): {detail} [could not record it: {e}]");
        }
    }
}

/// Forget the last check (at start: a line from before a restart is never shown, r1 CC 2).
pub fn reset(store: &Store) {
    let _ = store.write_state(STATUS_STATE, "{}");
}

/// The `consumers` status (and `bound`): `unknown (no check yet)` before the first check, and
/// `unknown (no check since …)` when the last one is older than [`STALE_AFTER_MS`] (r1 CC 2).
pub fn status(store: &Store, now_ms: u64) -> Value {
    let v = store.read_state(STATUS_STATE).ok().flatten().and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let checked = v.as_ref().and_then(|v| v.get("checked_ms")).and_then(Value::as_u64);
    match (v, checked) {
        (Some(v), Some(t)) if now_ms.saturating_sub(t) <= STALE_AFTER_MS => v,
        (_, Some(t)) => json!({"consumers": format!("unknown (no check since {t})"), "bound": null, "checked_ms": t}),
        _ => json!({"consumers": "unknown (no check yet)", "bound": null}),
    }
}

/// T-3708 (FCW-120, r1 MAJOR 4): whether a check's verdict stamps the `consumers` field fresh. Only a
/// verdict from a list the hub answered does; an `unknown (…)` verdict (no answer, an error, a malformed
/// list) leaves the field to go unknown at its bound.
pub fn stamps_field(v: &Verdict) -> bool {
    !v.line.starts_with("unknown")
}

#[cfg(test)]
mod tests {
    #[test]
    fn t3708_only_an_answered_list_stamps_the_field() {
        let me = "ed25519:mine";
        let unanswered = super::verdict("fx", "i1", me, Err((0, "hub unreachable: refused")));
        assert!(!super::stamps_field(&unanswered), "{}", unanswered.line);
        let odd = super::verdict("fx", "i1", me, Ok(&serde_json::json!({"nothing": 1})));
        assert!(!super::stamps_field(&odd), "{}", odd.line);
        let own = serde_json::json!({"bound": null, "consumers": [
            {"agent": "fx", "instance": "i1", "strength": "proven", "key_fp": me, "actions": ["read"], "first_ms": 1, "last_ms": 2}]});
        let one = super::verdict("fx", "i1", me, Ok(&own));
        assert_eq!(one.line, "one (proven)");
        assert!(super::stamps_field(&one));
    }

    use super::*;

    const FP: &str = "ed25519:mine";

    fn e(agent: &str, instance: &str, strength: &str, why: &str) -> Value {
        let fp = if strength == "proven" { FP } else { "" };
        json!({"agent": agent, "instance": instance, "strength": strength, "why": why, "key_fp": fp})
    }

    fn v(list: Vec<Value>) -> Value {
        json!({"consumers": list, "bound": null})
    }

    #[test]
    fn only_me_proven_is_one() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "-", "proven", "")]);
        let r = verdict("fx-a", "w1", FP, Ok(&a));
        assert_eq!(r.line, "one (proven)");
        assert!(r.others.is_empty());
    }

    #[test]
    fn a_second_identity_is_named_with_its_strength() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
        let r = verdict("fx-a", "w1", FP, Ok(&a));
        assert_eq!(r.line, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
        assert_eq!(r.others.len(), 1);
    }

    #[test]
    fn same_agent_other_instance_is_a_second_copy() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "w2", "proven", "")]);
        let r = verdict("fx-a", "w1", FP, Ok(&a));
        assert!(r.line.contains("second copy, instance w2"), "{}", r.line);
    }

    #[test]
    fn unreadable_is_unknown_never_one() {
        assert!(verdict("fx-a", "w1", FP, Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
        assert_eq!(verdict("fx-a", "w1", FP, Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
        assert!(verdict("fx-a", "w1", FP, Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
        let r = verdict("fx-a", "w1", "", Ok(&v(vec![e("fx-a", "w1", "proven", ""), e("fx-evil", "x", "claimed", "unsigned")])));
        assert!(r.line.starts_with("unknown ("), "{}", r.line);
        assert_eq!(r.others.len(), 1, "another agent is still flagged (r3 CC 5)");
        // a hub error is reported as itself, not masked by the missing fingerprint (r3 OC 2)
        assert_eq!(verdict("fx-a", "w1", "", Err((-32601, "x"))).line, "unknown (hub predates the consumer list)");
    }

    #[test]
    fn my_skewed_clock_is_flagged_not_hidden() {
        // an unproven read under my name cannot be told from a spoof (r1 CC 1), so it is flagged; a skewed
        // clock deserves the attention anyway (P-23 reports the skew)
        let a = v(vec![e("fx-a", "w1", "claimed", "clock skew")]);
        let r = verdict("fx-a", "w1", FP, Ok(&a));
        assert_eq!(r.line, "inbox consumed by two identities: fx-a (not seen), fx-a (claimed: clock skew) [claims this agent, instance w1]");
        assert_eq!(r.others.len(), 1);
    }

    #[test]
    fn attention_once_per_reader_per_day() {
        let est = crate::testkit::Estate::create_without_hub().unwrap();
        let store = Store::open(&est.store_root(), "fx-a").unwrap();
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
        let r = verdict("fx-a", "w1", FP, Ok(&a));
        record(&store, "inbox:h/fx-a", &r, 1_000);
        record(&store, "inbox:h/fx-a", &r, 2_000);
        let n = store.attention_entries().iter().filter(|l| l.contains(ATTENTION_KIND)).count();
        assert_eq!(n, 1);
        assert_eq!(status(&store, 2_000)["consumers"], r.line);
        // r1 CC 2: an old check is not shown as current; a reset forgets it
        assert!(status(&store, 2_000 + STALE_AFTER_MS + 1)["consumers"].as_str().unwrap().starts_with("unknown (no check since"));
        reset(&store);
        assert_eq!(status(&store, 2_000)["consumers"], "unknown (no check yet)");
    }

    #[test]
    fn the_owners_name_without_its_key_is_not_me() {
        // r1 CC 1 / OC 1: an unsigned reader using my agent and my instance, or the ack sentinel, is flagged
        for inst in ["w1", "-"] {
            let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", inst, "claimed", "unsigned")]);
            let r = verdict("fx-a", "w1", FP, Ok(&a));
            assert_eq!(r.others.len(), 1, "{inst}: {}", r.line);
        }
        // a proven entry by another key under my name is a second copy, not me
        let a = json!({"consumers": [e("fx-a", "w1", "proven", ""),
            {"agent": "fx-a", "instance": "w1", "strength": "proven", "why": "", "key_fp": "ed25519:other"}], "bound": null});
        assert_eq!(verdict("fx-a", "w1", FP, Ok(&a)).others.len(), 1);
    }
}
