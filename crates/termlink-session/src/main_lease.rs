//! T-3654 (OD-18 / R-67, R-67.a1, R-68): the project's "main" lease at the home hub.
//!
//! One SESSION per project is main. The lease is an ordinary hub claim (T-2029..T-2101) on
//! offset 0 of the topic `main:<project>`; nothing new is added to the hub protocol. The same
//! topic's log is the durable record: every acquire, release, observed lapse and operator pin
//! is one signed `main.lease.v1` post, so the generation (the highest acquire generation in
//! the log) is never reused and every change and override is auditable by reading the topic.
//!
//! Rules (R-67.a / R-67.a1 / R-67.e):
//! * The holder renews; a busy holder keeps main by renewing. Nobody but the holder renews,
//!   in particular never its sidecar (R-67.c: a sidecar-renewed lease proves a process, not a
//!   turn). A holder that stops renewing lapses at `claimed_until`.
//! * Another session takes over only after lapse + quiet period, and not before the cooldown
//!   since the last generation change has passed. A clean release skips the quiet period.
//!   There is no automatic take-back: a holder that lost main finds it held and is told.
//! * The lapsed holder itself may resume (same generation) while nobody else holds it.
//! * Operator pin first: a pinned session takes main at once, even from a live holder, and the
//!   override is written to the log (`reason: pin-override`, `from: <old holder>`).
//! * Duplicate holders at one generation (only possible when the log was restored or written
//!   by two partitioned hubs) are resolved by pin, else the longest uninterrupted holder, else
//!   the stable id (session id order). With one hub the claim itself is the mutual exclusion.
//! * Fail-closed: any hub error is `unknown`, never `main`.
//!
//! The timing constants are conservative defaults (see T-3654 Decisions); the only way to
//! change them is the `TERMLINK_MAIN_TEST_*` seams, for tests.

use std::collections::BTreeMap;
use std::time::Duration;

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use termlink_protocol::control::{error_code, method};
use termlink_protocol::jsonrpc::RpcResponse;
use termlink_protocol::transport::TransportAddr;

use crate::agent_identity::Identity;
use crate::claim_client::{self, ClaimError};
use crate::client::{Client, ClientError};

/// `msg_type` of every lease record on the `main:<project>` topic.
pub const MSG_TYPE: &str = "main.lease.v1";
/// Lease length per claim/renew: 15 minutes.
pub const LEASE_MS: u64 = 15 * 60_000;
/// Quiet period after a lapse before another session may take over: 5 minutes.
pub const QUIET_MS: u64 = 5 * 60_000;
/// Minimum time between two generation changes: 10 minutes.
pub const COOLDOWN_MS: u64 = 10 * 60_000;
/// Retention of the lease topic: the newest 1000 records (records are rare: one per change).
pub const KEEP: u64 = 1000;
/// Bound on each hub exchange.
pub const RPC_TIMEOUT: Duration = Duration::from_secs(10);

/// The lease topic of a project at its home hub.
pub fn topic(project: &str) -> String {
    format!("main:{project}")
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Lease timing. `Timing::from_env()` is the production entry point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Timing {
    pub lease_ms: u64,
    pub quiet_ms: u64,
    pub cooldown_ms: u64,
}

impl Default for Timing {
    fn default() -> Self {
        Timing { lease_ms: LEASE_MS, quiet_ms: QUIET_MS, cooldown_ms: COOLDOWN_MS }
    }
}

impl Timing {
    /// Defaults, overridden only by the test seams `TERMLINK_MAIN_TEST_LEASE_MS`,
    /// `TERMLINK_MAIN_TEST_QUIET_MS`, `TERMLINK_MAIN_TEST_COOLDOWN_MS`. The lease floor is the
    /// hub's 1000 ms claim minimum.
    pub fn from_env() -> Self {
        let num = |k: &str| std::env::var(k).ok().and_then(|v| v.trim().parse::<u64>().ok());
        let d = Timing::default();
        Timing {
            lease_ms: num("TERMLINK_MAIN_TEST_LEASE_MS").unwrap_or(d.lease_ms).clamp(1_000, 60 * 60_000),
            quiet_ms: num("TERMLINK_MAIN_TEST_QUIET_MS").unwrap_or(d.quiet_ms),
            cooldown_ms: num("TERMLINK_MAIN_TEST_COOLDOWN_MS").unwrap_or(d.cooldown_ms),
        }
    }
}

/// The session holding (or asking for) main.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holder {
    pub session: String,
    pub agent: String,
    pub host: String,
    pub pid: u32,
}

fn clean(s: &str) -> String {
    s.chars().map(|c| if matches!(c, ';' | '=') || c.is_control() { '_' } else { c }).collect()
}

impl Holder {
    /// The hub claim's `claimer` string: `main;s=<session>;a=<agent>;h=<host>;p=<pid>`.
    pub fn claimer(&self) -> String {
        format!("main;s={};a={};h={};p={}", clean(&self.session), clean(&self.agent), clean(&self.host), self.pid)
    }

    /// Parse a `claimer`; any other string (a raw claim on offset 0) becomes a holder whose
    /// session is the whole string, so it is still named and never mistaken for anyone else.
    pub fn from_claimer(c: &str) -> Holder {
        let mut h = Holder { session: c.to_string(), agent: String::new(), host: String::new(), pid: 0 };
        let Some(rest) = c.strip_prefix("main;") else { return h };
        for kv in rest.split(';') {
            match kv.split_once('=') {
                Some(("s", v)) => h.session = v.to_string(),
                Some(("a", v)) => h.agent = v.to_string(),
                Some(("h", v)) => h.host = v.to_string(),
                Some(("p", v)) => h.pid = v.parse().unwrap_or(0),
                _ => {}
            }
        }
        h
    }

    pub fn display(&self) -> String {
        let mut s = self.session.clone();
        if !self.host.is_empty() {
            s.push_str(&format!("@{}", self.host));
        }
        if self.pid != 0 {
            s.push_str(&format!(" pid {}", self.pid));
        }
        if !self.agent.is_empty() {
            s.push_str(&format!(" (agent {})", self.agent));
        }
        s
    }
}

/// One record on the lease topic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    Genesis {
        project: String,
        at_ms: u64,
    },
    Acquire {
        generation: u64,
        holder: Holder,
        claim_id: String,
        at_ms: u64,
        /// `first`, `takeover-after-lapse`, `after-release`, `resume`, `pin-override`, `pin`.
        reason: String,
        #[serde(default)]
        from: Option<Holder>,
        #[serde(default)]
        lapsed_at_ms: Option<u64>,
        /// Open claims the previous holder left on the obligation topics: uncertain, for the operator.
        #[serde(default)]
        obligations: Vec<Value>,
    },
    Release {
        generation: u64,
        holder: Holder,
        at_ms: u64,
    },
    Lapse {
        generation: u64,
        holder: Holder,
        lapsed_at_ms: u64,
        observed_at_ms: u64,
        observed_by: String,
    },
    Pin {
        /// `None` clears the pin.
        session: Option<String>,
        by: String,
        reason: String,
        at_ms: u64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logged {
    pub offset: u64,
    pub event: Event,
}

/// The newest acquire that counts.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Current {
    pub generation: u64,
    pub holder: Holder,
    pub claim_id: String,
    /// Start of the holder's uninterrupted holding (a resume after a lapse restarts it).
    pub since_ms: u64,
    pub at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Ended {
    pub at_ms: u64,
    /// A release (true) or a lapse (false).
    pub clean: bool,
}

/// A duplicate-holder candidate (R-67.a1 (3)).
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub holder: Holder,
    pub since_ms: u64,
}

/// R-67.a1 (3) duplicate order: operator pin, else the longest uninterrupted holder, else the
/// stable id (lowest session id). `None` only for an empty list.
pub fn resolve_duplicates<'a>(cands: &'a [Candidate], pin: Option<&str>) -> Option<&'a Candidate> {
    if let Some(p) = pin
        && let Some(c) = cands.iter().find(|c| c.holder.session == p)
    {
        return Some(c);
    }
    cands.iter().min_by(|a, b| a.since_ms.cmp(&b.since_ms).then_with(|| a.holder.session.cmp(&b.holder.session)))
}

/// The log, folded.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Folded {
    pub generation: u64,
    pub current: Option<Current>,
    pub ended: Option<Ended>,
    pub last_change_ms: Option<u64>,
    pub pin: Option<String>,
    pub pin_record: Option<Value>,
    /// Session -> (generation it lost at, who took it, when): "the old holder is told".
    pub losses: BTreeMap<String, (u64, Holder, u64)>,
    /// Set when two holders were recorded at one generation and the order resolved it.
    pub duplicate: Option<Value>,
    /// The newest records, for status (oldest first).
    pub recent: Vec<Value>,
    pub records: usize,
}

const RECENT: usize = 8;

pub fn fold(log: &[Logged]) -> Folded {
    let mut f = Folded::default();
    let mut same_gen: Vec<Candidate> = vec![];
    for l in log {
        f.records += 1;
        f.recent.push(json!({"offset": l.offset, "event": l.event}));
        if f.recent.len() > RECENT {
            f.recent.remove(0);
        }
        match &l.event {
            Event::Genesis { .. } => {}
            Event::Acquire { generation, holder, claim_id, at_ms, from, .. } => {
                let resume = f.current.as_ref().is_some_and(|c| c.generation == *generation && c.holder.session == holder.session);
                if *generation > f.generation || f.current.is_none() {
                    if let Some(prev) = from.as_ref().or(f.current.as_ref().map(|c| &c.holder))
                        && prev.session != holder.session
                    {
                        f.losses.insert(prev.session.clone(), (*generation, holder.clone(), *at_ms));
                    }
                    f.generation = (*generation).max(f.generation);
                    f.current = Some(Current { generation: *generation, holder: holder.clone(), claim_id: claim_id.clone(), since_ms: *at_ms, at_ms: *at_ms });
                    f.ended = None;
                    f.last_change_ms = Some(*at_ms);
                    same_gen = vec![Candidate { holder: holder.clone(), since_ms: *at_ms }];
                } else if resume {
                    let c = f.current.as_mut().expect("resume implies current");
                    if f.ended.is_some() {
                        c.since_ms = *at_ms;
                    }
                    c.claim_id = claim_id.clone();
                    c.at_ms = *at_ms;
                    f.ended = None;
                } else if *generation == f.generation {
                    // Two holders at one generation: a restored or partitioned log.
                    same_gen.push(Candidate { holder: holder.clone(), since_ms: *at_ms });
                    let win = resolve_duplicates(&same_gen, f.pin.as_deref()).cloned().expect("non-empty");
                    f.duplicate = Some(json!({
                        "generation": generation,
                        "candidates": same_gen.iter().map(|c| json!({"session": c.holder.session, "since_ms": c.since_ms})).collect::<Vec<_>>(),
                        "winner": win.holder.session,
                        "order": "operator pin, else longest uninterrupted holder, else stable id",
                    }));
                    if f.current.as_ref().map(|c| &c.holder.session) != Some(&win.holder.session) {
                        f.current = Some(Current { generation: *generation, holder: win.holder.clone(), claim_id: claim_id.clone(), since_ms: win.since_ms, at_ms: *at_ms });
                        f.ended = None;
                    }
                }
                // A lower generation is stale and ignored (it stays visible in `recent`).
            }
            Event::Release { generation, holder, at_ms } => {
                if f.current.as_ref().is_some_and(|c| c.generation == *generation && c.holder.session == holder.session) {
                    f.ended = Some(Ended { at_ms: *at_ms, clean: true });
                }
            }
            Event::Lapse { generation, holder, lapsed_at_ms, .. } => {
                if f.ended.is_none() && f.current.as_ref().is_some_and(|c| c.generation == *generation && c.holder.session == holder.session) {
                    f.ended = Some(Ended { at_ms: *lapsed_at_ms, clean: false });
                }
            }
            Event::Pin { session, by, reason, at_ms } => {
                f.pin = session.clone();
                f.pin_record = Some(json!({"session": session, "by": by, "reason": reason, "at_ms": at_ms, "offset": l.offset}));
            }
        }
    }
    f
}

/// The hub's claim row on offset 0 (the newest one, active or lapsed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub claim_id: String,
    pub claimer: String,
    pub claimed_at: i64,
    pub claimed_until: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Main,
    NotMain,
    Unknown,
}

impl Verdict {
    /// 0 main, 1 not-main, 2 unknown.
    pub fn exit_code(self) -> i32 {
        match self {
            Verdict::Main => 0,
            Verdict::NotMain => 1,
            Verdict::Unknown => 2,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Main => "main",
            Verdict::NotMain => "not-main",
            Verdict::Unknown => "unknown",
        }
    }
}

/// Who holds main right now, from the log and the hub's claim row.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct View {
    pub project: String,
    pub topic: String,
    pub now_ms: u64,
    /// The live holder, or `None` when vacant.
    pub holder: Option<Holder>,
    pub claim_id: Option<String>,
    pub claimed_until_ms: Option<u64>,
    pub generation: u64,
    pub since_ms: Option<u64>,
    pub pinned: Option<String>,
    /// When vacant: when it became vacant (`None` = never held, or not yet observed).
    pub vacant_since_ms: Option<u64>,
    /// When vacant: earliest time another session may take it (`None` = lapse not yet recorded).
    pub eligible_at_ms: Option<u64>,
    /// The live claim matches the newest acquire record.
    pub consistent: bool,
    pub ever_held: bool,
    pub previous: Option<Holder>,
    pub timing: Timing,
}

pub fn view(project: &str, f: &Folded, row: Option<&Row>, now: u64, t: Timing) -> View {
    let active = row.filter(|r| r.claimed_until > now as i64);
    let mut v = View {
        project: project.to_string(),
        topic: topic(project),
        now_ms: now,
        holder: None,
        claim_id: None,
        claimed_until_ms: None,
        generation: f.generation,
        since_ms: None,
        pinned: f.pin.clone(),
        vacant_since_ms: None,
        eligible_at_ms: None,
        consistent: true,
        ever_held: f.current.is_some(),
        previous: f.current.as_ref().map(|c| c.holder.clone()),
        timing: t,
    };
    if let Some(r) = active {
        let h = Holder::from_claimer(&r.claimer);
        v.consistent = f.current.as_ref().is_some_and(|c| c.claim_id == r.claim_id && c.holder.session == h.session);
        v.since_ms = f.current.as_ref().filter(|c| c.holder.session == h.session).map(|c| c.since_ms).or(Some(r.claimed_at as u64));
        v.claim_id = Some(r.claim_id.clone());
        v.claimed_until_ms = Some(r.claimed_until as u64);
        v.holder = Some(h);
        v.previous = None;
        return v;
    }
    let Some(cur) = &f.current else {
        // Never held: the first holder is whoever claims first (or the operator's pin).
        v.eligible_at_ms = Some(now);
        return v;
    };
    // Vacant after a holding: when did it end? The lapsed row, else the recorded release/lapse.
    let lapsed_row = row.filter(|r| r.claim_id == cur.claim_id).map(|r| Ended { at_ms: r.claimed_until.max(0) as u64, clean: false });
    let ended = f.ended.or(lapsed_row);
    if let Some(e) = ended {
        v.vacant_since_ms = Some(e.at_ms);
        let after_end = e.at_ms + if e.clean { 0 } else { t.quiet_ms };
        let after_change = f.last_change_ms.unwrap_or(0) + t.cooldown_ms;
        v.eligible_at_ms = Some(after_end.max(after_change));
    }
    v
}

/// What `claim` does next.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    Renew { claim_id: String },
    PinOverride { claim_id: String, from: Holder },
    Take { reason: &'static str },
    Resume,
    /// Vacant with no record of when the holder lapsed: record the lapse now, then wait.
    RecordLapse,
    Wait { eligible_at_ms: u64 },
    Held,
}

pub fn plan_claim(v: &View, f: &Folded, session: &str) -> Plan {
    if let Some(h) = &v.holder {
        if h.session == session {
            return Plan::Renew { claim_id: v.claim_id.clone().unwrap_or_default() };
        }
        if v.pinned.as_deref() == Some(session) {
            return Plan::PinOverride { claim_id: v.claim_id.clone().unwrap_or_default(), from: h.clone() };
        }
        return Plan::Held;
    }
    if v.pinned.as_deref() == Some(session) {
        return Plan::Take { reason: "pin" };
    }
    match &f.current {
        None => Plan::Take { reason: "first" },
        Some(c) if c.holder.session == session => Plan::Resume,
        Some(_) => match v.eligible_at_ms {
            None => Plan::RecordLapse,
            Some(t) if v.now_ms < t => Plan::Wait { eligible_at_ms: t },
            Some(_) => Plan::Take { reason: if f.ended.is_some_and(|e| e.clean) { "after-release" } else { "takeover-after-lapse" } },
        },
    }
}

/// The answer every verb prints.
#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub verdict: Verdict,
    pub session: Option<String>,
    /// What this call did (`renewed`, `acquired`, `released`, `pinned`, `none`, ...).
    pub action: String,
    pub message: String,
    pub view: Option<View>,
    /// For a session that lost main: the generation it lost at and to whom.
    pub lost: Option<Value>,
    pub pin_record: Option<Value>,
    pub duplicate: Option<Value>,
    pub recent: Vec<Value>,
}

impl Outcome {
    pub fn unknown(session: Option<&str>, why: impl Into<String>) -> Outcome {
        Outcome {
            verdict: Verdict::Unknown,
            session: session.map(str::to_string),
            action: "none".into(),
            message: why.into(),
            view: None,
            lost: None,
            pin_record: None,
            duplicate: None,
            recent: vec![],
        }
    }

    pub fn to_json(&self) -> Value {
        let v = self.view.as_ref();
        json!({
            "verdict": self.verdict.as_str(),
            "session": self.session,
            "action": self.action,
            "message": self.message,
            "project": v.map(|v| &v.project),
            "topic": v.map(|v| &v.topic),
            "holder": v.and_then(|v| v.holder.as_ref()),
            "generation": v.map(|v| v.generation),
            "since_ms": v.and_then(|v| v.since_ms),
            "claimed_until_ms": v.and_then(|v| v.claimed_until_ms),
            "pinned": v.and_then(|v| v.pinned.as_ref()),
            "vacant_since_ms": v.and_then(|v| v.vacant_since_ms),
            "eligible_at_ms": v.and_then(|v| v.eligible_at_ms),
            "previous": v.and_then(|v| v.previous.as_ref()),
            "consistent": v.map(|v| v.consistent),
            "timing": v.map(|v| v.timing),
            "lost": self.lost,
            "pin_record": self.pin_record,
            "duplicate": self.duplicate,
            "recent": self.recent,
        })
    }

    /// One line for hooks and humans.
    pub fn line(&self) -> String {
        format!("{}: {}", self.verdict.as_str(), self.message)
    }
}

fn outcome(verdict: Verdict, session: Option<&str>, action: &str, f: &Folded, v: View) -> Outcome {
    let message = describe(verdict, session, &v);
    let lost = session.and_then(|s| f.losses.get(s)).filter(|_| verdict != Verdict::Main).map(|(g, to, at)| {
        json!({"generation": g, "to": to, "at_ms": at})
    });
    Outcome {
        verdict,
        session: session.map(str::to_string),
        action: action.to_string(),
        message,
        view: Some(v),
        lost,
        pin_record: f.pin_record.clone(),
        duplicate: f.duplicate.clone(),
        recent: f.recent.clone(),
    }
}

fn describe(verdict: Verdict, session: Option<&str>, v: &View) -> String {
    match (&v.holder, verdict) {
        (Some(_), Verdict::Main) => format!("this session is main of {} (generation {})", v.project, v.generation),
        (Some(h), _) if session.is_none() => format!("main of {} is {} (generation {})", v.project, h.display(), v.generation),
        (Some(h), _) => format!("held by main {} (generation {})", h.display(), v.generation),
        (None, _) => {
            let when = match v.eligible_at_ms {
                Some(t) if t > v.now_ms => format!("; takeover eligible in {}s", (t - v.now_ms).div_ceil(1000)),
                Some(_) => "; takeover eligible now".into(),
                None => "; lapse not yet recorded".into(),
            };
            let prev = v.previous.as_ref().map(|p| format!(", last held by {}", p.display())).unwrap_or_default();
            format!("main of {} is vacant (generation {}{prev}){when}", v.project, v.generation)
        }
    }
}

/// Classify a hub claim row on offset 0 for one session (for the sidecar and `am-i`).
pub fn am_i(v: &View, session: &str) -> Verdict {
    match &v.holder {
        Some(h) if h.session == session => Verdict::Main,
        _ => Verdict::NotMain,
    }
}

// ---------------------------------------------------------------- hub I/O

#[derive(Debug, thiserror::Error)]
pub enum LeaseError {
    #[error("hub unreachable: {0}")]
    Transport(String),
    #[error("hub error {code}: {message}")]
    Hub { code: i64, message: String },
    #[error("{0}")]
    Other(String),
}

impl From<ClientError> for LeaseError {
    fn from(e: ClientError) -> Self {
        LeaseError::Transport(e.to_string())
    }
}

impl From<ClaimError> for LeaseError {
    fn from(e: ClaimError) -> Self {
        match e {
            ClaimError::Transport(t) => LeaseError::Transport(t.to_string()),
            ClaimError::Hub { code, message } => LeaseError::Hub { code, message },
            other => LeaseError::Other(other.to_string()),
        }
    }
}

/// A connection target plus what is needed to write records.
pub struct LeaseClient {
    pub addr: TransportAddr,
    /// `hub.auth` token for a TCP hub; `None` on the local unix socket.
    pub auth: Option<String>,
    /// Signing identity for lease records; read-only verbs work without one.
    pub identity: Option<Identity>,
    pub timing: Timing,
    /// Topics whose open claims by a lapsed holder are recorded as uncertain obligations.
    pub obligation_topics: Vec<String>,
}

impl LeaseClient {
    async fn call(&self, m: &str, params: Value) -> Result<Value, LeaseError> {
        let fut = async {
            let mut c = Client::connect_addr(&self.addr).await?;
            if let Some(tok) = &self.auth {
                let r = c.call(method::HUB_AUTH, json!("main-auth"), json!({"token": tok})).await?;
                if let RpcResponse::Error(e) = r {
                    return Ok::<_, ClientError>(RpcResponse::Error(e));
                }
            }
            c.call(m, json!("main-1"), params).await
        };
        let resp = tokio::time::timeout(RPC_TIMEOUT, fut)
            .await
            .map_err(|_| LeaseError::Transport(format!("{m}: no answer within {}s", RPC_TIMEOUT.as_secs())))??;
        match resp {
            RpcResponse::Success(s) => Ok(s.result),
            RpcResponse::Error(e) => Err(LeaseError::Hub { code: e.error.code, message: e.error.message }),
        }
    }

    /// The log, in offset order. An unknown topic is an empty log.
    pub async fn read_log(&self, project: &str) -> Result<(bool, Vec<Logged>), LeaseError> {
        let t = topic(project);
        let mut out = vec![];
        let mut cursor = 0u64;
        for _ in 0..64 {
            let r = match self.call(method::CHANNEL_SUBSCRIBE, json!({"topic": t, "cursor": cursor, "limit": 1000})).await {
                Ok(r) => r,
                Err(LeaseError::Hub { code, .. }) if code == error_code::CHANNEL_TOPIC_UNKNOWN => return Ok((false, vec![])),
                Err(e) => return Err(e),
            };
            let msgs = r.get("messages").and_then(Value::as_array).cloned().unwrap_or_default();
            for m in &msgs {
                if m.get("msg_type").and_then(Value::as_str) != Some(MSG_TYPE) {
                    continue;
                }
                let Some(off) = m.get("offset").and_then(Value::as_u64) else { continue };
                let Some(b64) = m.get("payload_b64").and_then(Value::as_str) else { continue };
                let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) else { continue };
                if let Ok(event) = serde_json::from_slice::<Event>(&bytes) {
                    out.push(Logged { offset: off, event });
                }
            }
            let next = r.get("next_cursor").and_then(Value::as_u64).unwrap_or(cursor);
            if msgs.is_empty() || next <= cursor {
                break;
            }
            cursor = next;
        }
        Ok((true, out))
    }

    /// The newest claim row on offset 0, active or lapsed.
    pub async fn read_row(&self, project: &str) -> Result<Option<Row>, LeaseError> {
        let rows = match claim_client::channel_claims_authed(&self.addr, self.auth.as_deref(), &topic(project), true).await {
            Ok(r) => r,
            Err(ClaimError::Hub { code, .. }) if code == error_code::CHANNEL_TOPIC_UNKNOWN => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        Ok(rows
            .into_iter()
            .filter(|r| r.offset == 0)
            .max_by_key(|r| r.claimed_until)
            .map(|r| Row { claim_id: r.claim_id, claimer: r.claimer, claimed_at: r.claimed_at, claimed_until: r.claimed_until }))
    }

    pub async fn snapshot(&self, project: &str) -> Result<(Folded, Option<Row>, View), LeaseError> {
        let (_, log) = self.read_log(project).await?;
        let row = self.read_row(project).await?;
        let f = fold(&log);
        let v = view(project, &f, row.as_ref(), now_ms(), self.timing);
        Ok((f, row, v))
    }

    async fn post(&self, project: &str, ev: &Event) -> Result<u64, LeaseError> {
        let id = self.identity.as_ref().ok_or_else(|| LeaseError::Other("no signing identity to write the lease record".into()))?;
        let t = topic(project);
        let payload = serde_json::to_vec(ev).map_err(|e| LeaseError::Other(e.to_string()))?;
        let ts = now_ms() as i64;
        let bytes = termlink_protocol::control::channel::canonical_sign_bytes(&t, MSG_TYPE, &payload, None, ts);
        let r = self
            .call(
                method::CHANNEL_POST,
                json!({
                    "topic": t,
                    "msg_type": MSG_TYPE,
                    "payload_b64": base64::engine::general_purpose::STANDARD.encode(&payload),
                    "ts": ts,
                    "sender_id": id.fingerprint(),
                    "sender_pubkey_hex": id.public_key_hex(),
                    "signature_hex": id.sign_hex(&bytes),
                }),
            )
            .await?;
        Ok(r.get("offset").and_then(Value::as_u64).unwrap_or(0))
    }

    /// Create the topic (newest-1000 retention) and its genesis record at offset 0, once.
    pub async fn ensure_topic(&self, project: &str) -> Result<(), LeaseError> {
        let (exists, log) = self.read_log(project).await?;
        if exists && !log.is_empty() {
            return Ok(());
        }
        self.call(
            method::CHANNEL_CREATE,
            json!({"name": topic(project), "retention": {"kind": "messages", "value": KEEP}, "reason": "T-3654 main lease (OD-18 / R-67)"}),
        )
        .await?;
        // A concurrent creator may have posted genesis already; a second one is harmless
        // (the lease is a claim on offset 0 whatever record sits there).
        let (_, log) = self.read_log(project).await?;
        if log.is_empty() {
            self.post(project, &Event::Genesis { project: project.to_string(), at_ms: now_ms() }).await?;
        }
        Ok(())
    }

    async fn obligations_of(&self, prev: &Holder) -> Vec<Value> {
        let mut out = vec![];
        for t in &self.obligation_topics {
            let Ok(rows) = claim_client::channel_claims_authed(&self.addr, self.auth.as_deref(), t, true).await else {
                out.push(json!({"topic": t, "class": "uncertain", "error": "could not read the topic's claims"}));
                continue;
            };
            for r in rows {
                let c = &r.claimer;
                if c == &prev.session || (!prev.agent.is_empty() && c == &prev.agent) || c == &prev.claimer() || c.contains(&prev.session) {
                    out.push(json!({
                        "topic": t, "offset": r.offset, "claimer": c, "claim_id": r.claim_id,
                        "claimed_until": r.claimed_until, "class": "uncertain", "to": "operator",
                    }));
                }
            }
        }
        out
    }

    /// Acquire or renew main for `me`, by the R-67 rules.
    pub async fn claim(&self, project: &str, me: &Holder) -> Outcome {
        match self.claim_inner(project, me).await {
            Ok(o) => o,
            Err(e) => Outcome::unknown(Some(&me.session), format!("could not read or write the lease: {e}")),
        }
    }

    async fn claim_inner(&self, project: &str, me: &Holder) -> Result<Outcome, LeaseError> {
        self.ensure_topic(project).await?;
        let t = topic(project);
        let claimer = me.claimer();
        let ttl = self.timing.lease_ms as u32;
        for _ in 0..3 {
            let (f, _row, v) = self.snapshot(project).await?;
            let take = |reason: &'static str, from: Option<Holder>, generation: u64, lapsed: Option<u64>| {
                let me = me.clone();
                let t = t.clone();
                let claimer = claimer.clone();
                async move {
                    let c = match claim_client::channel_claim_authed(&self.addr, self.auth.as_deref(), &t, 0, &claimer, ttl).await {
                        Ok(c) => c,
                        Err(ClaimError::Conflict { .. }) => return Ok(None),
                        Err(e) => return Err(LeaseError::from(e)),
                    };
                    let obligations = match &from {
                        Some(p) if reason != "resume" => self.obligations_of(p).await,
                        _ => vec![],
                    };
                    let ev = Event::Acquire {
                        generation,
                        holder: me.clone(),
                        claim_id: c.claim_id.clone(),
                        at_ms: now_ms(),
                        reason: reason.to_string(),
                        from,
                        lapsed_at_ms: lapsed,
                        obligations,
                    };
                    if let Err(e) = self.post(project, &ev).await {
                        // Never hold an unrecorded lease: give it back and report unknown.
                        let _ = claim_client::channel_release_authed(&self.addr, self.auth.as_deref(), &c.claim_id, &claimer, false).await;
                        return Err(e);
                    }
                    Ok(Some(()))
                }
            };
            let action = match plan_claim(&v, &f, &me.session) {
                Plan::Renew { claim_id } => {
                    match claim_client::channel_renew_authed(&self.addr, self.auth.as_deref(), &claim_id, &claimer, ttl).await {
                        Ok(_) => {
                            // Self-heal a claim whose acquire record is missing (crash between claim and post).
                            if !v.consistent {
                                let ev = Event::Acquire {
                                    generation: f.generation + 1,
                                    holder: me.clone(),
                                    claim_id: claim_id.clone(),
                                    at_ms: now_ms(),
                                    reason: "record-repair".into(),
                                    from: f.current.as_ref().map(|c| c.holder.clone()).filter(|h| h.session != me.session),
                                    lapsed_at_ms: None,
                                    obligations: vec![],
                                };
                                self.post(project, &ev).await?;
                            }
                            "renewed"
                        }
                        Err(ClaimError::Expired { .. }) | Err(ClaimError::NotFound { .. }) => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                Plan::PinOverride { claim_id, from } => {
                    match claim_client::channel_force_release_authed(
                        &self.addr,
                        self.auth.as_deref(),
                        &claim_id,
                        Some(&format!("T-3654 operator pin names {}", me.session)),
                    )
                    .await
                    {
                        Ok(_) | Err(ClaimError::NotFound { .. }) => {}
                        Err(e) => return Err(e.into()),
                    }
                    match take("pin-override", Some(from), f.generation + 1, None).await? {
                        Some(()) => "acquired",
                        None => continue,
                    }
                }
                Plan::Take { reason } => {
                    let from = f.current.as_ref().map(|c| c.holder.clone());
                    match take(reason, from, f.generation + 1, v.vacant_since_ms).await? {
                        Some(()) => "acquired",
                        None => continue,
                    }
                }
                Plan::Resume => match take("resume", None, f.generation.max(1), v.vacant_since_ms).await? {
                    Some(()) => "resumed",
                    None => continue,
                },
                Plan::RecordLapse => {
                    let cur = f.current.clone().expect("RecordLapse implies a previous holder");
                    let now = now_ms();
                    self.post(
                        project,
                        &Event::Lapse { generation: cur.generation, holder: cur.holder, lapsed_at_ms: now, observed_at_ms: now, observed_by: me.session.clone() },
                    )
                    .await?;
                    let (f, _, v) = self.snapshot(project).await?;
                    return Ok(outcome(Verdict::NotMain, Some(&me.session), "lapse-recorded", &f, v));
                }
                Plan::Wait { .. } | Plan::Held => {
                    return Ok(outcome(Verdict::NotMain, Some(&me.session), "none", &f, v));
                }
            };
            let (f, _, v) = self.snapshot(project).await?;
            let verdict = am_i(&v, &me.session);
            return Ok(outcome(verdict, Some(&me.session), action, &f, v));
        }
        let (f, _, v) = self.snapshot(project).await?;
        let verdict = am_i(&v, &me.session);
        Ok(outcome(verdict, Some(&me.session), "contended", &f, v))
    }

    /// Renew only: never acquires a lease this session did not hold (a lapsed own lease resumes).
    pub async fn renew(&self, project: &str, me: &Holder) -> Outcome {
        let r = async {
            let (f, _, v) = self.snapshot(project).await?;
            let mine = v.holder.as_ref().is_some_and(|h| h.session == me.session)
                || (v.holder.is_none() && f.current.as_ref().is_some_and(|c| c.holder.session == me.session));
            if !mine {
                return Ok::<_, LeaseError>(outcome(Verdict::NotMain, Some(&me.session), "none", &f, v));
            }
            Ok(self.claim(project, me).await)
        }
        .await;
        r.unwrap_or_else(|e| Outcome::unknown(Some(&me.session), format!("could not read the lease: {e}")))
    }

    /// Release on a clean shutdown.
    pub async fn release(&self, project: &str, me: &Holder) -> Outcome {
        let r = async {
            let (f, _, v) = self.snapshot(project).await?;
            let Some(claim_id) = v.claim_id.clone().filter(|_| am_i(&v, &me.session) == Verdict::Main) else {
                return Ok::<_, LeaseError>(outcome(Verdict::NotMain, Some(&me.session), "none", &f, v));
            };
            claim_client::channel_release_authed(&self.addr, self.auth.as_deref(), &claim_id, &me.claimer(), false).await?;
            self.post(project, &Event::Release { generation: v.generation, holder: me.clone(), at_ms: now_ms() }).await?;
            let (f, _, v) = self.snapshot(project).await?;
            Ok(outcome(Verdict::NotMain, Some(&me.session), "released", &f, v))
        }
        .await;
        r.unwrap_or_else(|e| Outcome::unknown(Some(&me.session), format!("could not release: {e}")))
    }

    /// Operator pin (`None` clears it). Audited as a record on the lease topic.
    pub async fn pin(&self, project: &str, session: Option<&str>, by: &str, reason: &str) -> Outcome {
        let r = async {
            self.ensure_topic(project).await?;
            self.post(project, &Event::Pin { session: session.map(str::to_string), by: by.to_string(), reason: reason.to_string(), at_ms: now_ms() })
                .await?;
            let (f, _, v) = self.snapshot(project).await?;
            Ok::<_, LeaseError>(outcome(Verdict::NotMain, None, "pinned", &f, v))
        }
        .await;
        r.unwrap_or_else(|e| Outcome::unknown(None, format!("could not record the pin: {e}")))
    }

    /// Read-only: who holds main, and (with a session) whether it is this session.
    pub async fn status(&self, project: &str, session: Option<&str>) -> Outcome {
        match self.snapshot(project).await {
            Ok((f, _, v)) => {
                // Without a session the question "am I main" has no subject: not-main.
                let verdict = session.map_or(Verdict::NotMain, |s| am_i(&v, s));
                outcome(verdict, session, "none", &f, v)
            }
            Err(e) => Outcome::unknown(session, format!("could not read the lease: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Holder {
        Holder { session: s.into(), agent: "agent-x".into(), host: "host".into(), pid: 7 }
    }
    fn acq(off: u64, generation: u64, s: &str, at: u64) -> Logged {
        Logged { offset: off, event: Event::Acquire { generation, holder: h(s), claim_id: format!("c-{s}-{at}"), at_ms: at, reason: "first".into(), from: None, lapsed_at_ms: None, obligations: vec![] } }
    }
    fn row(s: &str, at: i64, until: i64) -> Row {
        Row { claim_id: format!("c-{s}-{at}"), claimer: h(s).claimer(), claimed_at: at, claimed_until: until }
    }
    const T: Timing = Timing { lease_ms: 1_000, quiet_ms: 300, cooldown_ms: 500 };

    #[test]
    fn claimer_round_trips_and_raw_claimers_keep_their_name() {
        let x = Holder { session: "afce1d16;x".into(), agent: "ring20".into(), host: "h=1".into(), pid: 42 };
        let back = Holder::from_claimer(&x.claimer());
        assert_eq!(back.session, "afce1d16_x");
        assert_eq!(back.host, "h_1");
        assert_eq!(back.pid, 42);
        assert_eq!(Holder::from_claimer("worker-7").session, "worker-7");
    }

    #[test]
    fn never_held_is_vacant_and_eligible_now() {
        let f = fold(&[]);
        let v = view("p", &f, None, 100, T);
        assert!(v.holder.is_none());
        assert_eq!(v.eligible_at_ms, Some(100));
        assert_eq!(plan_claim(&v, &f, "s1"), Plan::Take { reason: "first" });
    }

    #[test]
    fn live_holder_keeps_main_and_others_are_held_off() {
        let f = fold(&[acq(1, 1, "s1", 1000)]);
        let v = view("p", &f, Some(&row("s1", 1000, 2000)), 1500, T);
        assert_eq!(am_i(&v, "s1"), Verdict::Main);
        assert_eq!(am_i(&v, "s2"), Verdict::NotMain);
        assert!(v.consistent);
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Held);
        assert!(matches!(plan_claim(&v, &f, "s1"), Plan::Renew { .. }));
    }

    #[test]
    fn takeover_waits_for_lapse_plus_quiet_and_cooldown() {
        let f = fold(&[acq(1, 1, "s1", 1000)]);
        // Lapsed at 2000: quiet ends 2300, cooldown since 1000 ends 1500 -> eligible at 2300.
        let v = view("p", &f, Some(&row("s1", 1000, 2000)), 2100, T);
        assert_eq!(v.eligible_at_ms, Some(2300));
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Wait { eligible_at_ms: 2300 });
        let v = view("p", &f, Some(&row("s1", 1000, 2000)), 2300, T);
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Take { reason: "takeover-after-lapse" });
        // The lapsed holder itself resumes at once (no take-back issue: nobody else holds it).
        assert_eq!(plan_claim(&v, &f, "s1"), Plan::Resume);
    }

    #[test]
    fn cooldown_binds_after_a_quick_clean_release() {
        let mut log = vec![acq(1, 1, "s1", 1000)];
        log.push(Logged { offset: 2, event: Event::Release { generation: 1, holder: h("s1"), at_ms: 1100 } });
        let f = fold(&log);
        let v = view("p", &f, None, 1200, T);
        // Clean release skips quiet; cooldown from 1000 -> 1500.
        assert_eq!(v.eligible_at_ms, Some(1500));
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Wait { eligible_at_ms: 1500 });
        let v = view("p", &f, None, 1500, T);
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Take { reason: "after-release" });
    }

    #[test]
    fn evicted_row_without_a_record_asks_for_a_lapse_record_first() {
        let f = fold(&[acq(1, 1, "s1", 1000)]);
        let v = view("p", &f, None, 9000, T);
        assert_eq!(v.eligible_at_ms, None);
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::RecordLapse);
        let mut log = vec![acq(1, 1, "s1", 1000)];
        log.push(Logged { offset: 2, event: Event::Lapse { generation: 1, holder: h("s1"), lapsed_at_ms: 9000, observed_at_ms: 9000, observed_by: "s2".into() } });
        let f = fold(&log);
        let v = view("p", &f, None, 9100, T);
        assert_eq!(v.eligible_at_ms, Some(9300));
    }

    #[test]
    fn pin_overrides_a_live_holder_and_takes_a_vacancy_at_once() {
        let mut log = vec![acq(1, 1, "s1", 1000)];
        log.push(Logged { offset: 2, event: Event::Pin { session: Some("s2".into()), by: "operator".into(), reason: "r".into(), at_ms: 1100 } });
        let f = fold(&log);
        let v = view("p", &f, Some(&row("s1", 1000, 5000)), 1200, T);
        assert_eq!(v.pinned.as_deref(), Some("s2"));
        assert!(matches!(plan_claim(&v, &f, "s2"), Plan::PinOverride { .. }));
        assert_eq!(plan_claim(&v, &f, "s3"), Plan::Held);
        let v = view("p", &f, Some(&row("s1", 1000, 2000)), 2001, T);
        assert_eq!(plan_claim(&v, &f, "s2"), Plan::Take { reason: "pin" });
    }

    #[test]
    fn generation_rises_on_every_change_and_the_loser_is_told() {
        let mut log = vec![acq(1, 1, "s1", 1000)];
        log.push(Logged { offset: 2, event: Event::Acquire { generation: 2, holder: h("s2"), claim_id: "c2".into(), at_ms: 3000, reason: "takeover-after-lapse".into(), from: Some(h("s1")), lapsed_at_ms: Some(2000), obligations: vec![] } });
        let f = fold(&log);
        assert_eq!(f.generation, 2);
        assert_eq!(f.current.as_ref().unwrap().holder.session, "s2");
        let (g, to, _) = f.losses.get("s1").unwrap();
        assert_eq!((*g, to.session.as_str()), (2, "s2"));
        // A stale lower-generation record changes nothing.
        let mut log2 = log.clone();
        log2.push(acq(3, 1, "s9", 4000));
        assert_eq!(fold(&log2).current.unwrap().holder.session, "s2");
    }

    #[test]
    fn duplicate_holders_resolve_by_pin_then_longest_then_stable_id() {
        let a = Candidate { holder: h("bbb"), since_ms: 100 };
        let b = Candidate { holder: h("aaa"), since_ms: 200 };
        let c = Candidate { holder: h("ccc"), since_ms: 100 };
        let all = [a.clone(), b.clone(), c.clone()];
        assert_eq!(resolve_duplicates(&all, Some("aaa")).unwrap().holder.session, "aaa");
        assert_eq!(resolve_duplicates(&all, None).unwrap().holder.session, "bbb", "longest, then lowest id");
        assert_eq!(resolve_duplicates(&all, Some("nobody")).unwrap().holder.session, "bbb");
        // In the fold: two acquires at one generation (a restored log).
        let log = vec![acq(1, 1, "zzz", 1000), acq(2, 1, "yyy", 900)];
        let f = fold(&log);
        assert_eq!(f.current.unwrap().holder.session, "yyy");
        assert!(f.duplicate.is_some());
    }

    #[test]
    fn verdict_exit_codes_are_the_hook_contract() {
        assert_eq!(Verdict::Main.exit_code(), 0);
        assert_eq!(Verdict::NotMain.exit_code(), 1);
        assert_eq!(Verdict::Unknown.exit_code(), 2);
        let o = Outcome::unknown(Some("s"), "hub down");
        assert_eq!(o.to_json()["verdict"], "unknown");
    }
}
