//! K-34 / V-21 (T-3694, B-25; 06 AL-13, ARC4 6.13.5): "role mail waiting, main not reading".
//!
//! One definition, computed per tick by whichever receiver holds the role mail (this sidecar;
//! AEF's receiver runs the same function by contract, 17.7). This module is the definition and
//! its episode state; it does not read the store or the hub itself: the caller passes the
//! waiting messages, the lease view (AL-12) and an out-of-band sink, and lands what comes back
//! (status `main.mail_waiting`, needs-attention entries; `scripts/session-start-alerts.sh`
//! already prints every needs-attention entry at session start, section 1d).
//!
//! Rules:
//! * Only role mail counts (to `main` or the project), never mail to an exact instance (FCW-64).
//! * The signal rises when some counted message has waited 10 min and nobody is a reading
//!   main: the lease is unknown, vacant, lapsed or provisional, the holder's last human prompt
//!   is unknown or older than 10 min (FCW-65), or the holder is another agent's session while
//!   the mail is held here unforwarded (FCW-204).
//! * It clears only when the old mail is handed over (`old10` empty).
//! * Out of band at 30 min, once per episode and at most once per project per hour across all
//!   receivers (the sink's per-project claim). No channel configured: `not available`; a send
//!   without the channel's own receipt: `not confirmed`. Never "sent" without a receipt (FCW-62).
//! * Within an active episode: one re-raised entry per urgent message crossing 10 min and one
//!   per new lease generation (CR-6.29). The weekly episode count is reported, never capped.

use serde_json::{Value, json};
use std::collections::BTreeSet;

/// V-21: role mail not handed over for this long raises the signal.
pub const WAIT_MS: u64 = 10 * 60_000;
/// A holder whose last human prompt is older than this is "not reading".
pub const READING_MS: u64 = 10 * 60_000;
/// Out-of-band notice at this age of the oldest waiting message.
pub const OOB_MS: u64 = 30 * 60_000;
/// The per-project out-of-band claim lasts this long: at most one notice per project per hour.
pub const OOB_CLAIM_MS: u64 = 60 * 60_000;
pub const ATTENTION_KIND: &str = "main-mail-waiting";

/// One stored message the receiver has not handed over, forwarded or refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waiting {
    pub id: String,
    /// When it was stored (the receiver's monotonic-ish clock, ms).
    pub stored_ms: u64,
    /// Addressed to a role (`main` or the project), not to an exact instance.
    pub role: bool,
    pub urgent: bool,
}

/// The lease as the receiver last read it (AL-12). `None` passed to [`Signal::tick`] = unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseFacts {
    /// `held`, `provisional`, `vacant`, `lapsed`.
    pub state: String,
    pub provisional: bool,
    pub holder_session: Option<String>,
    pub holder_agent: Option<String>,
    pub generation: u64,
    pub last_human_at_ms: Option<u64>,
    /// "Now" on the same clock as `last_human_at_ms` (the lease view's).
    pub now_ms: u64,
}

impl LeaseFacts {
    /// From the lease view the sidecar holds (`termlink_session::main_lease::View`).
    pub fn from_view(v: &termlink_session::main_lease::View) -> LeaseFacts {
        LeaseFacts {
            state: v.state.to_string(),
            provisional: v.provisional,
            holder_session: v.holder.as_ref().map(|h| h.session.clone()),
            holder_agent: v.holder.as_ref().map(|h| h.agent.clone()),
            generation: v.generation,
            last_human_at_ms: v.last_human_at_ms,
            now_ms: v.now_ms,
        }
    }

    pub fn holder_human_age_s(&self) -> Option<u64> {
        self.last_human_at_ms.map(|t| self.now_ms.saturating_sub(t) / 1000)
    }
}

/// AL-13 step 3: is there a main that reads the mail held by `my_agent`'s receiver?
pub fn reading(lease: Option<&LeaseFacts>, my_agent: &str) -> bool {
    let Some(l) = lease else { return false }; // unknown (FCW-65)
    l.state == "held"
        && !l.provisional
        && l.last_human_at_ms.is_some_and(|t| l.now_ms.saturating_sub(t) <= READING_MS)
        && l.holder_agent.as_deref() == Some(my_agent) // FCW-204
}

/// What the out-of-band step found or did (the status `oob` field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Oob {
    /// No channel configured (GP-11 not live).
    NotAvailable,
    /// Sent, but the channel gave no delivery receipt of its own.
    NotConfirmed,
    SentAt(u64),
    /// Another receiver holds the per-project claim and has recorded its outcome.
    ByOther { by: String, outcome: String, at_ms: u64 },
    /// Another receiver holds the claim and has not recorded an outcome yet.
    PendingAt(String),
    /// The sink could not be asked (claim errored): nothing was sent, nothing is claimed.
    Error(String),
}

impl Oob {
    pub fn render(&self) -> String {
        match self {
            Oob::NotAvailable => "not available".into(),
            Oob::NotConfirmed => "not confirmed".into(),
            Oob::SentAt(t) => format!("sent at {t}"),
            Oob::ByOther { by, outcome, at_ms } => format!("{outcome} (by {by} at {at_ms})"),
            Oob::PendingAt(h) => format!("pending at {h}"),
            Oob::Error(e) => format!("not sent: {e}"),
        }
    }
}

/// The per-project out-of-band channel (GP-11) and its claim. Fakes implement it in tests.
pub trait OobSink {
    /// A channel exists for this project.
    fn configured(&self) -> bool;
    /// Claim `oob:<project>` for [`OOB_CLAIM_MS`]. `Ok(None)` = won; `Ok(Some(holder))` =
    /// another receiver holds it.
    fn claim(&mut self, project: &str, me: &str) -> Result<Option<String>, String>;
    /// Send; `Some(receipt)` only with the channel's own delivery receipt.
    fn send(&mut self, project: &str, text: &str) -> Option<String>;
    /// Record this receiver's outcome on `oob:<project>`.
    fn record(&mut self, project: &str, me: &str, outcome: &str, at_ms: u64);
    /// The outcome the claim holder recorded, if any: (by, outcome, at).
    fn recorded(&self, project: &str) -> Option<(String, String, u64)>;
}

/// No out-of-band channel (the state before GP-11 is live).
pub struct NoSink;
impl OobSink for NoSink {
    fn configured(&self) -> bool {
        false
    }
    fn claim(&mut self, _: &str, _: &str) -> Result<Option<String>, String> {
        Err("no out-of-band channel".into())
    }
    fn send(&mut self, _: &str, _: &str) -> Option<String> {
        None
    }
    fn record(&mut self, _: &str, _: &str, _: &str, _: u64) {}
    fn recorded(&self, _: &str) -> Option<(String, String, u64)> {
        None
    }
}

/// One needs-attention entry: the coalesce key and the line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attention {
    pub key: String,
    pub detail: String,
}

/// What a tick produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// `main.mail_waiting` for status: the object while active, `null` otherwise.
    pub status: Value,
    pub attention: Vec<Attention>,
}

/// The episode state of one receiver. Recomputed from durable mail ages after a restart (a
/// restart may re-raise once; the coalesce key is per episode).
#[derive(Debug, Default)]
pub struct Signal {
    active: bool,
    since_ms: u64,
    episode: String,
    gen0: u64,
    reraised: BTreeSet<String>,
    oob_set_by_me: Option<Oob>,
    oob: Option<Oob>,
    /// Episodes raised (reported weekly, never capped: RNU).
    pub episodes: u64,
    told: BTreeSet<(String, u64)>,
}

impl Signal {
    pub fn new() -> Signal {
        Signal::default()
    }

    pub fn active(&self) -> bool {
        self.active
    }

    /// AL-13 `mail_waiting_tick`. `now_ms` is on the same clock as `Waiting::stored_ms`.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        project: &str,
        me: &str,
        my_agent: &str,
        now_ms: u64,
        mail: &[Waiting],
        lease: Option<&LeaseFacts>,
        sink: &mut dyn OobSink,
    ) -> Tick {
        let mut attention = vec![];
        // 1. role mail only (FCW-64).
        let w: Vec<&Waiting> = mail.iter().filter(|m| m.role).collect();
        // 2. waited 10 min.
        let old10: Vec<&&Waiting> = w.iter().filter(|m| now_ms.saturating_sub(m.stored_ms) >= WAIT_MS).collect();
        let is_reading = reading(lease, my_agent);
        let generation = lease.map(|l| l.generation).unwrap_or(0);
        // 4. raise.
        if !old10.is_empty() && !is_reading && !self.active {
            self.active = true;
            self.since_ms = now_ms;
            let oldest = old10.iter().min_by_key(|m| m.stored_ms).map(|m| m.id.clone()).unwrap_or_default();
            self.episode = format!("{project}:{oldest}");
            self.gen0 = generation;
            self.reraised.clear();
            self.oob_set_by_me = None;
            self.oob = None;
            self.episodes += 1;
            attention.push(Attention {
                key: self.episode.clone(),
                detail: format!(
                    "{ATTENTION_KIND}: {} role message(s) for {project} waiting {} min with no reading main ({})",
                    w.len(),
                    old10.iter().map(|m| now_ms.saturating_sub(m.stored_ms)).max().unwrap_or(0) / 60_000,
                    describe(lease)
                ),
            });
        }
        // 4a. re-raise within an episode (CR-6.29).
        if self.active {
            for m in w.iter().filter(|m| m.urgent && now_ms.saturating_sub(m.stored_ms) >= WAIT_MS) {
                let key = format!("{}:urgent:{}", self.episode, m.id);
                if self.reraised.insert(key.clone()) {
                    attention.push(Attention { key, detail: format!("{ATTENTION_KIND}: urgent message {} still waiting", m.id) });
                }
            }
            if lease.is_some() && generation != self.gen0 {
                self.gen0 = generation;
                let key = format!("{}:gen:{generation}", self.episode);
                if self.reraised.insert(key.clone()) {
                    attention.push(Attention { key, detail: format!("{ATTENTION_KIND}: lease moved to generation {generation}, mail still waiting") });
                }
            }
        }
        // 5. clears only when the old mail is handed over.
        if self.active && old10.is_empty() {
            self.active = false;
        }
        // 7. out of band at 30 min, once per episode (CR-6.28).
        let oldest_age = w.iter().map(|m| now_ms.saturating_sub(m.stored_ms)).max().unwrap_or(0);
        if self.active && oldest_age >= OOB_MS && self.oob_set_by_me.is_none() {
            let o = if !sink.configured() {
                Oob::NotAvailable // FCW-62
            } else {
                match sink.claim(project, me) {
                    Err(e) => Oob::Error(e),
                    Ok(Some(holder)) => match sink.recorded(project) {
                        // FCW-63: another receiver sent (or is sending) this hour's notice.
                        Some((by, outcome, at_ms)) => Oob::ByOther { by, outcome, at_ms },
                        None => Oob::PendingAt(holder),
                    },
                    Ok(None) => {
                        let r = sink.send(project, &format!("MAIL WAITING: role mail for {project} unread {} min; no reading main", oldest_age / 60_000));
                        let o = if r.is_some() { Oob::SentAt(now_ms) } else { Oob::NotConfirmed };
                        sink.record(project, me, &o.render(), now_ms);
                        o
                    }
                }
            };
            // Only an outcome this receiver produced is final for the episode; another
            // receiver's pending/recorded outcome is re-read on later ticks.
            match &o {
                Oob::ByOther { .. } | Oob::PendingAt(_) | Oob::Error(_) => self.oob = Some(o),
                _ => {
                    self.oob_set_by_me = Some(o.clone());
                    self.oob = Some(o);
                }
            }
        }
        // 6. status.
        let status = if self.active {
            json!({
                "count": w.len(),
                "oldest_age_s": oldest_age / 1000,
                "holder": lease.and_then(|l| l.holder_session.clone()),
                "holder_human_age_s": lease.and_then(LeaseFacts::holder_human_age_s),
                "lease_state": lease.map_or("unknown".to_string(), |l| l.state.clone()),
                "since_ms": self.since_ms,
                "oob": self.oob.as_ref().map(Oob::render),
                "episodes": self.episodes,
            })
        } else {
            Value::Null
        };
        Tick { status, attention }
    }

    /// The non-main line (AL-9 withhold path): once per (session, generation), only while an
    /// episode is active and only at a human prompt.
    pub fn non_main_line(&mut self, project: &str, session: &str, generation: u64, origin_human: bool) -> Option<String> {
        if !self.active || !origin_human || !self.told.insert((session.to_string(), generation)) {
            return None;
        }
        Some(format!("TermLink: role mail for {project} is waiting and no main is reading it (generation {generation}). `termlink main status` shows who holds main."))
    }
}

/// The signal as one sidecar runs it (T-3694 wiring): the daemon tick calls [`Runner::tick`]
/// once per tick with the deliverer's waiting mail and the fresh lease view; each raised entry
/// lands through `land` (the store's coalesced needs-attention list, which
/// `scripts/session-start-alerts.sh` prints at session start), and `status` reads
/// [`Runner::status_view`] as `main.mail_waiting`.
pub struct Runner {
    pub project: String,
    me: String,
    agent: String,
    inner: std::sync::Mutex<RunnerInner>,
}

struct RunnerInner {
    signal: Signal,
    sink: Box<dyn OobSink + Send>,
    status: Value,
    ticked_at_ms: Option<u64>,
    /// The last needs-attention entry that could not be written (never silent).
    land_error: Option<String>,
}

impl Runner {
    /// `me` names this receiver for the per-project out-of-band claim (its instance id).
    pub fn new(project: &str, me: &str, agent: &str, sink: Box<dyn OobSink + Send>) -> std::sync::Arc<Runner> {
        std::sync::Arc::new(Runner {
            project: project.to_string(),
            me: me.to_string(),
            agent: agent.to_string(),
            inner: std::sync::Mutex::new(RunnerInner { signal: Signal::new(), sink, status: Value::Null, ticked_at_ms: None, land_error: None }),
        })
    }

    /// One tick: compute, land every entry (coalesced by its key), keep the status.
    pub fn tick(
        &self,
        now_ms: u64,
        mail: &[Waiting],
        lease: Option<&LeaseFacts>,
        land: &mut dyn FnMut(&Attention) -> Result<bool, String>,
    ) -> Tick {
        let Ok(mut g) = self.inner.lock() else {
            return Tick { status: json!({"state": "unknown", "why": "mail_waiting lock poisoned"}), attention: vec![] };
        };
        let g = &mut *g;
        let t = g.signal.tick(&self.project, &self.me, &self.agent, now_ms, mail, lease, g.sink.as_mut());
        for a in &t.attention {
            match land(a) {
                Ok(_) => {}
                Err(e) => g.land_error = Some(format!("{}: {e}", a.key)),
            }
        }
        g.status = t.status.clone();
        g.ticked_at_ms = Some(now_ms);
        t
    }

    /// `main.mail_waiting`: the signal's object while an episode is active, `null` while none
    /// is, and `unknown` before the first tick (never `null` for "not looked").
    pub fn status_view(&self) -> Value {
        let Ok(g) = self.inner.lock() else {
            return json!({"state": "unknown", "why": "mail_waiting lock poisoned"});
        };
        if g.ticked_at_ms.is_none() {
            return json!({"state": "unknown", "why": "not computed yet"});
        }
        let mut v = g.status.clone();
        if let (Some(e), Value::Object(m)) = (&g.land_error, &mut v) {
            m.insert("attention_error".into(), Value::from(e.clone()));
        }
        v
    }

    /// The non-main line for `session` (AL-9 path), while an episode is active.
    pub fn non_main_line(&self, session: &str, generation: u64, origin_human: bool) -> Option<String> {
        self.inner.lock().ok()?.signal.non_main_line(&self.project, session, generation, origin_human)
    }
}

fn describe(lease: Option<&LeaseFacts>) -> String {
    match lease {
        None => "lease unknown".into(),
        Some(l) => match (&l.holder_session, l.holder_human_age_s()) {
            (Some(h), Some(a)) => format!("lease {} by {h}, last human prompt {}s ago", l.state, a),
            (Some(h), None) => format!("lease {} by {h}, last human prompt unknown", l.state),
            (None, _) => format!("lease {}", l.state),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: u64 = 60_000;

    fn role(id: &str, at: u64) -> Waiting {
        Waiting { id: id.into(), stored_ms: at, role: true, urgent: false }
    }
    fn lease(state: &str, agent: &str, generation: u64, human_age_ms: Option<u64>) -> LeaseFacts {
        let now = 100 * MIN;
        LeaseFacts {
            state: state.into(),
            provisional: state == "provisional",
            holder_session: (state == "held" || state == "provisional").then(|| "S-main".to_string()),
            holder_agent: (state == "held" || state == "provisional").then(|| agent.to_string()),
            generation,
            last_human_at_ms: human_age_ms.map(|a| now - a),
            now_ms: now,
        }
    }

    #[derive(Default)]
    struct Fake {
        on: bool,
        receipt: bool,
        holder: Option<String>,
        sent: Vec<String>,
        records: Vec<(String, String, u64)>,
    }
    impl OobSink for Fake {
        fn configured(&self) -> bool {
            self.on
        }
        fn claim(&mut self, _: &str, me: &str) -> Result<Option<String>, String> {
            match &self.holder {
                Some(h) if h != me => Ok(Some(h.clone())),
                _ => {
                    self.holder = Some(me.into());
                    Ok(None)
                }
            }
        }
        fn send(&mut self, _: &str, text: &str) -> Option<String> {
            self.sent.push(text.into());
            self.receipt.then(|| "rcpt-1".into())
        }
        fn record(&mut self, _: &str, me: &str, outcome: &str, at: u64) {
            self.records.push((me.into(), outcome.into(), at));
        }
        fn recorded(&self, _: &str) -> Option<(String, String, u64)> {
            self.records.last().cloned()
        }
    }

    // E-28 / E-139: nothing at 9:59, every field at 10:00, for each "no reading main" condition.
    #[test]
    fn e28_rises_at_ten_minutes_for_every_not_reading_condition() {
        let conds: Vec<(&str, Option<LeaseFacts>)> = vec![
            ("unknown", None),
            ("vacant", Some(lease("vacant", "a", 1, None))),
            ("lapsed", Some(lease("lapsed", "a", 1, None))),
            ("provisional", Some(lease("provisional", "a", 1, Some(MIN)))),
            ("holder silent 10 min", Some(lease("held", "a", 1, Some(11 * MIN)))),
            ("holder last human unknown", Some(lease("held", "a", 1, None))),
            ("holder of another agent (FCW-204)", Some(lease("held", "b", 1, Some(MIN)))),
        ];
        for (name, l) in conds {
            let mut s = Signal::new();
            let mail = [role("m1", 0)];
            let t = s.tick("p", "rx", "a", WAIT_MS - 1_000, &mail, l.as_ref(), &mut NoSink);
            assert!(t.status.is_null() && t.attention.is_empty(), "{name}: nothing at 9:59");
            let t = s.tick("p", "rx", "a", WAIT_MS, &mail, l.as_ref(), &mut NoSink);
            assert_eq!(t.attention.len(), 1, "{name}: one entry at 10:00");
            assert!(t.attention[0].detail.starts_with(ATTENTION_KIND));
            for k in ["count", "oldest_age_s", "holder", "holder_human_age_s", "since_ms"] {
                assert!(t.status.get(k).is_some(), "{name}: field {k}: {}", t.status);
            }
            assert_eq!(t.status["count"], 1);
        }
    }

    #[test]
    fn e28_a_reading_main_of_this_agent_raises_nothing() {
        let mut s = Signal::new();
        let l = lease("held", "a", 1, Some(9 * MIN));
        let t = s.tick("p", "rx", "a", 20 * MIN, &[role("m1", 0)], Some(&l), &mut NoSink);
        assert!(t.status.is_null() && t.attention.is_empty());
    }

    // E-28.5 control: exact-instance (conversation) mail never counts (FCW-64).
    #[test]
    fn e28_exact_instance_mail_never_counts() {
        let mut s = Signal::new();
        let m = Waiting { id: "x".into(), stored_ms: 0, role: false, urgent: true };
        let t = s.tick("p", "rx", "a", 60 * MIN, &[m], None, &mut NoSink);
        assert!(t.status.is_null() && t.attention.is_empty());
    }

    // E-28: OOB at 30:00 only; without a sink `not available` and never "sent"; once per episode.
    #[test]
    fn e28_oob_not_available_without_a_sink_and_never_sent() {
        let mut s = Signal::new();
        let mail = [role("m1", 0)];
        let t = s.tick("p", "rx", "a", OOB_MS - 1_000, &mail, None, &mut NoSink);
        assert!(t.status["oob"].is_null(), "no OOB before 30:00: {}", t.status);
        let t = s.tick("p", "rx", "a", OOB_MS, &mail, None, &mut NoSink);
        assert_eq!(t.status["oob"], "not available");
        assert!(!t.status.to_string().contains("sent"));
    }

    #[test]
    fn e28_oob_sent_once_per_episode_and_hour_and_never_without_a_receipt() {
        let mail = [role("m1", 0)];
        let mut sink = Fake { on: true, receipt: true, ..Fake::default() };
        let mut s = Signal::new();
        s.tick("p", "rx", "a", WAIT_MS, &mail, None, &mut sink);
        let t = s.tick("p", "rx", "a", OOB_MS, &mail, None, &mut sink);
        assert_eq!(t.status["oob"], format!("sent at {OOB_MS}"));
        for m in [31, 45, 89] {
            let t = s.tick("p", "rx", "a", m * MIN, &mail, None, &mut sink);
            assert_eq!(t.status["oob"], format!("sent at {OOB_MS}"), "never overwritten in the episode");
        }
        assert_eq!(sink.sent.len(), 1, "one notice in 90 min");
        // No receipt: "not confirmed", never "sent".
        let mut sink = Fake { on: true, receipt: false, ..Fake::default() };
        let mut s = Signal::new();
        let t = s.tick("p", "rx", "a", OOB_MS, &mail, None, &mut sink);
        assert_eq!(t.status["oob"], "not confirmed");
    }

    // E-28.6 (b): a second receiver shows the claim holder's recorded outcome, never sends.
    #[test]
    fn e28_second_receiver_shows_the_holders_outcome() {
        let mail = [role("m1", 0)];
        let mut sink = Fake { on: true, receipt: true, ..Fake::default() };
        let mut x = Signal::new();
        let mut y = Signal::new();
        x.tick("p", "rx-X", "a", OOB_MS, &mail, None, &mut sink);
        let t = y.tick("p", "rx-Y", "a", OOB_MS + 1_000, &mail, None, &mut sink);
        assert_eq!(sink.sent.len(), 1);
        assert!(t.status["oob"].as_str().unwrap().contains("by rx-X"), "{}", t.status);
        let t = x.tick("p", "rx-X", "a", OOB_MS + 2_000, &mail, None, &mut sink);
        assert_eq!(t.status["oob"], format!("sent at {OOB_MS}"), "the holder's own outcome stays");
    }

    // E-28.6 (c): urgent message crossing 10 min and a new generation re-raise once each.
    #[test]
    fn e28_reraise_once_per_urgent_message_and_per_generation() {
        let mut s = Signal::new();
        let l1 = lease("vacant", "a", 1, None);
        let mut mail = vec![role("m1", 0)];
        assert_eq!(s.tick("p", "rx", "a", WAIT_MS, &mail, Some(&l1), &mut NoSink).attention.len(), 1);
        mail.push(Waiting { id: "u1".into(), stored_ms: 5 * MIN, role: true, urgent: true });
        assert!(s.tick("p", "rx", "a", 14 * MIN, &mail, Some(&l1), &mut NoSink).attention.is_empty());
        let t = s.tick("p", "rx", "a", 15 * MIN, &mail, Some(&l1), &mut NoSink);
        assert_eq!(t.attention.len(), 1, "{t:?}");
        assert!(s.tick("p", "rx", "a", 16 * MIN, &mail, Some(&l1), &mut NoSink).attention.is_empty());
        let l2 = lease("lapsed", "a", 2, None);
        let t = s.tick("p", "rx", "a", 17 * MIN, &mail, Some(&l2), &mut NoSink);
        assert_eq!(t.attention.len(), 1);
        assert!(t.attention[0].detail.contains("generation 2"));
        assert!(s.tick("p", "rx", "a", 18 * MIN, &mail, Some(&l2), &mut NoSink).attention.is_empty());
    }

    // E-28: clears at hand-over; a later wait is a new episode (counted, never capped).
    #[test]
    fn e28_clears_on_hand_over_and_counts_episodes() {
        let mut s = Signal::new();
        assert!(!s.tick("p", "rx", "a", WAIT_MS, &[role("m1", 0)], None, &mut NoSink).status.is_null());
        assert!(s.tick("p", "rx", "a", WAIT_MS + 1, &[], None, &mut NoSink).status.is_null(), "cleared at hand-over");
        let t = s.tick("p", "rx", "a", 3 * WAIT_MS, &[role("m2", WAIT_MS + 5)], None, &mut NoSink);
        assert_eq!(t.status["episodes"], 2);
    }

    // E-28: exactly one non-main line per (session, generation), only at a human prompt.
    #[test]
    fn e28_one_non_main_line_per_generation() {
        let mut s = Signal::new();
        assert!(s.non_main_line("p", "S2", 1, true).is_none(), "no line while inactive");
        s.tick("p", "rx", "a", WAIT_MS, &[role("m1", 0)], None, &mut NoSink);
        assert!(s.non_main_line("p", "S2", 1, false).is_none(), "not at an automated prompt");
        assert!(s.non_main_line("p", "S2", 1, true).is_some());
        assert!(s.non_main_line("p", "S2", 1, true).is_none());
        assert!(s.non_main_line("p", "S2", 2, true).is_some());
    }

    // K-34 wiring (T-3694): the runner is unknown before its first tick, null while nothing
    // waits, lands each raised entry once (the land function coalesces by key), and a land
    // failure is shown in the status, never dropped.
    #[test]
    fn e28_runner_lands_each_entry_once_and_never_hides_a_failed_land() {
        let r = Runner::new("p", "rx", "a", Box::new(NoSink));
        assert_eq!(r.status_view()["state"], "unknown", "not computed yet is never null");
        let mut landed: Vec<String> = vec![];
        let mut land = |a: &Attention| -> Result<bool, String> {
            if landed.contains(&a.key) {
                return Ok(false);
            }
            landed.push(a.key.clone());
            Ok(true)
        };
        r.tick(WAIT_MS - 1, &[role("m1", 0)], None, &mut land);
        assert!(r.status_view().is_null(), "nothing at 9:59");
        r.tick(WAIT_MS, &[role("m1", 0)], None, &mut land);
        r.tick(WAIT_MS + 1, &[role("m1", 0)], None, &mut land);
        let v = r.status_view();
        assert_eq!(v["count"], 1, "{v}");
        assert_eq!(v["lease_state"], "unknown");
        assert_eq!(landed.len(), 1, "one entry per episode: {landed:?}");
        let r2 = Runner::new("p", "rx", "a", Box::new(NoSink));
        r2.tick(WAIT_MS, &[role("m1", 0)], None, &mut |_| Err("disk full".into()));
        assert!(r2.status_view()["attention_error"].as_str().is_some_and(|e| e.contains("disk full")), "{}", r2.status_view());
    }
}
