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
//! change them is the `TERMLINK_MAIN_TEST_*` seams, for tests, honoured only inside a marked
//! test estate (T-3686, see `crate::test_seam`).

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
pub use crate::test_seam::SeamRefused;

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
    /// `TERMLINK_MAIN_TEST_QUIET_MS`, `TERMLINK_MAIN_TEST_COOLDOWN_MS`, and those only inside a
    /// marked test estate (T-3686, AL-16): outside one, any `TERMLINK_MAIN_TEST_*` variable is
    /// a [`SeamRefused`] naming it, which every caller turns into `unknown` (exit 2). The lease
    /// floor is the hub's 1000 ms claim minimum.
    pub fn from_env() -> Result<Self, SeamRefused> {
        Self::resolve(std::env::vars_os(), crate::test_seam::in_estate_from_env)
    }

    /// [`Timing::from_env`] over an explicit environment and estate test (for tests). The
    /// estate is only looked up when a seam is present, so production pays nothing.
    pub fn resolve<I, F>(vars: I, in_estate: F) -> Result<Self, SeamRefused>
    where
        I: IntoIterator<Item = (std::ffi::OsString, std::ffi::OsString)>,
        F: FnOnce() -> bool,
    {
        let vars: Vec<_> = vars.into_iter().collect();
        let set = crate::test_seam::main_seams_set(vars.iter().cloned());
        let d = Timing::default();
        if set.is_empty() {
            return Ok(d);
        }
        if !in_estate() {
            return Err(SeamRefused { vars: set });
        }
        let num = |k: &str| {
            vars.iter()
                .find(|(n, _)| n == k)
                .and_then(|(_, v)| v.to_str())
                .and_then(|v| v.trim().parse::<u64>().ok())
        };
        Ok(Timing {
            lease_ms: num("TERMLINK_MAIN_TEST_LEASE_MS").unwrap_or(d.lease_ms).clamp(1_000, 60 * 60_000),
            quiet_ms: num("TERMLINK_MAIN_TEST_QUIET_MS").unwrap_or(d.quiet_ms),
            cooldown_ms: num("TERMLINK_MAIN_TEST_COOLDOWN_MS").unwrap_or(d.cooldown_ms),
        })
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

// ------------------------------------------------- B-25 (T-3694): prompt origin (AL-11, 6.13)

/// Per-session human-activity stamps (`Touch`), one per human-counted renewal. A separate
/// topic so that a flood of them can never push lease records out of `main:<p>`'s retention.
pub fn touch_topic(project: &str) -> String {
    format!("main-touch:{project}")
}
/// Claims that changed nothing because their origin was not `human` (`Ignored`).
pub fn ignored_topic(project: &str) -> String {
    format!("main-ignored:{project}")
}
/// Operator-set lease configuration: which harnesses report prompt origin (`OriginReporting`).
/// Created with explicit forever retention, so the switch never reverts by retention (CR-6.6).
pub fn config_topic(project: &str) -> String {
    format!("main-config:{project}")
}

/// The harness a claim comes from when the caller does not name one.
pub const DEFAULT_HARNESS: &str = "claude-code";
/// Records older than this on the touch/ignored topics are dropped by count (newest 1000).
pub const SIDE_KEEP: u64 = 1000;
/// Cross-host margin on the fast-takeover age (v0.2 FCW-56). Applied while the hub carries no
/// receive stamp of its own (CR-6.21 needs a hub change): when the holder's host differs from
/// the taker's, the two ages come from two wall clocks, so 30 s more silence is required.
pub const CROSS_HOST_MARGIN_MS: u64 = 30_000;

/// Where a prompt came from, as the adapter (checked by the receiver, AL-1) reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    Human,
    Automated,
    Unknown,
}

impl Origin {
    pub fn parse(s: &str) -> Option<Origin> {
        match s.trim() {
            "human" => Some(Origin::Human),
            "automated" => Some(Origin::Automated),
            "unknown" => Some(Origin::Unknown),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Origin::Human => "human",
            Origin::Automated => "automated",
            Origin::Unknown => "unknown",
        }
    }
}

/// Which hook a claim comes from (6.13.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimKind {
    /// UserPromptSubmit: the only kind that renews, for an origin-reporting harness.
    Prompt,
    /// SessionStart: a provisional take of a VACANT lease, nothing else.
    SessionStart,
    /// PostToolUse (tool activity): never renews for an origin-reporting harness.
    Tool,
}

impl ClaimKind {
    pub fn parse(s: &str) -> Option<ClaimKind> {
        match s.trim() {
            "prompt" => Some(ClaimKind::Prompt),
            "session-start" => Some(ClaimKind::SessionStart),
            "tool" | "post-tool-use" => Some(ClaimKind::Tool),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            ClaimKind::Prompt => "prompt",
            ClaimKind::SessionStart => "session-start",
            ClaimKind::Tool => "tool",
        }
    }
}

/// What a claim reports about itself. `ClaimRequest::default()` is the built claim (no origin).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRequest {
    pub kind: ClaimKind,
    /// `None` = the hooks sent no origin field (`absent`).
    pub origin: Option<Origin>,
    /// `pty-input`, `inferred`, `doorbell`, `watchdog`, ... (recorded, never decides).
    pub basis: Option<String>,
    /// SessionStart only: an interactive session (CR-6.5). `None` is not interactive.
    pub interactive: Option<bool>,
    pub harness: String,
}

impl Default for ClaimRequest {
    fn default() -> Self {
        ClaimRequest { kind: ClaimKind::Prompt, origin: None, basis: None, interactive: None, harness: DEFAULT_HARNESS.to_string() }
    }
}

fn is_false(b: &bool) -> bool {
    !*b
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
        /// `first`, `takeover-after-lapse`, `after-release`, `resume`, `pin-override`, `pin`,
        /// and (B-25) `provisional`, `over-provisional`, `after-provisional`, `fast-takeover`.
        reason: String,
        #[serde(default)]
        from: Option<Holder>,
        #[serde(default)]
        lapsed_at_ms: Option<u64>,
        /// Open claims the previous holder left on the obligation topics: uncertain, for the operator.
        #[serde(default)]
        obligations: Vec<Value>,
        /// B-25: `human`, `automated`, `unknown`, or `absent` (no origin field). Omitted on
        /// records written before B-25.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        origin: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        basis: Option<String>,
        /// B-25: a SessionStart take (valid for the quiet period, never blocks a human claim).
        #[serde(default, skip_serializing_if = "is_false")]
        provisional: bool,
        /// B-25 fast takeover: how long the previous holder had been without a human prompt.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        holder_human_age_s: Option<u64>,
    },
    /// B-25: a provisional holder's first human prompt makes it a full holder (same generation).
    Promote {
        generation: u64,
        holder: Holder,
        claim_id: String,
        at_ms: u64,
        origin: String,
        #[serde(default)]
        basis: Option<String>,
    },
    /// B-25, on `main-touch:<p>`: one human-counted renewal by the holder.
    Touch {
        generation: u64,
        session: String,
        claim_id: String,
        at_ms: u64,
        #[serde(default)]
        basis: Option<String>,
    },
    /// B-25, on `main-ignored:<p>`: a claim whose origin was not `human` changed nothing.
    Ignored {
        session: String,
        origin: String,
        #[serde(default)]
        basis: Option<String>,
        hook: String,
        harness: String,
        at_ms: u64,
    },
    /// B-25, on `main-config:<p>`: the operator switched a harness to (or off) origin reporting.
    OriginReporting {
        harness: String,
        on: bool,
        by: String,
        at_ms: u64,
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
    /// The posting identity's fingerprint (`sender_id`; the hub checks the signature against
    /// the posted key). B-25 counts a Touch only from the key that posted the holder's acquire.
    pub sender: Option<String>,
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
    /// B-25: a SessionStart take not yet promoted by a human prompt.
    pub provisional: bool,
    /// B-25: the acquire's origin (`None` on records written before B-25).
    pub origin: Option<String>,
    /// The identity that posted the acquire (the holder's key).
    pub sender: Option<String>,
    /// B-25: newest human-origin acquire or promote of this holding (claimer wall clock).
    pub human_at_ms: Option<u64>,
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
            Event::Acquire { generation, holder, claim_id, at_ms, from, origin, provisional, .. } => {
                let resume = f.current.as_ref().is_some_and(|c| c.generation == *generation && c.holder.session == holder.session);
                // A SessionStart (provisional) take is not a human prompt, whatever it reports.
                let human = (origin.as_deref() == Some("human") && !*provisional).then_some(*at_ms);
                if *generation > f.generation || f.current.is_none() {
                    if let Some(prev) = from.as_ref().or(f.current.as_ref().map(|c| &c.holder))
                        && prev.session != holder.session
                    {
                        f.losses.insert(prev.session.clone(), (*generation, holder.clone(), *at_ms));
                    }
                    f.generation = (*generation).max(f.generation);
                    f.current = Some(Current {
                        generation: *generation,
                        holder: holder.clone(),
                        claim_id: claim_id.clone(),
                        since_ms: *at_ms,
                        at_ms: *at_ms,
                        provisional: *provisional,
                        origin: origin.clone(),
                        sender: l.sender.clone(),
                        human_at_ms: human,
                    });
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
                    c.provisional = *provisional;
                    c.origin = origin.clone();
                    c.sender = l.sender.clone();
                    c.human_at_ms = human.max(c.human_at_ms);
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
                        f.current = Some(Current {
                            generation: *generation,
                            holder: win.holder.clone(),
                            claim_id: claim_id.clone(),
                            since_ms: win.since_ms,
                            at_ms: *at_ms,
                            provisional: *provisional,
                            origin: origin.clone(),
                            sender: l.sender.clone(),
                            human_at_ms: human,
                        });
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
            Event::Promote { generation, holder, claim_id, at_ms, origin, .. } => {
                // Only the live provisional holding of this session, by the key that took it.
                if let Some(c) = f.current.as_mut()
                    && c.generation == *generation
                    && c.holder.session == holder.session
                    && c.claim_id == *claim_id
                    && (c.sender.is_none() || c.sender == l.sender)
                {
                    c.provisional = false;
                    if origin == "human" {
                        c.human_at_ms = Some(*at_ms).max(c.human_at_ms);
                    }
                }
            }
            // Side-topic records never change the holding (folded by their own readers).
            Event::Touch { .. } | Event::Ignored { .. } | Event::OriginReporting { .. } => {}
        }
    }
    f
}

/// B-25 (AL-11 `last_human`): the holder's newest human-counted prompt at its generation —
/// the human acquire/promote of the holding, or a Touch on `main-touch:<p>` for the same
/// session, generation and claim id, posted by the key that posted the acquire (FCW-202).
/// `None` when there is none, or when the touch log could not be read (`touches = None`,
/// FCW-55): an undeterminable `last_human` never allows a fast takeover.
pub fn last_human(f: &Folded, touches: Option<&[Logged]>) -> Option<u64> {
    let c = f.current.as_ref()?;
    let touches = touches?;
    let from_touch = touches
        .iter()
        .filter_map(|l| match &l.event {
            Event::Touch { generation, session, claim_id, at_ms, .. }
                if *generation == c.generation
                    && *session == c.holder.session
                    && *claim_id == c.claim_id
                    && l.sender.is_some()
                    && l.sender == c.sender =>
            {
                Some(*at_ms)
            }
            _ => None,
        })
        .max();
    from_touch.max(c.human_at_ms)
}

/// B-25 (CR-6.6): the harnesses the operator has switched to origin reporting. The newest
/// record per harness wins; no record means not reporting.
pub fn reporting_harnesses(config: &[Logged]) -> Vec<String> {
    let mut on: BTreeMap<String, bool> = BTreeMap::new();
    for l in config {
        if let Event::OriginReporting { harness, on: o, .. } = &l.event {
            on.insert(harness.clone(), *o);
        }
    }
    on.into_iter().filter(|(_, o)| *o).map(|(h, _)| h).collect()
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
    /// B-25: `held`, `provisional`, `vacant` (never held, released, or a provisional that ran
    /// out), `lapsed`.
    pub state: &'static str,
    /// B-25: the live holder took main by SessionStart and has had no human prompt since.
    pub provisional: bool,
    /// B-25: origin of the live holding's acquire (`absent` for a non-reporting harness).
    pub origin: Option<String>,
    /// B-25: the holder's newest human-counted prompt (`None` = none or undeterminable).
    pub last_human_at_ms: Option<u64>,
    pub holder_human_age_s: Option<u64>,
    /// B-25: harnesses the operator switched to origin reporting (`None` = not read).
    pub origin_reporting: Option<Vec<String>>,
}

pub fn view(project: &str, f: &Folded, row: Option<&Row>, now: u64, t: Timing) -> View {
    let active = row.filter(|r| r.claimed_until > now as i64);
    let mut v = View {
        state: "vacant",
        provisional: false,
        origin: None,
        last_human_at_ms: None,
        holder_human_age_s: None,
        origin_reporting: None,
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
        if let Some(c) = f.current.as_ref().filter(|c| c.holder.session == h.session && c.claim_id == r.claim_id) {
            v.provisional = c.provisional;
            v.origin = c.origin.clone();
        }
        v.state = if v.provisional { "provisional" } else { "held" };
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
        if cur.provisional {
            // B-25 (E-23): a provisional holding that ran out leaves the lease VACANT at once:
            // no quiet period, no cooldown (it never blocked anyone).
            v.eligible_at_ms = Some(e.at_ms);
        } else {
            let after_end = e.at_ms + if e.clean { 0 } else { t.quiet_ms };
            let after_change = f.last_change_ms.unwrap_or(0) + t.cooldown_ms;
            v.eligible_at_ms = Some(after_end.max(after_change));
        }
        v.state = if e.clean || cur.provisional { "vacant" } else { "lapsed" };
    } else if cur.provisional {
        // A provisional row already evicted: it ran out; nothing to wait for.
        v.eligible_at_ms = Some(now);
        v.state = "vacant";
    } else {
        v.state = "lapsed";
    }
    v
}

/// B-25: fill the origin fields of a view from the side topics.
pub fn with_origin(mut v: View, f: &Folded, touches: Option<&[Logged]>, reporting: Option<Vec<String>>) -> View {
    if v.holder.is_some() {
        v.last_human_at_ms = last_human(f, touches);
        v.holder_human_age_s = v.last_human_at_ms.map(|t| v.now_ms.saturating_sub(t) / 1000);
    }
    v.origin_reporting = reporting;
    v
}

/// E-31 (W2 measurement, AR-6): counted renewals by basis, ignored claims by origin and basis,
/// and any holder change by a non-human acquire (a FAIL). A measurement, never a threshold.
pub fn renewal_report(log: &[Logged], touches: &[Logged], ignored: &[Logged]) -> Value {
    let mut by_basis: BTreeMap<String, u64> = BTreeMap::new();
    let mut counted = 0u64;
    for l in touches {
        if let Event::Touch { basis, .. } = &l.event {
            counted += 1;
            *by_basis.entry(basis.clone().unwrap_or_else(|| "none".into())).or_default() += 1;
        }
    }
    let mut ign: BTreeMap<String, u64> = BTreeMap::new();
    let mut ignored_total = 0u64;
    for l in ignored {
        if let Event::Ignored { origin, basis, .. } = &l.event {
            ignored_total += 1;
            *ign.entry(format!("{origin}/{}", basis.as_deref().unwrap_or("none"))).or_default() += 1;
        }
    }
    let fails: Vec<Value> = log
        .iter()
        .filter_map(|l| match &l.event {
            Event::Acquire { origin: Some(o), holder, generation, reason, .. } if o == "automated" || o == "unknown" => {
                Some(json!({"offset": l.offset, "generation": generation, "holder": holder.session, "origin": o, "reason": reason}))
            }
            _ => None,
        })
        .collect();
    let share: BTreeMap<String, Value> = by_basis
        .iter()
        .map(|(b, n)| (b.clone(), json!({"numerator": n, "denominator": counted})))
        .collect();
    json!({
        "counted_renewals": counted,
        "by_basis": share,
        "ignored_total": ignored_total,
        "ignored": ign,
        "non_human_holder_changes": fails,
        "verdict": if fails.is_empty() { "PASS" } else { "FAIL" },
    })
}

/// B-25 tightened plan (AL-11 table), for a harness the operator switched to origin reporting.
/// `last_human` is the holder's (see [`last_human`]); `my_host` decides the cross-host margin.
pub fn plan_reporting(v: &View, f: &Folded, me: &Holder, req: &ClaimRequest, last_human: Option<u64>) -> Plan {
    let session = me.session.as_str();
    let origin = req.origin.unwrap_or(Origin::Unknown); // FCW-53: absent in a reporting harness is unknown
    match req.kind {
        // 6.13.1: tool activity never touches the lease.
        ClaimKind::Tool => return Plan::AnswerOnly { why: "tool activity never renews main" },
        ClaimKind::SessionStart => {
            if v.pinned.as_deref() == Some(session) {
                return plan_claim(v, f, session);
            }
            let vacant = v.holder.is_none() && v.state == "vacant";
            if !vacant {
                return Plan::AnswerOnly { why: "SessionStart takes only a vacant lease" }; // FCW-57
            }
            if req.interactive != Some(true) {
                return Plan::AnswerOnly { why: "a non-interactive session never takes main" }; // FCW-203
            }
            return Plan::TakeProvisional;
        }
        ClaimKind::Prompt => {}
    }
    if origin != Origin::Human {
        return Plan::Ignored; // FCW-52
    }
    if let Some(h) = &v.holder {
        let claim_id = v.claim_id.clone().unwrap_or_default();
        if h.session == session {
            return if v.provisional { Plan::Promote { claim_id } } else { Plan::TouchRenew { claim_id } };
        }
        if v.pinned.as_deref() == Some(session) {
            return Plan::PinOverride { claim_id, from: h.clone() };
        }
        if v.pinned.as_deref() == Some(h.session.as_str()) {
            return Plan::Held; // FCW-54
        }
        if v.provisional {
            return Plan::OverProvisional { claim_id, from: h.clone() };
        }
        let Some(lh) = last_human else { return Plan::Held }; // FCW-55
        let margin = if !h.host.is_empty() && h.host != me.host { CROSS_HOST_MARGIN_MS } else { 0 };
        let age = v.now_ms.saturating_sub(lh);
        if age > v.timing.quiet_ms + margin {
            return Plan::FastTakeover { claim_id, from: h.clone(), age_s: age / 1000 };
        }
        return Plan::Held; // FCW-56
    }
    if v.pinned.as_deref() == Some(session) {
        return Plan::Take { reason: "pin" };
    }
    if v.pinned.is_some() {
        return Plan::Held; // AL-11 table: VACANT takes only with pinned in {none, me}
    }
    match &f.current {
        None => Plan::Take { reason: "first" },
        Some(c) if c.holder.session == session => Plan::Resume,
        // 6.13.4: a human prompt takes a vacated lease at once: no quiet, no cooldown.
        Some(c) if c.provisional => Plan::Take { reason: "after-provisional" },
        Some(_) if f.ended.is_some_and(|e| e.clean) => Plan::Take { reason: "after-release" },
        Some(_) => Plan::Take { reason: "takeover-after-lapse" },
    }
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
    // B-25 (reporting harnesses only):
    /// Answer the question, change nothing, record nothing (tool activity, SessionStart on a
    /// lease that is not vacant, a non-interactive SessionStart).
    AnswerOnly { why: &'static str },
    /// Origin not `human`: change nothing, record `ignored: origin` on `main-ignored:<p>`.
    Ignored,
    /// Interactive SessionStart on a VACANT lease: take it for the quiet period only.
    TakeProvisional,
    /// The provisional holder's first human prompt: renew to the full lease.
    Promote { claim_id: String },
    /// The holder's human prompt: stamp a Touch, then renew (in that order, invariant 7).
    TouchRenew { claim_id: String },
    /// A human prompt elsewhere takes a provisional holding at once.
    OverProvisional { claim_id: String, from: Holder },
    /// The holder has had no human prompt for longer than the quiet period (V-20).
    FastTakeover { claim_id: String, from: Holder, age_s: u64 },
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
    /// B-25: e.g. "this harness's hooks do not report prompt origin" (NID, E-22).
    pub note: Option<String>,
    /// B-25: the newest `ignored: origin` records (status only).
    pub ignored_recent: Vec<Value>,
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
            note: None,
            ignored_recent: vec![],
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
            "state": v.map(|v| v.state),
            "provisional": v.map(|v| v.provisional),
            "origin": v.and_then(|v| v.origin.as_ref()),
            "last_human_at_ms": v.and_then(|v| v.last_human_at_ms),
            "holder_human_age_s": v.and_then(|v| v.holder_human_age_s),
            "origin_reporting": v.and_then(|v| v.origin_reporting.as_ref()),
            "note": self.note,
            "ignored_recent": self.ignored_recent,
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
        note: None,
        ignored_recent: vec![],
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

/// The claimer string to present to the hub for a renew/release by `session`.
///
/// The lease identity is the SESSION; the hub checks ownership by the exact `claimer` string,
/// which also carries the agent, host and pid of the process that claimed. A later process of
/// the same session (a SessionEnd hook, a prompt hook) differs in pid, so it presents the live
/// row's own claimer. `None` when there is no row or the row belongs to another session, so
/// another session is still refused (T-3660).
pub fn session_claimer(row: Option<&Row>, session: &str) -> Option<String> {
    row.filter(|r| Holder::from_claimer(&r.claimer).session == session).map(|r| r.claimer.clone())
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

/// B-25 fields an acquire carries.
#[derive(Debug, Clone, Default)]
struct AcqTag {
    origin: Option<String>,
    basis: Option<String>,
    provisional: bool,
    age_s: Option<u64>,
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
        self.read_topic(&topic(project)).await
    }

    /// Any lease-family topic (`main:`, `main-touch:`, `main-ignored:`, `main-config:`), in
    /// offset order, with each record's sender. An unknown topic is `(false, [])`.
    pub async fn read_topic(&self, t: &str) -> Result<(bool, Vec<Logged>), LeaseError> {
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
                    let sender = m.get("sender_id").and_then(Value::as_str).map(str::to_string);
                    out.push(Logged { offset: off, event, sender });
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

    /// B-25: the harnesses switched to origin reporting. A read error is an error (FCW-201:
    /// never a guess either way); an unknown topic is "none".
    pub async fn origin_reporting(&self, project: &str) -> Result<Vec<String>, LeaseError> {
        let (_, log) = self.read_topic(&config_topic(project)).await?;
        Ok(reporting_harnesses(&log))
    }

    /// B-25: the snapshot plus the holder's `last_human` (a touch log that cannot be read
    /// leaves it `None`, FCW-55) and the reporting harnesses.
    pub async fn snapshot_full(&self, project: &str, reporting: Option<Vec<String>>) -> Result<(Folded, Option<Row>, View), LeaseError> {
        let (f, row, v) = self.snapshot(project).await?;
        let touches = if v.holder.is_some() { self.read_topic(&touch_topic(project)).await.ok().map(|(_, l)| l) } else { Some(vec![]) };
        let v = with_origin(v, &f, touches.as_deref(), reporting);
        Ok((f, row, v))
    }

    async fn post(&self, project: &str, ev: &Event) -> Result<u64, LeaseError> {
        self.post_to(&topic(project), ev).await
    }

    async fn post_to(&self, t: &str, ev: &Event) -> Result<u64, LeaseError> {
        let id = self.identity.as_ref().ok_or_else(|| LeaseError::Other("no signing identity to write the lease record".into()))?;
        let payload = serde_json::to_vec(ev).map_err(|e| LeaseError::Other(e.to_string()))?;
        let ts = now_ms() as i64;
        let bytes = termlink_protocol::control::channel::canonical_sign_bytes(t, MSG_TYPE, &payload, None, ts);
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

    /// B-25: post to a side topic, creating it on first use (`main-touch:`/`main-ignored:`
    /// newest 1000; `main-config:` forever, named owner and reason, so it never reverts).
    async fn post_side(&self, t: &str, ev: &Event) -> Result<u64, LeaseError> {
        match self.post_to(t, ev).await {
            Err(LeaseError::Hub { code, .. }) if code == error_code::CHANNEL_TOPIC_UNKNOWN => {}
            other => return other,
        }
        let retention = if t.starts_with("main-config:") { json!({"kind": "forever"}) } else { json!({"kind": "messages", "value": SIDE_KEEP}) };
        self.call(
            method::CHANNEL_CREATE,
            json!({"name": t, "retention": retention, "owner": "operator", "reason": "T-3694 main lease B-25 (OD-18 / R-67)"}),
        )
        .await?;
        self.post_to(t, ev).await
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

    /// Acquire or renew main for `me`, by the R-67 rules (the built claim: no origin field).
    pub async fn claim(&self, project: &str, me: &Holder) -> Outcome {
        self.claim_with(project, me, &ClaimRequest::default()).await
    }

    /// B-25 (AL-11): a claim that says which hook it comes from and the prompt's origin.
    /// A harness the operator has not switched to origin reporting is decided exactly as built
    /// (NID), with the origin recorded; a reporting harness gets the tightened plan.
    pub async fn claim_with(&self, project: &str, me: &Holder, req: &ClaimRequest) -> Outcome {
        match self.claim_inner(project, me, req).await {
            Ok(o) => o,
            Err(e) => Outcome::unknown(Some(&me.session), format!("could not read or write the lease: {e}")),
        }
    }

    /// Claim offset 0 and record the acquire. `Ok(None)` on a CAS conflict (re-plan).
    #[allow(clippy::too_many_arguments)]
    async fn take(
        &self,
        project: &str,
        me: &Holder,
        reason: &str,
        from: Option<Holder>,
        generation: u64,
        lapsed: Option<u64>,
        ttl: u32,
        tag: &AcqTag,
    ) -> Result<Option<()>, LeaseError> {
        let claimer = me.claimer();
        let c = match claim_client::channel_claim_authed(&self.addr, self.auth.as_deref(), &topic(project), 0, &claimer, ttl).await {
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
            origin: tag.origin.clone(),
            basis: tag.basis.clone(),
            provisional: tag.provisional,
            holder_human_age_s: tag.age_s,
        };
        if let Err(e) = self.post(project, &ev).await {
            // Never hold an unrecorded lease: give it back and report unknown.
            let _ = claim_client::channel_release_authed(&self.addr, self.auth.as_deref(), &c.claim_id, &claimer, false).await;
            return Err(e);
        }
        Ok(Some(()))
    }

    async fn force_release(&self, claim_id: &str, why: &str) -> Result<(), LeaseError> {
        match claim_client::channel_force_release_authed(&self.addr, self.auth.as_deref(), claim_id, Some(why)).await {
            Ok(_) | Err(ClaimError::NotFound { .. }) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn claim_inner(&self, project: &str, me: &Holder, req: &ClaimRequest) -> Result<Outcome, LeaseError> {
        // CR-6.6 / FCW-201: the operator's switch decides; unreadable is unknown, never a guess.
        let reporting = self.origin_reporting(project).await?;
        let rep = reporting.contains(&req.harness);
        self.ensure_topic(project).await?;
        if !rep {
            let tag = AcqTag { origin: Some(req.origin.map_or("absent", Origin::as_str).to_string()), basis: req.basis.clone(), ..AcqTag::default() };
            let mut o = self.claim_built(project, me, &tag, Some(reporting)).await?;
            if req.origin.is_none() {
                o.note = Some(format!("this harness ({}) does not report prompt origin; the claim counts as built", req.harness));
            } else {
                o.note = Some(format!("harness {} is not switched to origin reporting; origin recorded, claim decided as built", req.harness));
            }
            return Ok(o);
        }
        self.claim_reporting(project, me, req, reporting).await
    }

    /// The built claim (T-3654), with the acquire records tagged by origin.
    async fn claim_built(&self, project: &str, me: &Holder, tag: &AcqTag, reporting: Option<Vec<String>>) -> Result<Outcome, LeaseError> {
        let claimer = me.claimer();
        let ttl = self.timing.lease_ms as u32;
        for _ in 0..3 {
            let (f, row, v) = self.snapshot(project).await?;
            let action = match plan_claim(&v, &f, &me.session) {
                Plan::Renew { claim_id } => {
                    let held_as = session_claimer(row.as_ref(), &me.session).unwrap_or_else(|| claimer.clone());
                    match claim_client::channel_renew_authed(&self.addr, self.auth.as_deref(), &claim_id, &held_as, ttl).await {
                        Ok(_) => {
                            self.repair(project, me, &f, &v, &claim_id, tag).await?;
                            "renewed"
                        }
                        Err(ClaimError::Expired { .. }) | Err(ClaimError::NotFound { .. }) => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                Plan::PinOverride { claim_id, from } => {
                    self.force_release(&claim_id, &format!("T-3654 operator pin names {}", me.session)).await?;
                    match self.take(project, me, "pin-override", Some(from), f.generation + 1, None, ttl, tag).await? {
                        Some(()) => "acquired",
                        None => continue,
                    }
                }
                Plan::Take { reason } => {
                    let from = f.current.as_ref().map(|c| c.holder.clone());
                    match self.take(project, me, reason, from, f.generation + 1, v.vacant_since_ms, ttl, tag).await? {
                        Some(()) => "acquired",
                        None => continue,
                    }
                }
                Plan::Resume => match self.take(project, me, "resume", None, f.generation.max(1), v.vacant_since_ms, ttl, tag).await? {
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
                    return Ok(outcome(Verdict::NotMain, Some(&me.session), "lapse-recorded", &f, with_origin(v, &f, None, reporting)));
                }
                _ => {
                    return Ok(outcome(Verdict::NotMain, Some(&me.session), "none", &f, with_origin(v, &f, None, reporting)));
                }
            };
            let (f, _, v) = self.snapshot_full(project, reporting.clone()).await?;
            let verdict = am_i(&v, &me.session);
            return Ok(outcome(verdict, Some(&me.session), action, &f, v));
        }
        let (f, _, v) = self.snapshot_full(project, reporting).await?;
        let verdict = am_i(&v, &me.session);
        Ok(outcome(verdict, Some(&me.session), "contended", &f, v))
    }

    /// Self-heal a claim whose acquire record is missing (crash between claim and post).
    async fn repair(&self, project: &str, me: &Holder, f: &Folded, v: &View, claim_id: &str, tag: &AcqTag) -> Result<(), LeaseError> {
        if v.consistent {
            return Ok(());
        }
        let ev = Event::Acquire {
            generation: f.generation + 1,
            holder: me.clone(),
            claim_id: claim_id.to_string(),
            at_ms: now_ms(),
            reason: "record-repair".into(),
            from: f.current.as_ref().map(|c| c.holder.clone()).filter(|h| h.session != me.session),
            lapsed_at_ms: None,
            obligations: vec![],
            origin: tag.origin.clone(),
            basis: tag.basis.clone(),
            provisional: false,
            holder_human_age_s: None,
        };
        self.post(project, &ev).await.map(|_| ())
    }

    /// B-25: the tightened claim for an origin-reporting harness (AL-11 table).
    async fn claim_reporting(&self, project: &str, me: &Holder, req: &ClaimRequest, reporting: Vec<String>) -> Result<Outcome, LeaseError> {
        let claimer = me.claimer();
        let ttl = self.timing.lease_ms as u32;
        let prov_ttl = self.timing.quiet_ms.max(1_000) as u32; // V-20: provisional valid for the quiet period
        let origin = req.origin.unwrap_or(Origin::Unknown);
        let human = AcqTag { origin: Some(origin.as_str().to_string()), basis: req.basis.clone(), ..AcqTag::default() };
        let rep = Some(reporting.clone());
        for _ in 0..3 {
            let (f, row, v) = self.snapshot_full(project, rep.clone()).await?;
            let plan = plan_reporting(&v, &f, me, req, v.last_human_at_ms);
            let action: String = match plan {
                Plan::AnswerOnly { why } => {
                    let verdict = am_i(&v, &me.session);
                    let mut o = outcome(verdict, Some(&me.session), "none", &f, v);
                    o.note = Some(why.to_string());
                    return Ok(o);
                }
                Plan::Ignored => {
                    let ev = Event::Ignored {
                        session: me.session.clone(),
                        origin: req.origin.map_or("absent", Origin::as_str).to_string(),
                        basis: req.basis.clone(),
                        hook: req.kind.as_str().to_string(),
                        harness: req.harness.clone(),
                        at_ms: now_ms(),
                    };
                    let action = match self.post_side(&ignored_topic(project), &ev).await {
                        Ok(_) => "ignored: origin",
                        Err(_) => "ignored-unrecorded", // FCW-59: the lease is unchanged either way
                    };
                    let verdict = am_i(&v, &me.session);
                    let mut o = outcome(verdict, Some(&me.session), action, &f, v);
                    o.note = Some(format!("origin {} does not count for main (only human prompts renew, take or take over)", origin.as_str()));
                    return Ok(o);
                }
                Plan::TakeProvisional => {
                    let tag = AcqTag { provisional: true, ..human.clone() };
                    let from = f.current.as_ref().map(|c| c.holder.clone());
                    match self.take(project, me, "provisional", from, f.generation + 1, v.vacant_since_ms, prov_ttl, &tag).await? {
                        Some(()) => "acquired-provisional".into(),
                        None => continue,
                    }
                }
                Plan::Promote { claim_id } => {
                    let held_as = session_claimer(row.as_ref(), &me.session).unwrap_or_else(|| claimer.clone());
                    let ev = Event::Promote {
                        generation: v.generation,
                        holder: me.clone(),
                        claim_id: claim_id.clone(),
                        at_ms: now_ms(),
                        origin: origin.as_str().into(),
                        basis: req.basis.clone(),
                    };
                    self.post(project, &ev).await?;
                    match claim_client::channel_renew_authed(&self.addr, self.auth.as_deref(), &claim_id, &held_as, ttl).await {
                        Ok(_) => "promoted".into(),
                        Err(ClaimError::Expired { .. }) | Err(ClaimError::NotFound { .. }) => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                Plan::TouchRenew { claim_id } => {
                    let held_as = session_claimer(row.as_ref(), &me.session).unwrap_or_else(|| claimer.clone());
                    // Invariant 7: stamp first, then extend. No stamp, no renewal (FCW-58).
                    let ev = Event::Touch { generation: v.generation, session: me.session.clone(), claim_id: claim_id.clone(), at_ms: now_ms(), basis: req.basis.clone() };
                    if let Err(e) = self.post_side(&touch_topic(project), &ev).await {
                        return Ok(Outcome::unknown(Some(&me.session), format!("could not record the human prompt (Touch), so main was not renewed: {e}")));
                    }
                    match claim_client::channel_renew_authed(&self.addr, self.auth.as_deref(), &claim_id, &held_as, ttl).await {
                        Ok(_) => {
                            self.repair(project, me, &f, &v, &claim_id, &human).await?;
                            "renewed".into()
                        }
                        Err(ClaimError::Expired { .. }) | Err(ClaimError::NotFound { .. }) => continue,
                        Err(e) => return Err(e.into()),
                    }
                }
                Plan::PinOverride { claim_id, from } => {
                    self.force_release(&claim_id, &format!("T-3654 operator pin names {}", me.session)).await?;
                    match self.take(project, me, "pin-override", Some(from), f.generation + 1, None, ttl, &human).await? {
                        Some(()) => "acquired".into(),
                        None => continue,
                    }
                }
                Plan::OverProvisional { claim_id, from } => {
                    self.force_release(&claim_id, &format!("T-3694 human prompt in {} over a provisional holder", me.session)).await?;
                    match self.take(project, me, "over-provisional", Some(from), f.generation + 1, None, ttl, &human).await? {
                        Some(()) => "acquired".into(),
                        None => continue,
                    }
                }
                Plan::FastTakeover { claim_id, from, .. } => {
                    // Race narrowing: re-read the holder's last human prompt on a fresh snapshot.
                    let (f2, _, v2) = self.snapshot_full(project, rep.clone()).await?;
                    let still = plan_reporting(&v2, &f2, me, req, v2.last_human_at_ms);
                    let Plan::FastTakeover { age_s, .. } = still else { continue };
                    if v2.claim_id.as_deref() != Some(claim_id.as_str()) {
                        continue;
                    }
                    self.force_release(&claim_id, &format!("T-3694 fast-takeover by {}", me.session)).await?;
                    let tag = AcqTag { age_s: Some(age_s), ..human.clone() };
                    match self.take(project, me, "fast-takeover", Some(from), f2.generation + 1, None, ttl, &tag).await? {
                        Some(()) => "acquired".into(),
                        None => continue,
                    }
                }
                Plan::Take { reason } => {
                    let from = f.current.as_ref().map(|c| c.holder.clone());
                    match self.take(project, me, reason, from, f.generation + 1, v.vacant_since_ms, ttl, &human).await? {
                        Some(()) => "acquired".into(),
                        None => continue,
                    }
                }
                Plan::Resume => match self.take(project, me, "resume", None, f.generation.max(1), v.vacant_since_ms, ttl, &human).await? {
                    Some(()) => "resumed".into(),
                    None => continue,
                },
                Plan::Renew { .. } | Plan::RecordLapse | Plan::Wait { .. } | Plan::Held => {
                    let verdict = am_i(&v, &me.session);
                    return Ok(outcome(verdict, Some(&me.session), "none", &f, v));
                }
            };
            let (f, _, v) = self.snapshot_full(project, rep.clone()).await?;
            let verdict = am_i(&v, &me.session);
            return Ok(outcome(verdict, Some(&me.session), &action, &f, v));
        }
        let (f, _, v) = self.snapshot_full(project, rep).await?;
        let verdict = am_i(&v, &me.session);
        Ok(outcome(verdict, Some(&me.session), "contended", &f, v))
    }

    /// B-25 operator act (CR-6.6): switch a harness to (or off) origin reporting. Only a runme
    /// operator action calls this, after the harness's adapter passes the parity fixtures.
    pub async fn set_origin_reporting(&self, project: &str, harness: &str, on: bool, by: &str) -> Outcome {
        let r = async {
            self.post_side(&config_topic(project), &Event::OriginReporting { harness: harness.into(), on, by: by.into(), at_ms: now_ms() }).await?;
            let reporting = self.origin_reporting(project).await?;
            if reporting.iter().any(|h| h == harness) != on {
                return Err(LeaseError::Other("the origin-reporting record did not read back".into()));
            }
            let (f, _, v) = self.snapshot_full(project, Some(reporting)).await?;
            Ok::<_, LeaseError>(outcome(Verdict::NotMain, None, if on { "origin-reporting-on" } else { "origin-reporting-off" }, &f, v))
        }
        .await;
        r.unwrap_or_else(|e| Outcome::unknown(None, format!("could not record origin reporting: {e}")))
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
            let (f, row, v) = self.snapshot(project).await?;
            let held = v.claim_id.clone().filter(|_| am_i(&v, &me.session) == Verdict::Main).zip(session_claimer(row.as_ref(), &me.session));
            let Some((claim_id, held_as)) = held else {
                return Ok::<_, LeaseError>(outcome(Verdict::NotMain, Some(&me.session), "none", &f, v));
            };
            claim_client::channel_release_authed(&self.addr, self.auth.as_deref(), &claim_id, &held_as, false).await?;
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
        // B-25: origin reporting and the ignored records are shown when readable; a status
        // read never fails on them (only a claim must know the switch, FCW-201).
        let reporting = self.origin_reporting(project).await.ok();
        match self.snapshot_full(project, reporting).await {
            Ok((f, _, v)) => {
                // Without a session the question "am I main" has no subject: not-main.
                let verdict = session.map_or(Verdict::NotMain, |s| am_i(&v, s));
                let mut o = outcome(verdict, session, "none", &f, v);
                if let Ok((_, log)) = self.read_topic(&ignored_topic(project)).await {
                    o.ignored_recent = log.iter().rev().take(RECENT).rev().map(|l| json!({"offset": l.offset, "event": l.event})).collect();
                }
                if o.view.as_ref().is_some_and(|v| v.origin.as_deref() == Some("absent")) {
                    o.note = Some("this harness's hooks do not report prompt origin; its claims count as built".into());
                }
                o
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
        Logged { offset: off, event: Event::Acquire { generation, holder: h(s), claim_id: format!("c-{s}-{at}"), at_ms: at, reason: "first".into(), from: None, lapsed_at_ms: None, obligations: vec![], origin: None, basis: None, provisional: false, holder_human_age_s: None }, sender: Some(format!("key-{s}")) }
    }
    fn row(s: &str, at: i64, until: i64) -> Row {
        Row { claim_id: format!("c-{s}-{at}"), claimer: h(s).claimer(), claimed_at: at, claimed_until: until }
    }
    const T: Timing = Timing { lease_ms: 1_000, quiet_ms: 300, cooldown_ms: 500 };

    fn env(pairs: &[(&str, &str)]) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
        pairs.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect()
    }

    // T-3686 (E-163.1, FCW-60): outside a test estate a seam is refused, naming the variable.
    #[test]
    fn seam_outside_estate_is_refused_naming_the_variable() {
        let e = env(&[("PATH", "/bin"), ("TERMLINK_MAIN_TEST_LEASE_MS", "1000")]);
        let err = Timing::resolve(e, || false).expect_err("seam outside an estate must be refused");
        assert_eq!(err.vars, vec!["TERMLINK_MAIN_TEST_LEASE_MS".to_string()]);
        assert!(err.to_string().contains("TERMLINK_MAIN_TEST_LEASE_MS"));
        assert!(!err.to_string().contains("1000"), "the value is never echoed");
        // An unknown variable with the prefix is refused too, even if unparseable.
        let err = Timing::resolve(env(&[("TERMLINK_MAIN_TEST_QUIET_MS", "junk")]), || false).unwrap_err();
        assert_eq!(err.vars, vec!["TERMLINK_MAIN_TEST_QUIET_MS".to_string()]);
    }

    #[test]
    fn seam_inside_estate_drives_the_timings() {
        let e = env(&[
            ("TERMLINK_MAIN_TEST_LEASE_MS", "1500"),
            ("TERMLINK_MAIN_TEST_QUIET_MS", "800"),
            ("TERMLINK_MAIN_TEST_COOLDOWN_MS", "400"),
        ]);
        assert_eq!(Timing::resolve(e, || true), Ok(Timing { lease_ms: 1_500, quiet_ms: 800, cooldown_ms: 400 }));
        // The hub's 1000 ms claim floor still applies inside an estate.
        let t = Timing::resolve(env(&[("TERMLINK_MAIN_TEST_LEASE_MS", "5")]), || true).unwrap();
        assert_eq!(t.lease_ms, 1_000);
    }

    #[test]
    fn no_seam_is_defaults_and_never_asks_the_estate() {
        let t = Timing::resolve(env(&[("PATH", "/bin")]), || panic!("estate looked up without a seam")).unwrap();
        assert_eq!(t, Timing::default());
    }

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
        log.push(Logged { offset: 2, event: Event::Release { generation: 1, holder: h("s1"), at_ms: 1100 }, sender: None });
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
        log.push(Logged { offset: 2, event: Event::Lapse { generation: 1, holder: h("s1"), lapsed_at_ms: 9000, observed_at_ms: 9000, observed_by: "s2".into() }, sender: None });
        let f = fold(&log);
        let v = view("p", &f, None, 9100, T);
        assert_eq!(v.eligible_at_ms, Some(9300));
    }

    #[test]
    fn pin_overrides_a_live_holder_and_takes_a_vacancy_at_once() {
        let mut log = vec![acq(1, 1, "s1", 1000)];
        log.push(Logged { offset: 2, event: Event::Pin { session: Some("s2".into()), by: "operator".into(), reason: "r".into(), at_ms: 1100 }, sender: None });
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
        log.push(Logged { offset: 2, event: Event::Acquire { generation: 2, holder: h("s2"), claim_id: "c2".into(), at_ms: 3000, reason: "takeover-after-lapse".into(), from: Some(h("s1")), lapsed_at_ms: Some(2000), obligations: vec![], origin: None, basis: None, provisional: false, holder_human_age_s: None }, sender: None });
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
    fn session_claimer_matches_by_session_not_process() {
        let r = row("S-A", 1, 10);
        let mut later = h("S-A");
        later.pid = 99;
        later.agent = "hook".into();
        assert_ne!(later.claimer(), r.claimer, "a later process of the session has another claimer string");
        assert_eq!(session_claimer(Some(&r), "S-A"), Some(r.claimer.clone()));
        assert_eq!(session_claimer(Some(&r), "S-B"), None, "another session is refused");
        assert_eq!(session_claimer(None, "S-A"), None);
        let raw = Row { claimer: "worker-7".into(), ..r.clone() };
        assert_eq!(session_claimer(Some(&raw), "S-A"), None);
    }

    // ---------------------------------------------------------------- B-25 (T-3694)

    const MIN: u64 = 60_000;
    /// Production timings: quiet 5 min (V-20), lease 15 min (V-9).
    const P: Timing = Timing { lease_ms: LEASE_MS, quiet_ms: QUIET_MS, cooldown_ms: COOLDOWN_MS };

    fn acq_full(off: u64, generation: u64, s: &str, at: u64, origin: &str, provisional: bool, claim: &str) -> Logged {
        Logged {
            offset: off,
            event: Event::Acquire {
                generation,
                holder: h(s),
                claim_id: claim.into(),
                at_ms: at,
                reason: if provisional { "provisional".into() } else { "first".into() },
                from: None,
                lapsed_at_ms: None,
                obligations: vec![],
                origin: Some(origin.into()),
                basis: None,
                provisional,
                holder_human_age_s: None,
            },
            sender: Some(format!("key-{s}")),
        }
    }
    fn live(s: &str, claim: &str, at: i64, until: i64) -> Row {
        Row { claim_id: claim.into(), claimer: h(s).claimer(), claimed_at: at, claimed_until: until }
    }
    fn touch(off: u64, generation: u64, s: &str, claim: &str, at: u64, key: &str) -> Logged {
        Logged { offset: off, event: Event::Touch { generation, session: s.into(), claim_id: claim.into(), at_ms: at, basis: Some("pty-input".into()) }, sender: Some(key.into()) }
    }
    fn req(kind: ClaimKind, origin: Option<Origin>) -> ClaimRequest {
        ClaimRequest { kind, origin, basis: None, interactive: Some(true), harness: DEFAULT_HARNESS.into() }
    }
    fn human() -> ClaimRequest {
        req(ClaimKind::Prompt, Some(Origin::Human))
    }
    /// S1 holds (human acquire at t0 = 0) on a live 15 min claim, seen at `now`.
    fn held_s1(now: u64) -> (Folded, View) {
        let f = fold(&[acq_full(1, 1, "s1", 0, "human", false, "c1")]);
        let v = with_origin(view("p", &f, Some(&live("s1", "c1", 0, LEASE_MS as i64)), now, P), &f, Some(&[]), Some(vec![DEFAULT_HARNESS.into()]));
        (f, v)
    }

    // E-20: only human-origin prompts renew, acquire or take over; others are Ignored in every state.
    #[test]
    fn e20_only_human_origin_counts_in_every_state() {
        let states: Vec<(&str, Folded, View)> = {
            let (f1, v1) = held_s1(MIN);
            let f2 = fold(&[]);
            let v2 = view("p", &f2, None, 0, P);
            let f3 = fold(&[acq_full(1, 1, "s1", 0, "human", false, "c1")]);
            let v3 = view("p", &f3, Some(&live("s1", "c1", 0, LEASE_MS as i64)), LEASE_MS + 1, P);
            let f4 = fold(&[acq_full(1, 1, "s1", 0, "human", true, "c1")]);
            let v4 = view("p", &f4, Some(&live("s1", "c1", 0, QUIET_MS as i64)), MIN, P);
            vec![("held", f1, v1), ("vacant", f2, v2), ("lapsed", f3, v3), ("provisional", f4, v4)]
        };
        for (name, f, v) in &states {
            for who in ["s1", "s2"] {
                for o in [Some(Origin::Automated), Some(Origin::Unknown), None] {
                    let p = plan_reporting(v, f, &h(who), &req(ClaimKind::Prompt, o), Some(0));
                    assert_eq!(p, Plan::Ignored, "{name}/{who}/{o:?} must change nothing");
                }
                let p = plan_reporting(v, f, &h(who), &human(), Some(0));
                assert_ne!(p, Plan::Ignored, "{name}/{who}: a human prompt counts");
            }
        }
    }

    // E-21: tool activity never renews (any state); SessionStart takes only a VACANT lease, provisionally.
    #[test]
    fn e21_tool_never_renews_and_session_start_is_provisional() {
        let (f, v) = held_s1(MIN);
        assert!(matches!(plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::Tool, Some(Origin::Human)), Some(0)), Plan::AnswerOnly { .. }));
        let f0 = fold(&[]);
        let v0 = view("p", &f0, None, 0, P);
        assert!(matches!(plan_reporting(&v0, &f0, &h("s1"), &req(ClaimKind::Tool, Some(Origin::Human)), None), Plan::AnswerOnly { .. }));
        assert_eq!(plan_reporting(&v0, &f0, &h("s1"), &req(ClaimKind::SessionStart, None), None), Plan::TakeProvisional);
        // SessionStart on a held lease, its own or another's: answer only (FCW-57).
        assert!(matches!(plan_reporting(&v, &f, &h("s2"), &req(ClaimKind::SessionStart, None), Some(0)), Plan::AnswerOnly { .. }));
        assert!(matches!(plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::SessionStart, None), Some(0)), Plan::AnswerOnly { .. }));
        // The provisional take reads back as `provisional`, never `held`.
        let fp = fold(&[acq_full(1, 1, "s1", 0, "human", true, "c1")]);
        let vp = view("p", &fp, Some(&live("s1", "c1", 0, QUIET_MS as i64)), 1_000, P);
        assert_eq!((vp.state, vp.provisional), ("provisional", true));
        // 100 tool events over 20 simulated minutes: none renews, so the lease lapses at 15 min.
        let mut renewals = 0;
        for i in 0..100u64 {
            let now = i * 12_000;
            let vv = view("p", &f, Some(&live("s1", "c1", 0, LEASE_MS as i64)), now, P);
            if !matches!(plan_reporting(&vv, &f, &h("s1"), &req(ClaimKind::Tool, Some(Origin::Human)), Some(0)), Plan::AnswerOnly { .. }) {
                renewals += 1;
            }
        }
        assert_eq!(renewals, 0);
        assert_eq!(view("p", &f, Some(&live("s1", "c1", 0, LEASE_MS as i64)), 20 * MIN, P).state, "lapsed");
    }

    // E-22: the switch is per harness, set only by an operator record; claims never set it.
    #[test]
    fn e22_origin_reporting_is_per_harness_and_only_an_operator_record_sets_it() {
        let claims_only = vec![acq_full(1, 1, "s1", 0, "human", false, "c1")];
        assert!(reporting_harnesses(&claims_only).is_empty(), "a claim with an origin never switches a harness");
        let mut cfg = vec![Logged { offset: 0, event: Event::OriginReporting { harness: "h2".into(), on: true, by: "op".into(), at_ms: 1 }, sender: None }];
        assert_eq!(reporting_harnesses(&cfg), vec!["h2".to_string()]);
        cfg.push(Logged { offset: 1, event: Event::OriginReporting { harness: "h2".into(), on: false, by: "op".into(), at_ms: 2 }, sender: None });
        assert!(reporting_harnesses(&cfg).is_empty(), "newest record per harness wins");
        // In a reporting harness an absent origin is unknown: ignored (FCW-53).
        let (f, v) = held_s1(MIN);
        assert_eq!(plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::Prompt, None), Some(0)), Plan::Ignored);
        // Old records (no origin fields) still parse.
        let old = r#"{"kind":"acquire","generation":1,"holder":{"session":"s","agent":"","host":"","pid":0},"claim_id":"c","at_ms":1,"reason":"first"}"#;
        let e: Event = serde_json::from_str(old).expect("pre-B-25 record parses");
        assert!(matches!(e, Event::Acquire { origin: None, provisional: false, .. }));
        // ...and a built record serialises without the new fields (old readers see the same bytes).
        let built = serde_json::to_string(&e).unwrap();
        assert!(!built.contains("provisional") && !built.contains("origin"), "{built}");
    }

    // E-23: provisional: taken at once by a human claim, lapses straight to VACANT, is main while it holds.
    #[test]
    fn e23_provisional_holder_never_blocks_and_lapses_to_vacant() {
        let f = fold(&[acq_full(1, 1, "s1", 0, "human", true, "c1")]);
        let r = live("s1", "c1", 0, QUIET_MS as i64);
        // (a) S2 human claim at 1 min: taken at once (no quiet, no cooldown).
        let v = view("p", &f, Some(&r), MIN, P);
        assert!(matches!(plan_reporting(&v, &f, &h("s2"), &human(), None), Plan::OverProvisional { .. }));
        // (c) am-i from S1 at 2 min: main.
        assert_eq!(am_i(&view("p", &f, Some(&r), 2 * MIN, P), "s1"), Verdict::Main);
        // (b) provisional at 4:59, VACANT (eligible at once) at 5:01.
        assert_eq!(view("p", &f, Some(&r), QUIET_MS - 1_000, P).state, "provisional");
        let v = view("p", &f, Some(&r), QUIET_MS + 1_000, P);
        assert_eq!(v.state, "vacant");
        assert_eq!(v.eligible_at_ms, Some(QUIET_MS));
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &human(), None), Plan::Take { reason: "after-provisional" });
        assert_eq!(plan_reporting(&v, &f, &h("s3"), &req(ClaimKind::SessionStart, None), None), Plan::TakeProvisional);
        // The holder's own first human prompt promotes it.
        let v = view("p", &f, Some(&r), MIN, P);
        assert!(matches!(plan_reporting(&v, &f, &h("s1"), &human(), None), Plan::Promote { .. }));
        // E-23.6 / FCW-203: a non-interactive SessionStart on a vacant lease takes nothing.
        let f0 = fold(&[]);
        let v0 = view("p", &f0, None, 0, P);
        let mut ss = req(ClaimKind::SessionStart, None);
        ss.interactive = Some(false);
        assert!(matches!(plan_reporting(&v0, &f0, &h("w"), &ss, None), Plan::AnswerOnly { .. }));
        ss.interactive = None;
        assert!(matches!(plan_reporting(&v0, &f0, &h("w"), &ss, None), Plan::AnswerOnly { .. }));
        // A provisional take never counts as a human prompt, even tagged human.
        assert_eq!(last_human(&f, Some(&[])), None);
        // A promote clears `provisional` in the fold.
        let mut log = vec![acq_full(1, 1, "s1", 0, "human", true, "c1")];
        log.push(Logged { offset: 2, event: Event::Promote { generation: 1, holder: h("s1"), claim_id: "c1".into(), at_ms: 100, origin: "human".into(), basis: None }, sender: Some("key-s1".into()) });
        let f = fold(&log);
        assert!(!f.current.as_ref().unwrap().provisional);
        assert_eq!(f.current.unwrap().human_at_ms, Some(100));
    }

    // E-24: fast takeover after 5 min of holder human silence, never at 4:59, never of a pinned holder.
    #[test]
    fn e24_fast_takeover_at_five_minutes_not_before_never_pinned() {
        let (f, v) = held_s1(QUIET_MS - 1_000);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &human(), v.last_human_at_ms), Plan::Held, "4:59 refused");
        let (f, v) = held_s1(QUIET_MS + 1_000);
        assert_eq!(v.last_human_at_ms, Some(0));
        match plan_reporting(&v, &f, &h("s2"), &human(), v.last_human_at_ms) {
            Plan::FastTakeover { from, age_s, .. } => assert_eq!((from.session.as_str(), age_s), ("s1", (QUIET_MS + 1_000) / 1000)),
            p => panic!("5:01 must be taken, got {p:?}"),
        }
        // Pinned S1 is never fast-taken (FCW-54).
        let mut log = vec![acq_full(1, 1, "s1", 0, "human", false, "c1")];
        log.push(Logged { offset: 2, event: Event::Pin { session: Some("s1".into()), by: "op".into(), reason: "r".into(), at_ms: 1 }, sender: None });
        let fp = fold(&log);
        let vp = view("p", &fp, Some(&live("s1", "c1", 0, LEASE_MS as i64)), 10 * MIN, P);
        assert_eq!(plan_reporting(&vp, &fp, &h("s2"), &human(), Some(0)), Plan::Held);
        // An automated claim at 30 min changes nothing.
        let (f, v) = held_s1(30 * MIN);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &req(ClaimKind::Prompt, Some(Origin::Automated)), Some(0)), Plan::Ignored);
    }

    // E-24.6 (b) / FCW-202: a Touch signed by another key, or with a stale claim id, never counts.
    #[test]
    fn e24_touch_counts_only_from_the_holders_key_and_claim() {
        let f = fold(&[acq_full(1, 1, "s1", 0, "human", false, "c1")]);
        let good = [touch(1, 1, "s1", "c1", 200_000, "key-s1")];
        assert_eq!(last_human(&f, Some(&good)), Some(200_000));
        let forged = [touch(1, 1, "s1", "c1", 200_000, "key-mallory")];
        assert_eq!(last_human(&f, Some(&forged)), Some(0), "another agent's key does not count");
        let stale = [touch(1, 1, "s1", "c-old", 200_000, "key-s1")];
        assert_eq!(last_human(&f, Some(&stale)), Some(0), "a stale claim id does not count");
        let old_gen = [touch(1, 0, "s1", "c1", 200_000, "key-s1")];
        assert_eq!(last_human(&f, Some(&old_gen)), Some(0), "an earlier generation does not count");
        // A Touch keeps the holder fresh against a fast takeover.
        let v = with_origin(view("p", &f, Some(&live("s1", "c1", 0, LEASE_MS as i64)), 200_000 + QUIET_MS - 1_000, P), &f, Some(&good), None);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &human(), v.last_human_at_ms), Plan::Held);
    }

    // FCW-56 (v0.2 margin while the hub has no receive stamp): another host needs 30 s more.
    #[test]
    fn e24_cross_host_margin_until_the_hub_stamps_receive_time() {
        let (f, v) = held_s1(QUIET_MS + 1_000);
        let mut far = h("s2");
        far.host = "other-host".into();
        assert_eq!(plan_reporting(&v, &f, &far, &human(), Some(0)), Plan::Held);
        let (f, v) = held_s1(QUIET_MS + CROSS_HOST_MARGIN_MS + 1_000);
        assert!(matches!(plan_reporting(&v, &f, &far, &human(), Some(0)), Plan::FastTakeover { .. }));
    }

    // E-26: lapse at 15 min whatever the agent does; human prompt resumes (S1) or takes at once (S2).
    #[test]
    fn e26_lapse_without_human_renewal_then_resume_or_take_at_once() {
        let f = fold(&[acq_full(1, 1, "s1", 0, "human", false, "c1")]);
        let r = live("s1", "c1", 0, LEASE_MS as i64);
        // Busy agent: tool events and automated prompts every 30 s never renew.
        for i in 0..30u64 {
            let v = view("p", &f, Some(&r), i * 30_000, P);
            let t = plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::Tool, Some(Origin::Human)), Some(0));
            let a = plan_reporting(&v, &f, &h("s1"), &req(ClaimKind::Prompt, Some(Origin::Automated)), Some(0));
            assert!(matches!(t, Plan::AnswerOnly { .. }) && a == Plan::Ignored);
        }
        let v = view("p", &f, Some(&r), LEASE_MS + 1_000, P);
        assert_eq!(v.state, "lapsed");
        assert_eq!(plan_reporting(&v, &f, &h("s1"), &human(), None), Plan::Resume);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &human(), None), Plan::Take { reason: "takeover-after-lapse" }, "no quiet period");
        // The built plan for the same state still waits for the quiet period (NID for absent harnesses).
        assert!(matches!(plan_claim(&v, &f, "s2"), Plan::Wait { .. }));
    }

    // E-27: a doorbell line reported automated basis doorbell never renews or takes.
    #[test]
    fn e27_doorbell_line_never_renews_or_takes() {
        let mut r = req(ClaimKind::Prompt, Some(Origin::Automated));
        r.basis = Some("doorbell".into());
        let (f, v) = held_s1(30 * MIN);
        assert_eq!(plan_reporting(&v, &f, &h("s1"), &r, Some(0)), Plan::Ignored);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &r, Some(0)), Plan::Ignored);
    }

    // E-162 / FCW-55: an unreadable touch log means no fast takeover.
    #[test]
    fn e162_unreadable_touch_log_never_allows_fast_takeover() {
        let f = fold(&[acq_full(1, 1, "s1", 0, "human", false, "c1")]);
        assert_eq!(last_human(&f, None), None);
        let v = with_origin(view("p", &f, Some(&live("s1", "c1", 0, LEASE_MS as i64)), 14 * MIN, P), &f, None, None);
        assert_eq!(v.last_human_at_ms, None);
        assert_eq!(plan_reporting(&v, &f, &h("s2"), &human(), v.last_human_at_ms), Plan::Held);
        // A holder whose harness never reported origin has no human record: never fast-taken (NID).
        let fa = fold(&[acq_full(1, 1, "s1", 0, "absent", false, "c1")]);
        assert_eq!(last_human(&fa, Some(&[])), None);
    }

    // E-25: bounded model check of the lease state machine (D-4 (b), 6.13) over the plan
    // functions, executed against a simulated hub claim row.
    #[test]
    fn e25_model_check_lease_invariants() {
        let (states, violations) = model_check(40, 10_000, false);
        assert!(violations.is_empty(), "violations: {violations:?}");
        assert!(states > 100, "explored {states} abstract states (state, holder, generation, pin)");
    }

    // E-25.5 control: a mutant that lets an automated claim take a vacant/lapsed lease breaks
    // the non-human invariant (the checker can see it).
    #[test]
    fn e25_control_the_checker_catches_a_non_human_take() {
        let (_, violations) = model_check(40, 5_000, true);
        assert!(violations.iter().any(|v| v.contains("non-human")), "the mutant went unseen: {violations:?}");
    }

    /// A tiny simulated hub: log, offset-0 row, touch log; drives `plan_reporting` / `plan_claim`
    /// and checks the 5.11.d invariants after every step. Returns (distinct states, violations).
    fn model_check(steps: usize, walks: usize, mutant_automated_takes: bool) -> (usize, Vec<String>) {
        let sessions = ["s1", "s2", "s3"];
        let mut seen = std::collections::BTreeSet::new();
        let mut bad = vec![];
        let mut seed: u64 = 0x05ee_db25;
        let mut rnd = |n: u64| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) % n
        };
        for _ in 0..walks {
            let mut log: Vec<Logged> = vec![];
            let mut touches: Vec<Logged> = vec![];
            let mut row: Option<Row> = None;
            let mut now: u64 = 0;
            let mut next_claim = 0u64;
            let mut last_gen = 0u64;
            for _ in 0..steps {
                let f = fold(&log);
                let v0 = view("p", &f, row.as_ref(), now, P);
                let v = with_origin(v0, &f, Some(&touches), Some(vec![DEFAULT_HARNESS.into()]));
                seen.insert(format!("{}|{:?}|{}|{:?}", v.state, v.holder.as_ref().map(|h| h.session.clone()), v.generation, v.pinned));
                let before = (v.holder.as_ref().map(|h| h.session.clone()), v.generation, row.as_ref().map(|r| r.claimed_until));
                let who = sessions[rnd(3) as usize];
                let me = Holder { session: who.into(), agent: if who == "s3" { "agent-y".into() } else { "agent-x".into() }, host: "host".into(), pid: 7 };
                let action = rnd(10);
                let (origin, kind) = match action {
                    0..=3 => (Some(Origin::Human), ClaimKind::Prompt),
                    4 => (Some(Origin::Automated), ClaimKind::Prompt),
                    5 => (Some(Origin::Unknown), ClaimKind::Prompt),
                    6 => (Some(Origin::Human), ClaimKind::Tool),
                    7 => (None, ClaimKind::SessionStart),
                    8 => {
                        now += [30_000, 4 * MIN, 6 * MIN, 16 * MIN][rnd(4) as usize];
                        continue;
                    }
                    _ => {
                        if rnd(4) == 0 {
                            let pin = if rnd(2) == 0 { Some(who.to_string()) } else { None };
                            log.push(Logged { offset: log.len() as u64 + 1, event: Event::Pin { session: pin, by: "op".into(), reason: "m".into(), at_ms: now }, sender: None });
                        } else if am_i(&v, who) == Verdict::Main {
                            row = None;
                            log.push(Logged { offset: log.len() as u64 + 1, event: Event::Release { generation: v.generation, holder: me.clone(), at_ms: now }, sender: None });
                        }
                        continue;
                    }
                };
                let r = ClaimRequest { kind, origin, basis: None, interactive: Some(true), harness: DEFAULT_HARNESS.into() };
                let mut plan = plan_reporting(&v, &f, &me, &r, v.last_human_at_ms);
                if mutant_automated_takes && plan == Plan::Ignored && v.holder.is_none() {
                    plan = Plan::Take { reason: "mutant" };
                }
                let key = Some(format!("key-{who}"));
                let mut acquire = |log: &mut Vec<Logged>, row: &mut Option<Row>, reason: &str, generation: u64, prov: bool, ttl: u64| {
                    next_claim += 1;
                    let cid = format!("c{next_claim}");
                    *row = Some(Row { claim_id: cid.clone(), claimer: me.claimer(), claimed_at: now as i64, claimed_until: (now + ttl) as i64 });
                    let o = if plan_is_human(origin) { "human" } else { "automated" };
                    log.push(Logged {
                        offset: log.len() as u64 + 1,
                        event: Event::Acquire { generation, holder: me.clone(), claim_id: cid, at_ms: now, reason: reason.into(), from: None, lapsed_at_ms: None, obligations: vec![], origin: Some(o.into()), basis: None, provisional: prov, holder_human_age_s: None },
                        sender: key.clone(),
                    });
                };
                match plan {
                    Plan::Take { reason } => acquire(&mut log, &mut row, reason, f.generation + 1, false, LEASE_MS),
                    Plan::TakeProvisional => acquire(&mut log, &mut row, "provisional", f.generation + 1, true, QUIET_MS),
                    Plan::Resume => acquire(&mut log, &mut row, "resume", f.generation.max(1), false, LEASE_MS),
                    Plan::PinOverride { .. } | Plan::OverProvisional { .. } | Plan::FastTakeover { .. } => {
                        acquire(&mut log, &mut row, "override", f.generation + 1, false, LEASE_MS)
                    }
                    Plan::Promote { claim_id } => {
                        log.push(Logged { offset: log.len() as u64 + 1, event: Event::Promote { generation: v.generation, holder: me.clone(), claim_id, at_ms: now, origin: "human".into(), basis: None }, sender: key.clone() });
                        if let Some(r) = row.as_mut() {
                            r.claimed_until = (now + LEASE_MS) as i64;
                        }
                    }
                    Plan::TouchRenew { claim_id } => {
                        touches.push(touch(touches.len() as u64, v.generation, who, &claim_id, now, &format!("key-{who}")));
                        if let Some(r) = row.as_mut() {
                            r.claimed_until = (now + LEASE_MS) as i64;
                        }
                    }
                    _ => {}
                }
                let f2 = fold(&log);
                let v2 = view("p", &f2, row.as_ref(), now, P);
                let after = (v2.holder.as_ref().map(|h| h.session.clone()), v2.generation, row.as_ref().map(|r| r.claimed_until));
                if v2.generation < last_gen {
                    bad.push(format!("generation decreased {last_gen} -> {}", v2.generation));
                }
                last_gen = v2.generation;
                if !plan_is_human(origin) && kind == ClaimKind::Prompt && before != after {
                    bad.push(format!("non-human claim by {who} changed {before:?} -> {after:?}"));
                }
                if kind == ClaimKind::Tool && before != after {
                    bad.push(format!("tool activity changed {before:?} -> {after:?}"));
                }
                if before.0 != after.0 && after.0.is_some() && f2.current.as_ref().map(|c| &c.holder.session) != after.0.as_ref() {
                    bad.push("holder change without an acquire record".into());
                }
                if let (Some(old), Some(new)) = (&before.0, &after.0)
                    && old != new
                    && v.pinned.as_deref() == Some(old.as_str())
                    && v.pinned.as_deref() != Some(who)
                {
                    bad.push(format!("pinned holder {old} displaced by {who}"));
                }
                // 6.13.3 bound: a human prompt by another session, more than quiet (+0 margin,
                // same host) after the holder's last human prompt, gets main at that prompt.
                if plan_is_human(origin)
                    && kind == ClaimKind::Prompt
                    && v.holder.as_ref().is_some_and(|h| h.session != who)
                    && !v.provisional
                    && v.pinned.is_none()
                    && v.last_human_at_ms.is_some_and(|t| now.saturating_sub(t) > QUIET_MS)
                    && after.0.as_deref() != Some(who)
                {
                    bad.push(format!("6.13.3 bound: {who} not main after holder silence"));
                }
                if bad.len() > 20 {
                    return (seen.len(), bad);
                }
            }
        }
        (seen.len(), bad)
    }

    fn plan_is_human(o: Option<Origin>) -> bool {
        o == Some(Origin::Human)
    }

    // E-31.5 control: the report generator on a seeded record set.
    #[test]
    fn e31_report_counts_renewals_by_basis_and_flags_non_human_holder_changes() {
        let touches = [touch(1, 1, "s1", "c1", 1, "k"), touch(2, 1, "s1", "c1", 2, "k"), {
            let mut t = touch(3, 1, "s1", "c1", 3, "k");
            if let Event::Touch { basis, .. } = &mut t.event {
                *basis = Some("inferred".into());
            }
            t
        }];
        let ignored = [Logged { offset: 1, event: Event::Ignored { session: "s2".into(), origin: "automated".into(), basis: Some("doorbell".into()), hook: "prompt".into(), harness: "h".into(), at_ms: 1 }, sender: None }];
        let clean = [acq_full(1, 1, "s1", 0, "human", false, "c1")];
        let r = renewal_report(&clean, &touches, &ignored);
        assert_eq!(r["counted_renewals"], 3);
        assert_eq!(r["by_basis"]["pty-input"]["numerator"], 2);
        assert_eq!(r["by_basis"]["inferred"]["denominator"], 3);
        assert_eq!(r["ignored"]["automated/doorbell"], 1);
        assert_eq!(r["verdict"], "PASS");
        let bad = [acq_full(1, 1, "s1", 0, "automated", false, "c1")];
        assert_eq!(renewal_report(&bad, &touches, &ignored)["verdict"], "FAIL");
    }

    // E-84 (R3 prompt origin): the published parity scenario, decided by TermLink's lease.
    // AEF's adapter (P-44, B-29) runs the same file; a doorbell line reported `human` must
    // fail the parity run (the negative control is marked in the file).
    #[test]
    fn e84_prompt_origin_parity_scenario_matches_the_lease() {
        let doc: Value = serde_json::from_str(include_str!("../../../tests/fixtures/b25-parity/r3-prompt-origin.json")).expect("scenario parses");
        let cases = doc["cases"].as_array().expect("cases");
        assert!(cases.len() >= 8);
        for c in cases {
            let name = c["name"].as_str().unwrap();
            let (f, v) = match c["state"].as_str().unwrap() {
                "held-by-other-fresh" => held_s1(MIN),
                "held-by-other-silent" => held_s1(QUIET_MS + 1_000),
                "vacant" => {
                    let f = fold(&[]);
                    let v = view("p", &f, None, 0, P);
                    (f, v)
                }
                "held-by-me" => held_s1(MIN),
                s => panic!("unknown state {s}"),
            };
            let who = if c["state"] == "held-by-me" { "s1" } else { "s2" };
            let r = ClaimRequest {
                kind: ClaimKind::parse(c["hook"].as_str().unwrap()).unwrap(),
                origin: c["origin"].as_str().and_then(Origin::parse),
                basis: c["basis"].as_str().map(str::to_string),
                interactive: c["interactive"].as_bool(),
                harness: DEFAULT_HARNESS.into(),
            };
            let got = match plan_reporting(&v, &f, &h(who), &r, v.last_human_at_ms) {
                Plan::Ignored => "ignored",
                Plan::AnswerOnly { .. } | Plan::Held => "unchanged",
                Plan::TakeProvisional => "provisional",
                Plan::TouchRenew { .. } | Plan::Promote { .. } => "renewed",
                Plan::FastTakeover { .. } => "fast-takeover",
                Plan::Take { .. } | Plan::Resume | Plan::OverProvisional { .. } | Plan::PinOverride { .. } => "acquired",
                p => panic!("{name}: unexpected plan {p:?}"),
            };
            assert_eq!(got, c["expect"].as_str().unwrap(), "{name}");
        }
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
