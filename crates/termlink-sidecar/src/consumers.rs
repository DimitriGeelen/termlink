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

/// Classify the hub's answer to `inbox.consumers` (`Err((code, message))` when the call failed).
pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
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
    let mut mine: Vec<Seen> = Vec::new();
    let mut others: Vec<Seen> = Vec::new();
    for e in list {
        let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let seen = Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why") };
        let is_me = seen.agent == me_agent && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);
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
                if o.agent == me_agent {
                    format!("{} [second copy, instance {}]", o.shown(), o.instance)
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

/// Record a verdict: the status line, and one needs-attention entry per other reader per UTC day.
pub fn record(store: &Store, topic: &str, v: &Verdict, now_ms: u64) {
    let state = json!({"consumers": v.line, "bound": v.bound}).to_string();
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

/// The `consumers` status (and `bound`), `unknown (no check yet)` before the first check.
pub fn status(store: &Store) -> Value {
    store
        .read_state(STATUS_STATE)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or_else(|| json!({"consumers": "unknown (no check yet)", "bound": null}))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(agent: &str, instance: &str, strength: &str, why: &str) -> Value {
        json!({"agent": agent, "instance": instance, "strength": strength, "why": why})
    }

    fn v(list: Vec<Value>) -> Value {
        json!({"consumers": list, "bound": null})
    }

    #[test]
    fn only_me_proven_is_one() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "-", "proven", "")]);
        let r = verdict("fx-a", "w1", Ok(&a));
        assert_eq!(r.line, "one (proven)");
        assert!(r.others.is_empty());
    }

    #[test]
    fn a_second_identity_is_named_with_its_strength() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
        let r = verdict("fx-a", "w1", Ok(&a));
        assert_eq!(r.line, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
        assert_eq!(r.others.len(), 1);
    }

    #[test]
    fn same_agent_other_instance_is_a_second_copy() {
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "w2", "proven", "")]);
        let r = verdict("fx-a", "w1", Ok(&a));
        assert!(r.line.contains("second copy, instance w2"), "{}", r.line);
    }

    #[test]
    fn unreadable_is_unknown_never_one() {
        assert!(verdict("fx-a", "w1", Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
        assert_eq!(verdict("fx-a", "w1", Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
        assert!(verdict("fx-a", "w1", Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
    }

    #[test]
    fn my_skewed_clock_shows_as_claimed() {
        let a = v(vec![e("fx-a", "w1", "claimed", "clock skew")]);
        assert_eq!(verdict("fx-a", "w1", Ok(&a)).line, "one (claimed: clock skew)");
    }

    #[test]
    fn attention_once_per_reader_per_day() {
        let est = crate::testkit::Estate::create_without_hub().unwrap();
        let store = Store::open(&est.store_root(), "fx-a").unwrap();
        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
        let r = verdict("fx-a", "w1", Ok(&a));
        record(&store, "inbox:h/fx-a", &r, 1_000);
        record(&store, "inbox:h/fx-a", &r, 2_000);
        let n = store.attention_entries().iter().filter(|l| l.contains(ATTENTION_KIND)).count();
        assert_eq!(n, 1);
        assert_eq!(status(&store)["consumers"], r.line);
    }
}
