//! B-16 (T-3697, plan P-7): the doorbell D+ and the 300 s watchdog K-33.
//!
//! 06 AL-5 (the ring gate), AL-6 (re-checks, the SI-1 line, intent before the
//! keystrokes), AL-7 (the ring schedule: back-off, UNSUBMITTED, STALLED) and AL-8
//! (the watchdog, independent of READY). The decisions are pure functions over a
//! snapshot ([`decide`], [`Sched`], [`wd_eval`], [`choose`]); every effect goes
//! through [`Ports`], so the fixture suite drives the whole path on a simulated
//! clock and the daemon drives it on the real one ([`LivePorts`]).
//!
//! Operator rulings carried here: O-2 D+ (ring only on READY, no typing for 30 s
//! and no conversation, re-checked after the veto); F-1 ruling A (the watchdog
//! re-reads typing after the veto, 30 s before acting); F-20 ruling D (on macOS
//! the interim `ps` veto, status `watchdog: degraded (macOS, ps veto)`).
//!
//! Nothing working is switched off: the doorbell (and the watchdog with it) is
//! OFF per agent until the flag file `<store>/doorbell-on` exists ([`SWITCH_FLAG`]);
//! a flag that cannot be read is off. The built delivery switch (`delivery-off`)
//! stops it as well (SI-23, FCW-27).

use crate::meter::Conversation;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const TICK_MS: u64 = crate::deadline::TICK_MS;
/// V-17: no client input for this long before a normal ring or a watchdog line.
pub const V17_MS: u64 = 30_000;
/// V-18: the watchdog's silence interval.
pub const V18_MS: u64 = 300_000;
/// CR-6.38 [P]: attempts per message before it gets no further line (FCW-199).
pub const MAX_ATTEMPTS: u32 = 8;
/// 06 AL-9 / ARC4 6.9.4: a ring without evidence after this long is labelled.
pub const UNCONFIRMED_MS: u64 = 90_000;
/// READY older than this reads not-ready (FCW-21).
pub const READY_MAX_MS: u64 = 24 * 3600 * 1000;
/// The per-agent kill switch: the doorbell runs only while this file exists.
pub const SWITCH_FLAG: &str = "doorbell-on";
/// The ring log (OBS-9) and the H-11 rows, one JSON object per line, in the store.
pub const RING_LOG: &str = "ring-log.jsonl";
/// Admin-chain event of a watchdog row (OBS-12).
pub const WATCHDOG_EVENT: &str = "doorbell-watchdog";
/// Local chain stage of a typed line (OBS-10). Never HANDED_OVER (E-4).
pub const ATTEMPTED: &str = "ATTEMPTED";
pub const HANDOVER_UNCONFIRMED: &str = "HANDOVER_UNCONFIRMED";
/// The macOS interim veto's CPU threshold over the 1.5 s window [P] (ARC4 6.9.7.f).
pub const PS_CPU_MS: u64 = 30;
pub const DEGRADED_MACOS: &str = "degraded (macOS, ps veto)";
/// H-11 (d): a ring this close to a turn start counts.
pub const TURN_START_MS: u64 = 2_000;

/// The SI-1 line (06 AL-6 `compose`): one function shared with the session.
pub fn compose(ids: &[String], count: u64) -> Result<String, String> {
    termlink_session::handler::si1_line(ids, count).map_err(str::to_owned)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    Doorbell,
    Watchdog,
}

impl Via {
    pub fn name(self) -> &'static str {
        match self {
            Via::Doorbell => "doorbell",
            Via::Watchdog => "watchdog",
        }
    }
}

// ---------------------------------------------------------------- AL-7 schedule

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingState {
    Idle,
    Backoff,
    AwaitSubmit,
    Unsubmitted,
    Stalled,
}

impl RingState {
    pub fn name(self) -> &'static str {
        match self {
            RingState::Idle => "idle",
            RingState::Backoff => "backoff",
            RingState::AwaitSubmit => "await-submit",
            RingState::Unsubmitted => "unsubmitted",
            RingState::Stalled => "stalled",
        }
    }
}

/// The typed line waiting for its submit.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRing {
    pub ring_id: String,
    pub via: Via,
    pub mono: u64,
    pub ids: Vec<String>,
}

/// What a schedule tick found (AL-7 table).
#[derive(Debug, Clone, PartialEq)]
pub enum SchedEvent {
    /// A doorbell line not submitted within 1 tick: H-11 (e).
    Unsubmitted(PendingRing),
    /// A watchdog line not submitted within 1 tick (FCW-197): H-11 (e), attention.
    Stalled(PendingRing),
}

/// O(k): the tick offset of normal ring k after T0 (06 4.2).
pub fn offset(k: u32) -> u64 {
    match k {
        0 => 0,
        1 => 4,
        2 => 12,
        k => 12 + 20 * (u64::from(k) - 2),
    }
}

/// The per-session ring schedule (AL-7).
#[derive(Debug, Clone, PartialEq)]
pub struct Sched {
    pub state: RingState,
    pub k: u32,
    pub t0: u64,
    pub last: Option<u64>,
    pub pending: Option<PendingRing>,
    pub stalled_since: Option<u64>,
}

impl Default for Sched {
    fn default() -> Self {
        Sched { state: RingState::Idle, k: 0, t0: 0, last: None, pending: None, stalled_since: None }
    }
}

impl Sched {
    /// AL-7 `may_ring` (FCW-25).
    pub fn may_ring(&self, tick: u64) -> Result<(), &'static str> {
        match self.state {
            RingState::AwaitSubmit | RingState::Unsubmitted | RingState::Stalled => Err("unsubmitted"),
            RingState::Idle => Ok(()),
            RingState::Backoff => {
                if tick < self.t0 + offset(self.k + 1) {
                    Err("backoff")
                } else if self.last == Some(tick) {
                    Err("one-per-tick")
                } else {
                    Ok(())
                }
            }
        }
    }

    /// A line was typed (or may have been: `unknown`, FCW-36).
    pub fn on_typed(&mut self, tick: u64, mono: u64, via: Via, ring_id: &str, ids: &[String]) {
        let p = PendingRing { ring_id: ring_id.to_owned(), via, mono, ids: ids.to_vec() };
        match (via, self.state) {
            (_, RingState::Stalled) => return, // nothing is typed into a STALLED session
            (Via::Doorbell, RingState::Idle) | (Via::Watchdog, RingState::Idle) => {
                self.k = 0;
                self.t0 = tick;
            }
            (Via::Doorbell, RingState::Backoff) => self.k += 1,
            _ => {} // a watchdog line never moves k (CR-6.8)
        }
        self.last = Some(tick);
        self.pending = Some(p);
        self.state = RingState::AwaitSubmit;
    }

    /// A prompt was submitted at `mono` (any origin). Returns the line it ended, if any.
    pub fn on_submit(&mut self, mono: u64) -> Option<PendingRing> {
        match self.state {
            RingState::AwaitSubmit | RingState::Unsubmitted | RingState::Stalled => {
                let p = self.pending.take();
                // AWAIT_SUBMIT needs the submit after the line; the later states take any.
                if self.state == RingState::AwaitSubmit && p.as_ref().is_some_and(|p| mono < p.mono) {
                    self.pending = p;
                    return None;
                }
                self.state = RingState::Backoff;
                self.stalled_since = None;
                p
            }
            _ => None,
        }
    }

    /// A tick with no submit: AWAIT_SUBMIT older than 1 tick moves on.
    pub fn on_tick(&mut self, mono: u64) -> Option<SchedEvent> {
        if self.state != RingState::AwaitSubmit {
            return None;
        }
        let p = self.pending.clone()?;
        if mono.saturating_sub(p.mono) <= TICK_MS {
            return None;
        }
        match p.via {
            Via::Doorbell => {
                self.state = RingState::Unsubmitted;
                Some(SchedEvent::Unsubmitted(p))
            }
            Via::Watchdog => {
                self.state = RingState::Stalled;
                self.stalled_since = Some(mono);
                Some(SchedEvent::Stalled(p))
            }
        }
    }

    /// Progress: the mail was handed over or none is pending (not from STALLED).
    pub fn on_progress(&mut self) {
        if self.state != RingState::Stalled {
            *self = Sched::default();
        }
    }

    /// A new session (session_start) or NOT_RUNNING: a new input buffer.
    pub fn on_session_start(&mut self) {
        *self = Sched::default();
    }

    /// The tick of the next normal ring (OBS-11), if one can come.
    pub fn next_tick(&self) -> Option<u64> {
        match self.state {
            RingState::Idle => Some(0),
            RingState::Backoff => Some(self.t0 + offset(self.k + 1)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------- AL-5 gate

/// One pending message as the gate sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct Msg {
    pub id: String,
    pub conv: String,
    pub urgent: bool,
    pub stored_mono: u64,
    pub last_pull_mono: Option<u64>,
    pub attempts: u32,
    /// The exact session its conversation is bound to (P-5), if any.
    pub bound: Option<String>,
}

/// The READY record as the doorbell may read it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadyIn {
    pub state: String,
    pub seq: u64,
    /// Received by this sidecar process (FCW-6 / FCW-21).
    pub current_epoch: bool,
    pub rx_mono: u64,
}

/// S1 of a session.
#[derive(Debug, Clone, PartialEq)]
pub enum S1In {
    Unknown,
    /// The last client input (None: none since launch).
    Known(Option<u64>),
}

/// The gate's inputs for one session on one tick (one snapshot).
pub struct GateIn<'a> {
    pub now: u64,
    pub pending: &'a [Msg],
    /// Does this session get role mail (main, or no lease)? (AL-12)
    pub role_mail: bool,
    /// Does it get the mail of conversations bound to it? (P-5; not under withhold-all)
    pub bound_mail: bool,
    pub session: &'a str,
    pub pty_launched: bool,
    pub harness_registered: bool,
    pub ready: Option<&'a ReadyIn>,
    pub s1: &'a S1In,
    pub conversation: &'a Conversation,
    pub schedule: Result<(), &'static str>,
}

/// A gate decision (OBS-8).
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Ring { ids: Vec<String>, convs: Vec<String>, count: u64, ready_seq: u64 },
    Hold(&'static str),
}

impl Decision {
    pub fn outcome(&self) -> String {
        match self {
            Decision::Ring { .. } => "ring".into(),
            Decision::Hold(r) => format!("hold:{r}"),
        }
    }
}

/// The messages that may be rung (or fired) for `session`: AL-5 step 1.
pub fn eligible<'a>(pending: &'a [Msg], session: &str, role_mail: bool, bound_mail: bool) -> Vec<&'a Msg> {
    pending
        .iter()
        .filter(|m| m.attempts < MAX_ATTEMPTS)
        .filter(|m| match &m.bound {
            Some(s) if s == session => bound_mail || role_mail,
            Some(_) => role_mail, // main reads everything, as built (P-5)
            None => role_mail,
        })
        .collect()
}

/// Order the hand-over way (urgent first, then oldest) and take the first 4.
fn first_four(mut p: Vec<&Msg>) -> (Vec<String>, Vec<String>, u64) {
    p.sort_by_key(|m| (!m.urgent, m.stored_mono, m.id.clone()));
    let count = p.len() as u64;
    let take: Vec<&Msg> = p.into_iter().take(4).collect();
    (take.iter().map(|m| m.id.clone()).collect(), take.iter().map(|m| m.conv.clone()).collect(), count)
}

/// AL-5 `decide`: the four D+ conditions plus the schedule, in the ruled order.
pub fn decide(g: &GateIn) -> Decision {
    let p = eligible(g.pending, g.session, g.role_mail, g.bound_mail);
    // FCW-20: the hook path goes first.
    let p: Vec<&Msg> = p
        .into_iter()
        .filter(|m| g.now.saturating_sub(m.stored_mono) >= TICK_MS && m.last_pull_mono.is_none_or(|t| g.now.saturating_sub(t) >= TICK_MS))
        .collect();
    if p.is_empty() {
        return Decision::Hold("nothing");
    }
    if !g.pty_launched || !g.harness_registered {
        return Decision::Hold("not-a-termlink-pty"); // FCW-24
    }
    let Some(r) = g.ready.filter(|r| r.state == "READY" && r.current_epoch && g.now.saturating_sub(r.rx_mono) <= READY_MAX_MS) else {
        return Decision::Hold("not-ready"); // FCW-21
    };
    match g.s1 {
        S1In::Unknown => return Decision::Hold("typing"), // FCW-22
        S1In::Known(Some(at)) if g.now.saturating_sub(*at) < V17_MS => return Decision::Hold("typing"),
        S1In::Known(_) => {}
    }
    let p = match g.conversation {
        Conversation::None => p,
        Conversation::Operator | Conversation::Unknown(_) => return Decision::Hold("conversation"), // FCW-23
        Conversation::Agent(c) => {
            let p: Vec<&Msg> = p.into_iter().filter(|m| c.contains(&m.conv)).collect();
            if p.is_empty() {
                return Decision::Hold("conversation");
            }
            p
        }
    };
    if let Err(why) = g.schedule {
        return Decision::Hold(why); // FCW-25
    }
    let (ids, convs, count) = first_four(p);
    Decision::Ring { ids, convs, count, ready_seq: r.seq }
}

/// FCW-26 / FCW-198: of several, the session with the largest last activity;
/// ties (and no activity at all) the lowest session id.
pub fn choose(cands: &[(String, Option<u64>)]) -> Option<usize> {
    (0..cands.len()).max_by(|&a, &b| cands[a].1.cmp(&cands[b].1).then_with(|| cands[b].0.cmp(&cands[a].0)))
}

// ---------------------------------------------------------------- AL-8 watchdog

#[derive(Debug, Clone, PartialEq)]
pub enum Wd {
    Quiet,
    Unavailable(String),
    Held(&'static str),
    Counting,
    Due,
}

pub struct WdIn {
    pub now: u64,
    pub has_pending: bool,
    pub pty_launched: bool,
    pub s1_known: bool,
    pub s2_known: bool,
    pub stalled: bool,
    pub last_activity: Option<u64>,
    pub last_typed: Option<u64>,
    pub last_fire: Option<u64>,
    /// When the doorbell first saw the session: silence counts from here at most.
    pub baseline: u64,
}

/// AL-8 `watchdog_tick` for one session. READY and the conversation are never inputs.
pub fn wd_eval(w: &WdIn) -> Wd {
    if !w.has_pending {
        return Wd::Quiet;
    }
    if !w.pty_launched {
        return Wd::Unavailable("not a TermLink PTY session".into()); // FCW-39
    }
    if !w.s1_known {
        return Wd::Unavailable("S1 unknown".into());
    }
    if !w.s2_known {
        return Wd::Unavailable("S2 unknown".into());
    }
    if w.stalled {
        return Wd::Held("stalled"); // FCW-197
    }
    let since = [w.last_activity, w.last_typed, w.last_fire, Some(w.baseline)].into_iter().flatten().max().unwrap_or(w.baseline);
    if w.now.saturating_sub(since) < V18_MS {
        return Wd::Counting; // FCW-38
    }
    Wd::Due
}

// ---------------------------------------------------------------- veto

#[derive(Debug, Clone, PartialEq)]
pub enum Veto {
    Quiet,
    Active(String),
    Unmeasurable(String),
}

impl From<crate::activity::Verdict> for Veto {
    fn from(v: crate::activity::Verdict) -> Self {
        match v {
            crate::activity::Verdict::Active(s) => Veto::Active(s.to_owned()),
            crate::activity::Verdict::Quiet => Veto::Quiet,
            crate::activity::Verdict::Unmeasurable(w) => Veto::Unmeasurable(w),
        }
    }
}

/// One row of `ps -A -o pid=,ppid=,stat=,time=,args=`.
#[derive(Debug, Clone, PartialEq)]
pub struct PsRow {
    pub pid: u32,
    pub ppid: u32,
    pub stat: String,
    pub cpu_ms: u64,
    pub args: String,
}

/// `[[dd-]hh:]mm:ss[.cc]` to ms.
fn ps_time(t: &str) -> Option<u64> {
    let (days, rest) = match t.split_once('-') {
        Some((d, r)) => (d.parse::<u64>().ok()?, r),
        None => (0, t),
    };
    let (whole, frac) = match rest.split_once('.') {
        Some((w, f)) => (w, f),
        None => (rest, ""),
    };
    let mut secs = 0u64;
    for part in whole.split(':') {
        secs = secs * 60 + part.parse::<u64>().ok()?;
    }
    let cs: u64 = if frac.is_empty() { 0 } else { format!("{frac:0<2}")[..2].parse().ok()? };
    Some((days * 86_400 + secs) * 1000 + cs * 10)
}

/// Parse `ps` output; `None` when no row parses (ps unusable).
pub fn parse_ps(text: &str) -> Option<Vec<PsRow>> {
    let rows: Vec<PsRow> = text
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let pid = it.next()?.parse().ok()?;
            let ppid = it.next()?.parse().ok()?;
            let stat = it.next()?.to_owned();
            let cpu_ms = ps_time(it.next()?)?;
            let args = it.collect::<Vec<_>>().join(" ");
            Some(PsRow { pid, ppid, stat, cpu_ms, args })
        })
        .collect();
    (!rows.is_empty()).then_some(rows)
}

fn ps_tree(rows: &[PsRow], root: u32) -> Vec<&PsRow> {
    let mut out: Vec<&PsRow> = rows.iter().filter(|r| r.pid == root).collect();
    let mut i = 0;
    while i < out.len() {
        let p = out[i].pid;
        let kids: Vec<&PsRow> = rows.iter().filter(|r| r.ppid == p && r.pid != root && !out.iter().any(|o| o.pid == r.pid)).collect();
        out.extend(kids);
        i += 1;
    }
    out
}

/// The macOS interim veto (06 AL-6 step 4, ruling D on F-20): two `ps` samples
/// 1.5 s apart. Active("child") when the harness tree holds a running child
/// (state R or D) or any child that is not one of the harness's own idle helpers
/// (`helpers`: argument prefixes from the registry entry); Active("cpu") when the
/// tree's CPU time grew by at least [`PS_CPU_MS`]; Unmeasurable when the root is gone.
pub fn ps_judge(a: &[PsRow], b: &[PsRow], root: u32, helpers: &[String]) -> Veto {
    let tb = ps_tree(b, root);
    if tb.is_empty() {
        return Veto::Unmeasurable(format!("harness pid {root} not in ps"));
    }
    for r in tb.iter().filter(|r| r.pid != root) {
        let running = r.stat.starts_with('R') || r.stat.starts_with('D');
        let helper = helpers.iter().any(|h| !h.is_empty() && r.args.starts_with(h.as_str()));
        if running || !helper {
            return Veto::Active("child".into());
        }
    }
    let cpu = |t: &[&PsRow]| t.iter().map(|r| r.cpu_ms).sum::<u64>();
    let ta = ps_tree(a, root);
    if cpu(&tb).saturating_sub(cpu(&ta)) >= PS_CPU_MS {
        return Veto::Active("cpu".into());
    }
    Veto::Quiet
}

/// Run the interim veto on this host (used where `/proc` is absent).
pub fn measure_ps(root: u32, helpers: &[String]) -> Veto {
    let run = || {
        let o = std::process::Command::new("ps").args(["-A", "-o", "pid=,ppid=,stat=,time=,args="]).stdin(std::process::Stdio::null()).output().ok()?;
        o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
    };
    let Some(a) = run().as_deref().and_then(parse_ps) else { return Veto::Unmeasurable("ps unusable".into()) };
    std::thread::sleep(std::time::Duration::from_millis(crate::activity::WINDOW_MS));
    let Some(b) = run().as_deref().and_then(parse_ps) else { return Veto::Unmeasurable("ps unusable".into()) };
    ps_judge(&a, &b, root, helpers)
}

// ---------------------------------------------------------------- AL-6 ports

/// The harness identity recorded at `session_start` (04 17.4.a, CR-6.19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Harness {
    pub pid: u32,
    pub exe: Option<String>,
    pub script: Option<String>,
}

/// One `session.input_activity` read: the foreground leader and the last client input.
#[derive(Debug, Clone, PartialEq)]
pub struct Fg {
    pub leader_pid: Option<u32>,
    pub exe: Option<String>,
    pub script: Option<String>,
    pub client_last: Option<u64>,
}

/// `is_harness`: the PTY's foreground leader is the recorded harness pid with the
/// same `(exe_realpath, script_path)` — never a process name (E-2.7).
pub fn is_harness(h: &Harness, fg: &Fg) -> bool {
    fg.leader_pid == Some(h.pid) && fg.exe.is_some() && fg.exe == h.exe && fg.script == h.script
}

/// What a notify call did (06 AL-6 step 10).
#[derive(Debug, Clone, PartialEq)]
pub enum Notified {
    Typed,
    Failed(String),
    /// Timed out: maybe typed (FCW-36).
    Unknown,
}

/// Every effect the doorbell has. [`LivePorts`] is the daemon's; tests fake it.
pub trait Ports {
    fn mono(&self) -> u64;
    /// The kill switch (SI-23): read now, never cached. Unreadable is off.
    fn switch_on(&self) -> bool;
    fn ready(&self, session: &str) -> Option<ReadyIn>;
    fn conversation(&self, session: &str, now: u64) -> Conversation;
    fn input_activity(&self, session: &str) -> Result<Fg, String>;
    /// The process-tree veto over the harness tree; `true` when the macOS `ps` veto measured.
    fn veto(&self, session: &str, harness_pid: u32) -> (Veto, bool);
    /// Append a ring-log row durably (OBS-9, H-11). For `via = watchdog` intents
    /// also the admin-chain row (FCW-42).
    fn append(&self, row: &Value, admin: bool) -> Result<(), String>;
    fn notify(&self, session: &str, ids: &[String], count: u64) -> Notified;
    /// The local ATTEMPTED stage of each id (OBS-10).
    fn attempted(&self, ids: &[String], via: Via, ring_id: &str);
    fn unconfirmed(&self, id: &str);
    fn attention(&self, kind: &str, key: &str, detail: &str);
    fn new_id(&self) -> String;
}

/// The snapshot a normal ring was decided on (AL-5 step 9).
#[derive(Debug, Clone)]
pub struct Snap {
    pub ready_seq: u64,
    pub convs: Vec<String>,
}

/// What `type_ring` did.
#[derive(Debug, Clone, PartialEq)]
pub enum RingOut {
    Held(String),
    Done { ring_id: String, notified: Notified },
}

fn conv_compatible(c: &Conversation, convs: &[String]) -> bool {
    match c {
        Conversation::None => true,
        Conversation::Agent(cs) => convs.iter().all(|x| cs.contains(x)),
        _ => false,
    }
}

/// 06 AL-6 `type_ring`: the re-checks in the ruled order, each failing closed,
/// the intent before the keystrokes and the outcome after. `row` is the
/// watchdog's admin row (via = watchdog) or `{}`.
#[allow(clippy::too_many_arguments)]
pub fn type_ring(p: &dyn Ports, session: &str, h: &Harness, ids: &[String], count: u64, snap: Option<&Snap>, via: Via, mut row: Value) -> (RingOut, Value) {
    let t0 = p.mono();
    let mut checks: Vec<Value> = vec![];
    let held = |checks: &mut Vec<Value>, why: &str, row: &mut Value| {
        checks.push(json!(format!("held:{why}")));
        row["checks"] = json!(checks);
        row["outcome"] = json!(format!("held:{why}"));
        (RingOut::Held(why.to_owned()), row.clone())
    };
    // 1 the kill switch (FCW-27)
    if !p.switch_on() {
        return held(&mut checks, "kill-switch", &mut row);
    }
    checks.push(json!("kill-switch:on"));
    let ready_ok = |p: &dyn Ports| match (p.ready(session), snap) {
        (Some(r), Some(s)) => r.state == "READY" && r.seq == s.ready_seq && r.current_epoch,
        _ => false,
    };
    // 2 READY re-read (doorbell only)
    if via == Via::Doorbell {
        if !ready_ok(p) {
            return held(&mut checks, "ready", &mut row); // FCW-28
        }
        checks.push(json!("ready:ok"));
    }
    // 3 foreground and typing
    let typing = |fg: &Fg, now: u64| fg.client_last.is_some_and(|c| now.saturating_sub(c) < V17_MS);
    match p.input_activity(session) {
        Ok(fg) if is_harness(h, &fg) => {
            if typing(&fg, p.mono()) {
                return held(&mut checks, "typing", &mut row); // FCW-31
            }
        }
        _ => return held(&mut checks, "foreground", &mut row), // FCW-29
    }
    checks.push(json!("foreground:harness"));
    // 4 the process-tree veto
    let (v, ps) = p.veto(session, h.pid);
    // OBS-15 (ruling D): the macOS interim veto measured; the engine shows `degraded`.
    row["veto_by"] = json!(if ps { "ps" } else { "proc" });
    match v {
        Veto::Active(sig) => return held(&mut checks, &format!("veto:{sig}"), &mut row), // FCW-30
        Veto::Unmeasurable(_) if via == Via::Watchdog => return held(&mut checks, "veto-unmeasurable", &mut row), // FCW-41
        Veto::Unmeasurable(_) => checks.push(json!("veto:unmeasurable(no veto, readiness decides)")),
        Veto::Quiet => checks.push(json!("veto:quiet")),
    }
    // 5 after the veto window, the re-reads (CR-6.1, CR-6.2)
    match p.input_activity(session) {
        Ok(fg) if is_harness(h, &fg) => {
            if typing(&fg, p.mono()) {
                return held(&mut checks, "typing", &mut row);
            }
        }
        _ => return held(&mut checks, "foreground", &mut row),
    }
    if via == Via::Doorbell {
        if !ready_ok(p) {
            return held(&mut checks, "ready", &mut row);
        }
        if !conv_compatible(&p.conversation(session, p.mono()), &snap.map(|s| s.convs.clone()).unwrap_or_default()) {
            return held(&mut checks, "conversation", &mut row); // FCW-32
        }
    }
    checks.push(json!("reread:ok"));
    // 6 the line
    if let Err(e) = compose(ids, count) {
        return held(&mut checks, &format!("compose:{e}"), &mut row); // FCW-33
    }
    // 7 stale checks, before any intent (CR-6.9)
    let t_last = p.mono();
    if t_last.saturating_sub(t0) > TICK_MS {
        return held(&mut checks, "stale-checks", &mut row);
    }
    // 8 the intent, durably, before the keystrokes (FCW-35, FCW-42)
    let ring_id = p.new_id();
    row["kind"] = json!(if via == Via::Watchdog { WATCHDOG_EVENT } else { "ring" });
    row["phase"] = json!("intent");
    row["ring_id"] = json!(ring_id);
    row["session"] = json!(session);
    row["ids"] = json!(ids);
    row["count"] = json!(count);
    row["via"] = json!(via.name());
    row["checks"] = json!(checks);
    row["t_first_check"] = json!(t0);
    row["t_last_check"] = json!(t_last);
    if let Err(e) = p.append(&row, via == Via::Watchdog) {
        row["outcome"] = json!("held:intent-unrecorded");
        row["error"] = json!(e);
        return (RingOut::Held("intent-unrecorded".into()), row);
    }
    // 9 the keystrokes
    let n = p.notify(session, ids, count);
    let now = p.mono();
    let outcome = match &n {
        Notified::Typed => json!("typed"),
        Notified::Failed(e) => json!(format!("failed({e})")),
        Notified::Unknown => json!("unknown"),
    };
    let out = json!({
        "kind": row["kind"], "phase": "outcome", "ring_id": ring_id, "session": session, "via": via.name(),
        "outcome": outcome, "check_to_keystroke_ms": now.saturating_sub(t_last),
        "first_check_to_keystroke_ms": now.saturating_sub(t0), "mono": now,
    });
    // 10 the outcome (a failed write is visible in status, the keystrokes happened)
    let _ = p.append(&out, via == Via::Watchdog);
    // 11 ATTEMPTED, never more (E-4)
    if !matches!(n, Notified::Failed(_)) {
        p.attempted(ids, via, &ring_id);
    }
    row["outcome"] = outcome;
    row["check_to_keystroke_ms"] = out["check_to_keystroke_ms"].clone();
    row["first_check_to_keystroke_ms"] = out["first_check_to_keystroke_ms"].clone();
    (RingOut::Done { ring_id, notified: n }, row)
}

// ---------------------------------------------------------------- the engine

/// What the daemon knows about one harness session on this tick.
#[derive(Debug, Clone)]
pub struct SessFacts {
    pub name: String,
    pub ended: bool,
    pub harness: Option<Harness>,
    pub registered: bool,
    pub pty_launched: bool,
    pub s1: S1In,
    pub s2_known: bool,
    /// (s1, s2, s3) last activity, mono.
    pub acts: (Option<u64>, Option<u64>, Option<u64>),
    pub ready: Option<ReadyIn>,
    pub conversation: Conversation,
    /// The last prompt submit: (rx mono, doorbell ids it named).
    pub last_submit: Option<(u64, Vec<String>)>,
    pub role_mail: bool,
    pub bound_mail: bool,
}

impl SessFacts {
    pub fn last_activity(&self) -> Option<u64> {
        let (a, b, c) = self.acts;
        [a, if self.s2_known { b } else { None }, c].into_iter().flatten().max()
    }
}

/// H-11 counters (a)..(f), each with numerator and denominator (RNU).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct H11 {
    /// (a) rings with a recorded check-to-keystroke interval, and the largest.
    pub a_rings: u64,
    pub a_max_ms: u64,
    /// (b) stray lines: submits whose doorbell ids were all already handed over.
    pub b_stray: u64,
    /// (c) vetoes while gate-passed (idle and ready), over normal ring attempts.
    pub c_vetoes: u64,
    pub c_attempts: u64,
    /// (d) rings within 2 s of a turn start not caused by the line.
    pub d_near_turn: u64,
    /// (e) typed lines not followed by a submit within 1 tick.
    pub e_unsubmitted: u64,
    /// (f) watchdog fires, and those that found READY busy.
    pub f_fires: u64,
    pub f_ready_busy: u64,
    /// The denominator: lines typed (doorbell + watchdog, typed or unknown).
    pub rings: u64,
}

#[derive(Debug, Clone, Default)]
struct SessState {
    sched: Sched,
    first_seen: u64,
    last_typed: Option<u64>,
    wd_last_fire: Option<u64>,
    /// Hold kinds already rowed in this silence episode (FCW-40).
    wd_rowed: BTreeSet<String>,
    wd: Option<Wd>,
    seen_submit: Option<u64>,
    last_decision: Option<String>,
    degraded: bool,
    unmeasurable_day: Option<u64>,
}

/// The doorbell of one agent.
#[derive(Default)]
pub struct Doorbell {
    tick: u64,
    sessions: BTreeMap<String, SessState>,
    /// id -> (mono of the ATTEMPTED stage, labelled)
    attempted: BTreeMap<String, (u64, bool)>,
    pub h11: H11,
    recent: VecDeque<Value>,
    append_failed: u64,
}

/// What one tick did (for tests and status).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TickReport {
    pub decisions: Vec<(String, String)>,
    pub rung: Option<(String, RingOut)>,
    pub watchdog: Option<(String, RingOut)>,
}

impl Doorbell {
    pub fn new() -> Self {
        Doorbell::default()
    }

    pub fn tick_no(&self) -> u64 {
        self.tick
    }

    pub fn sched(&self, session: &str) -> Option<&Sched> {
        self.sessions.get(session).map(|s| &s.sched)
    }

    fn row(&mut self, p: &dyn Ports, v: Value) {
        if p.append(&v, false).is_err() {
            self.append_failed += 1;
        }
        self.recent.push_back(v);
        while self.recent.len() > 64 {
            self.recent.pop_front();
        }
    }

    /// FCW-37 (06 AL-7 on_start, AL-8 6): rebuild from the ring log after a
    /// restart. The last line typed into a session with no outcome, or with outcome
    /// typed|unknown and no submit after it, leaves the session UNSUBMITTED (STALLED
    /// when it was the watchdog's); `last_typed` and the last fire are restored, so a
    /// restart never shortens the 300 s.
    pub fn restore(&mut self, log: &[Value], last_submit: &dyn Fn(&str) -> Option<u64>) {
        let mut intents: BTreeMap<String, (Value, Option<String>)> = BTreeMap::new();
        let mut sess_of: BTreeMap<String, String> = BTreeMap::new();
        for v in log {
            let (Some(rid), Some(phase)) = (v["ring_id"].as_str(), v["phase"].as_str()) else { continue };
            match phase {
                "intent" => {
                    if let Some(s) = v["session"].as_str() {
                        sess_of.insert(s.to_owned(), rid.to_owned());
                    }
                    intents.insert(rid.to_owned(), (v.clone(), None));
                }
                "outcome" => {
                    if let Some(e) = intents.get_mut(rid) {
                        e.1 = v["outcome"].as_str().map(str::to_owned);
                    }
                }
                _ => {}
            }
        }
        for (sess, rid) in sess_of {
            let Some((intent, outcome)) = intents.get(&rid) else { continue };
            let mono = intent["t_last_check"].as_u64().unwrap_or(0);
            let via = if intent["via"] == "watchdog" { Via::Watchdog } else { Via::Doorbell };
            let st = self.sessions.entry(sess.clone()).or_default();
            if outcome.as_deref().is_some_and(|o| o.starts_with("failed")) {
                continue;
            }
            st.last_typed = Some(mono);
            if via == Via::Watchdog {
                st.wd_last_fire = Some(mono);
            }
            if last_submit(&sess).is_some_and(|t| t >= mono) {
                continue;
            }
            let ids: Vec<String> = intent["ids"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default();
            st.sched.pending = Some(PendingRing { ring_id: rid.clone(), via, mono, ids });
            st.sched.state = if via == Via::Watchdog { RingState::Stalled } else { RingState::Unsubmitted };
            if via == Via::Watchdog {
                st.sched.stalled_since = Some(mono);
            }
        }
    }

    /// One tick: schedule bookkeeping, the normal ring (AL-5/AL-6), the watchdog
    /// (AL-8), the HANDOVER_UNCONFIRMED labels. `pending` is every message STORED,
    /// not handed over, without an open claim and not withheld (the deliverer's
    /// candidates); `handed` is the set of ids handed over so far (for stray lines).
    pub fn tick(&mut self, p: &dyn Ports, facts: &[SessFacts], pending: &[Msg], handed: &dyn Fn(&str) -> bool) -> TickReport {
        self.tick += 1;
        let tick = self.tick;
        let now = p.mono();
        let mut rep = TickReport::default();
        let pending_ids: BTreeSet<&str> = pending.iter().map(|m| m.id.as_str()).collect();

        // Schedule bookkeeping per session.
        for f in facts {
            let st = self.sessions.entry(f.name.clone()).or_insert_with(|| SessState { first_seen: now, ..Default::default() });
            if f.ended || f.ready.as_ref().is_some_and(|r| r.state == "NOT_RUNNING") {
                st.sched.on_session_start();
                continue;
            }
            let mut rows = vec![];
            if let Some((rx, dids)) = &f.last_submit
                && st.seen_submit.is_none_or(|s| *rx > s)
            {
                st.seen_submit = Some(*rx);
                if let Some(ended) = st.sched.on_submit(*rx) {
                    // (b): the line's ids were all handed over already or unknown.
                    if !dids.is_empty() && dids.iter().all(|i| handed(i) || !pending_ids.contains(i.as_str())) {
                        self.h11.b_stray += 1;
                        rows.push(json!({"h11": "b", "session": f.name, "ring_id": ended.ring_id, "mono": now}));
                    }
                    // (d): a turn began within 2 s of the line, not by the line.
                    if rx.abs_diff(ended.mono) <= TURN_START_MS && !ended.ids.iter().any(|i| dids.contains(i)) {
                        self.h11.d_near_turn += 1;
                        rows.push(json!({"h11": "d", "session": f.name, "ring_id": ended.ring_id, "mono": now}));
                    }
                }
            }
            match st.sched.on_tick(now) {
                Some(SchedEvent::Unsubmitted(pr)) => {
                    self.h11.e_unsubmitted += 1;
                    rows.push(json!({"h11": "e", "session": f.name, "ring_id": pr.ring_id, "via": "doorbell", "mono": now}));
                }
                Some(SchedEvent::Stalled(pr)) => {
                    self.h11.e_unsubmitted += 1;
                    rows.push(json!({"h11": "e", "session": f.name, "ring_id": pr.ring_id, "via": "watchdog", "mono": now}));
                    p.attention("session-stalled", &format!("{}:{}", f.name, pr.ring_id), &format!("doorbell: stalled since {now}: the watchdog line for session {} was not submitted; nothing more is typed until a submit or a restart", f.name));
                }
                None => {}
            }
            if eligible(pending, &f.name, f.role_mail, f.bound_mail).is_empty() {
                st.sched.on_progress();
            }
            for r in rows {
                self.row(p, r);
            }
        }

        // FCW-199 (CR-6.38): a message at MAX_ATTEMPTS gets no further line, and one
        // needs-attention entry; a hook pull still hands it over.
        for m in pending.iter().filter(|m| m.attempts >= MAX_ATTEMPTS) {
            p.attention("doorbell-max-attempts", &m.id, &format!("message {} reached {MAX_ATTEMPTS} delivery attempts without a hand-over: no further doorbell or watchdog line; a hook pull still hands it over", m.id));
        }
        // The normal ring: one decision per session, at most one ring per agent.
        let mut ring_cands: Vec<(String, Option<u64>)> = vec![];
        let mut decisions: BTreeMap<String, Decision> = BTreeMap::new();
        for f in facts.iter().filter(|f| !f.ended) {
            let st = &self.sessions[&f.name];
            let d = decide(&GateIn {
                now,
                pending,
                role_mail: f.role_mail,
                bound_mail: f.bound_mail,
                session: &f.name,
                pty_launched: f.pty_launched,
                harness_registered: f.registered && f.harness.is_some(),
                ready: f.ready.as_ref(),
                s1: &f.s1,
                conversation: &f.conversation,
                schedule: st.sched.may_ring(tick),
            });
            rep.decisions.push((f.name.clone(), d.outcome()));
            if matches!(d, Decision::Ring { .. }) {
                ring_cands.push((f.name.clone(), f.last_activity()));
            }
            decisions.insert(f.name.clone(), d);
        }
        for (s, o) in &rep.decisions {
            if let Some(st) = self.sessions.get_mut(s) {
                st.last_decision = Some(o.clone());
            }
        }
        if let Some(i) = choose(&ring_cands) {
            let name = ring_cands[i].0.clone();
            let f = facts.iter().find(|f| f.name == name).expect("candidate is a session");
            if let (Some(Decision::Ring { ids, convs, count, ready_seq }), Some(h)) = (decisions.get(&name), f.harness.as_ref()) {
                self.h11.c_attempts += 1;
                let snap = Snap { ready_seq: *ready_seq, convs: convs.clone() };
                let (out, row) = type_ring(p, &name, h, ids, *count, Some(&snap), Via::Doorbell, json!({}));
                self.note_veto(&name, &row);
                if let RingOut::Held(why) = &out {
                    if why.starts_with("veto:") {
                        self.h11.c_vetoes += 1;
                        self.row(p, json!({"h11": "c", "session": name, "held": why, "mono": now}));
                    }
                    if why == "intent-unrecorded" {
                        p.attention("ring-intent-unrecorded", &format!("{name}:{tick}"), &format!("the ring intent for {name} could not be written: nothing typed"));
                    }
                    self.recent.push_back(row);
                } else if let RingOut::Done { ring_id, notified } = &out {
                    self.after_typed(p, &name, tick, now, Via::Doorbell, ring_id, ids, notified, &row);
                }
                rep.rung = Some((name, out));
            }
        }

        // The watchdog: its own conditions, at most one fire per agent per tick.
        let mut due: Vec<(String, Option<u64>)> = vec![];
        for f in facts.iter().filter(|f| !f.ended) {
            let st = self.sessions.get_mut(&f.name).expect("seen above");
            let w = wd_eval(&WdIn {
                now,
                has_pending: !eligible(pending, &f.name, f.role_mail, f.bound_mail).is_empty(),
                pty_launched: f.pty_launched && f.registered && f.harness.is_some(),
                s1_known: f.s1 != S1In::Unknown,
                s2_known: f.s2_known,
                stalled: st.sched.state == RingState::Stalled,
                last_activity: f.last_activity(),
                last_typed: st.last_typed,
                last_fire: st.wd_last_fire,
                baseline: st.first_seen,
            });
            // A new silence episode starts whenever the session is not due/held.
            if matches!(w, Wd::Counting | Wd::Quiet) {
                st.wd_rowed.clear();
            }
            if w == Wd::Held("stalled") && st.wd_rowed.insert("stalled".into()) {
                let r = json!({"kind": WATCHDOG_EVENT, "session": f.name, "outcome": "held:stalled", "mono": now});
                let _ = p.append(&r, true);
            }
            if w == Wd::Due {
                due.push((f.name.clone(), f.last_activity()));
            }
            st.wd = Some(w);
        }
        if let Some(i) = choose(&due) {
            let name = due[i].0.clone();
            let f = facts.iter().find(|f| f.name == name).expect("due is a session");
            let mut el = eligible(pending, &name, f.role_mail, f.bound_mail);
            el.sort_by_key(|m| (!m.urgent, m.stored_mono, m.id.clone()));
            let count = el.len() as u64;
            let ids: Vec<String> = el.iter().take(4).map(|m| m.id.clone()).collect();
            let st = &self.sessions[&name];
            let since = [f.last_activity(), st.last_typed, st.wd_last_fire, Some(st.first_seen)].into_iter().flatten().max().unwrap_or(now);
            let ready_flag = match &f.ready {
                Some(r) if r.state == "READY" => "ready",
                Some(r) if r.state == "BUSY" => "busy",
                _ => "absent",
            };
            let (s1, s2, s3) = f.acts;
            let row = json!({
                "kind": WATCHDOG_EVENT, "session": name, "at": {"mono": now, "wall": crate::store::now_ms()},
                "silent_for_s": now.saturating_sub(since) / 1000, "ready_flag": ready_flag,
                "ready_age_s": f.ready.as_ref().map(|r| now.saturating_sub(r.rx_mono) / 1000),
                "last_activity": {"s1": s1, "s2": s2, "s3": s3}, "checks": [], "outcome": "pending", "message_ids": ids,
            });
            if let Some(h) = f.harness.as_ref() {
                let (out, row) = type_ring(p, &name, h, &ids, count, None, Via::Watchdog, row);
                self.note_veto(&name, &row);
                match &out {
                    RingOut::Held(why) => {
                        let st = self.sessions.get_mut(&name).expect("due is a session");
                        if why == "veto-unmeasurable" {
                            // FCW-41: an entry once per session per day, not a routine row.
                            let day = crate::store::now_ms() / 86_400_000;
                            if st.unmeasurable_day != Some(day) {
                                st.unmeasurable_day = Some(day);
                                p.attention("watchdog-unavailable", &format!("{name}:{day}"), &format!("watchdog: unavailable (veto unmeasurable) for session {name}"));
                            }
                        } else if why != "intent-unrecorded" && st.wd_rowed.insert(why.clone()) {
                            let _ = p.append(&row, true); // FCW-40: one row per hold kind per episode
                        }
                    }
                    RingOut::Done { ring_id, notified } => {
                        let st = self.sessions.get_mut(&name).expect("due is a session");
                        st.wd_last_fire = Some(now);
                        self.h11.f_fires += 1;
                        if ready_flag == "busy" {
                            self.h11.f_ready_busy += 1;
                            let day = crate::store::now_ms() / 86_400_000;
                            p.attention("ready-busy-while-silent", &format!("{name}:{day}"), &format!("READY flag busy for {} s while nothing happened (session {name})", now.saturating_sub(since) / 1000));
                        }
                        self.row(p, json!({"h11": "f", "session": name, "ring_id": ring_id, "ready_flag": ready_flag, "mono": now}));
                        self.after_typed(p, &name, tick, now, Via::Watchdog, ring_id, &ids, notified, &row);
                    }
                }
                rep.watchdog = Some((name, out));
            }
        }

        // HANDOVER_UNCONFIRMED: a typed line without evidence after 90 s (E-4).
        self.attempted.retain(|id, _| pending_ids.contains(id.as_str()) && !handed(id));
        for (id, (at, labelled)) in self.attempted.iter_mut() {
            if !*labelled && now.saturating_sub(*at) >= UNCONFIRMED_MS {
                *labelled = true;
                p.unconfirmed(id);
            }
        }
        rep
    }

    #[allow(clippy::too_many_arguments)]
    fn after_typed(&mut self, p: &dyn Ports, name: &str, tick: u64, now: u64, via: Via, ring_id: &str, ids: &[String], n: &Notified, row: &Value) {
        if !matches!(n, Notified::Failed(_)) {
            self.h11.rings += 1;
            if let Some(ms) = row["check_to_keystroke_ms"].as_u64() {
                self.h11.a_rings += 1;
                self.h11.a_max_ms = self.h11.a_max_ms.max(ms);
            }
            let st = self.sessions.get_mut(name).expect("a typed session is known");
            st.sched.on_typed(tick, now, via, ring_id, ids);
            st.last_typed = Some(now);
            for id in ids {
                self.attempted.entry(id.clone()).or_insert((now, false));
            }
            self.row(p, json!({"h11": "a", "session": name, "ring_id": ring_id, "via": via.name(), "check_to_keystroke_ms": row["check_to_keystroke_ms"], "first_check_to_keystroke_ms": row["first_check_to_keystroke_ms"], "mono": now}));
        }
        self.recent.push_back(row.clone());
        while self.recent.len() > 64 {
            self.recent.pop_front();
        }
    }

    /// Mark a session's watchdog as measured by the macOS `ps` veto (status line).
    pub fn set_degraded(&mut self, session: &str, on: bool) {
        self.sessions.entry(session.to_owned()).or_default().degraded = on;
    }

    fn note_veto(&mut self, session: &str, row: &Value) {
        match row["veto_by"].as_str() {
            Some("ps") => self.set_degraded(session, true),
            Some(_) => self.set_degraded(session, false),
            None => {}
        }
    }

    /// Status `doorbell` (OBS-11, OBS-15; PD-1: only the lines B-16 needs).
    pub fn status_view(&self, switch_on: bool) -> Value {
        let mut sessions = serde_json::Map::new();
        for (k, s) in &self.sessions {
            let wd = match &s.wd {
                _ if !switch_on => "off (doorbell switched off)".to_string(),
                Some(Wd::Unavailable(r)) => format!("unavailable ({r})"),
                _ if s.degraded => DEGRADED_MACOS.to_string(),
                Some(Wd::Held("stalled")) => "available (held: stalled)".into(),
                Some(_) => "available".into(),
                None => "unknown (no tick yet)".into(),
            };
            sessions.insert(
                k.clone(),
                json!({
                    "state": s.sched.state.name(), "k": s.sched.k, "next_tick": s.sched.next_tick(),
                    "stalled_since": s.sched.stalled_since, "watchdog": wd, "last_decision": s.last_decision,
                }),
            );
        }
        let h = &self.h11;
        json!({
            "switch": if switch_on { "on" } else { "off" },
            "tick": self.tick,
            "sessions": sessions,
            "h11": {
                "a_check_to_keystroke": {"rings": h.a_rings, "max_ms": h.a_max_ms},
                "b_stray": {"n": h.b_stray, "of_rings": h.rings},
                "c_vetoes": {"n": h.c_vetoes, "of_attempts": h.c_attempts},
                "d_near_turn_start": {"n": h.d_near_turn, "of_rings": h.rings},
                "e_unsubmitted": {"n": h.e_unsubmitted, "of_rings": h.rings},
                "f_watchdog": {"fires": h.f_fires, "ready_busy": h.f_ready_busy, "of_rings": h.rings},
            },
            "ring_log_append_failed": self.append_failed,
        })
    }
}

// ---------------------------------------------------------------- live ports

/// The kill switch: on only while `<store>/doorbell-on` exists and delivery is on.
/// Anything unreadable is off (E-161).
pub fn switch_on(store: &crate::store::Store) -> bool {
    std::fs::symlink_metadata(store.dir().join(SWITCH_FLAG)).is_ok_and(|m| m.is_file()) && crate::policy::delivery_enabled(store)
}

/// Append one JSON row to `path` and fsync it.
pub fn append_row(path: &Path, row: &Value) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).mode(0o600).open(path)?;
    writeln!(f, "{row}")?;
    f.sync_all()
}

/// Read the ring log (unparseable lines skipped).
pub fn read_log(path: &Path) -> Vec<Value> {
    std::fs::read_to_string(path).map(|t| t.lines().filter_map(|l| serde_json::from_str(l).ok()).collect()).unwrap_or_default()
}

/// The harness identity of `pid` from `/proc` (04 17.4.a).
pub fn harness_of(proc_root: &Path, pid: u32) -> Option<Harness> {
    let dir = proc_root.join(pid.to_string());
    let exe = std::fs::read_link(dir.join("exe")).ok().map(|p| p.to_string_lossy().into_owned())?;
    Some(Harness { pid, exe: Some(exe), script: termlink_session::input_meter::script_path(&dir) })
}

/// The registry of harnesses (04 17.4.a): `(exe_realpath, script_path)` entries.
/// An entry `exe` alone matches a native binary; `exe` with a script matches an
/// interpreter running that script. The default: the Claude Code native binary
/// (a file named `claude`) or `node` running Claude Code's `cli.js`.
pub fn registered(h: &Harness, extra: &[(String, Option<String>)]) -> bool {
    let Some(exe) = h.exe.as_deref() else { return false };
    if extra.iter().any(|(e, s)| e == exe && (s.is_none() || s.as_deref() == h.script.as_deref())) {
        return true;
    }
    let name = Path::new(exe).file_name().and_then(|n| n.to_str()).unwrap_or("");
    (name == "claude" && h.script.as_deref().is_none_or(|s| !s.ends_with(".js")))
        || (name == "node" && h.script.as_deref().is_some_and(|s| s.ends_with("/@anthropic-ai/claude-code/cli.js")))
}

/// Ancestors of `pid` (nearest first), at most 64.
pub fn ancestors(proc_root: &Path, pid: u32) -> Vec<u32> {
    let mut out = vec![];
    let mut cur = pid;
    for _ in 0..64 {
        let Ok(s) = std::fs::read_to_string(proc_root.join(cur.to_string()).join("stat")) else { break };
        let Some(rest) = s.rfind(')').map(|i| &s[i + 1..]) else { break };
        let Some(pp) = rest.split_whitespace().nth(1).and_then(|x| x.parse::<u32>().ok()) else { break };
        if pp <= 1 {
            break;
        }
        out.push(pp);
        cur = pp;
    }
    out
}

/// Session binding (S1 and `command.notify`): the TermLink session whose server
/// process is an ancestor of the harness pid. The sidecar finds it itself from
/// `/proc` and the session registry; no hook hands it a socket. A wrong binding
/// can only hold: every keystroke re-checks the foreground against the harness.
pub fn find_binding(proc_root: &Path, harness_pid: u32, regs: &[termlink_session::registration::Registration]) -> Option<(PathBuf, Option<String>, String)> {
    let anc = ancestors(proc_root, harness_pid);
    anc.iter().find_map(|a| {
        regs.iter().find(|r| r.pid == *a).and_then(|r| r.addr.as_unix_path().map(|p| (p.to_path_buf(), r.token_secret.clone(), r.id.to_string())))
    })
}

/// The daemon's doorbell: binds sessions, builds the facts, runs one tick.
pub struct Runner {
    pub agent: String,
    store: std::sync::Arc<crate::store::Store>,
    deliverer: std::sync::Arc<crate::deliver::Deliverer>,
    meter: std::sync::Arc<crate::meter::Meter>,
    rpc: std::sync::Arc<crate::meter::SessionRpc>,
    lease: Option<std::sync::Arc<crate::lease::Shared>>,
    /// The TermLink sessions directory to bind from (None: the default).
    sessions_dir: Option<PathBuf>,
    /// Extra registry entries `(exe_realpath, script_path)`.
    pub registry: Vec<(String, Option<String>)>,
    /// The macOS interim veto's idle-helper argument prefixes.
    pub helpers: Vec<String>,
    db: std::sync::Mutex<Doorbell>,
    harness: std::sync::Mutex<BTreeMap<String, Option<Harness>>>,
    restored: std::sync::atomic::AtomicBool,
}

fn lk<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

impl Runner {
    pub fn new(
        agent: &str,
        store: std::sync::Arc<crate::store::Store>,
        deliverer: std::sync::Arc<crate::deliver::Deliverer>,
        meter: std::sync::Arc<crate::meter::Meter>,
        rpc: std::sync::Arc<crate::meter::SessionRpc>,
        lease: Option<std::sync::Arc<crate::lease::Shared>>,
    ) -> Self {
        Runner {
            agent: agent.to_owned(),
            store,
            deliverer,
            meter,
            rpc,
            lease,
            sessions_dir: None,
            registry: vec![],
            helpers: vec![],
            db: std::sync::Mutex::new(Doorbell::new()),
            harness: std::sync::Mutex::new(BTreeMap::new()),
            restored: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Test seam: bind from this TermLink sessions directory.
    pub fn with_sessions_dir(mut self, d: PathBuf) -> Self {
        self.sessions_dir = Some(d);
        self
    }

    pub fn ring_log(&self) -> PathBuf {
        self.store.dir().join(RING_LOG)
    }

    /// Bind each harness session to its TermLink session (S1 and `command.notify`).
    fn bind_all(&self, facts: &[crate::meter::DoorbellFacts]) {
        let proc_root = Path::new("/proc");
        let mut regs: Option<Vec<termlink_session::registration::Registration>> = None;
        let mut hs = lk(&self.harness);
        for f in facts.iter().filter(|f| !f.ended) {
            let Some(pid) = f.harness_pid else {
                hs.remove(&f.name);
                continue;
            };
            let known = hs.get(&f.name).cloned().flatten();
            if known.as_ref().is_some_and(|h| h.pid == pid) && self.rpc.is_bound(&f.name) {
                continue;
            }
            hs.insert(f.name.clone(), harness_of(proc_root, pid));
            let regs = regs.get_or_insert_with(|| {
                match &self.sessions_dir {
                    Some(d) => termlink_session::manager::list_sessions_in(d, false),
                    None => termlink_session::manager::list_sessions(false),
                }
                .unwrap_or_default()
            });
            match find_binding(proc_root, pid, regs) {
                Some((sock, secret, tl)) => self.rpc.bind_secret(&f.name, &sock, &tl, secret),
                None => self.rpc.unbind(&f.name),
            }
        }
    }

    /// One doorbell tick (after the meter's). Nothing happens while the switch is off.
    pub fn tick(&self) -> Option<TickReport> {
        if !switch_on(&self.store) {
            return None;
        }
        let now = self.meter.now();
        let facts = self.meter.doorbell_facts(now);
        if !self.restored.swap(true, std::sync::atomic::Ordering::SeqCst) {
            let log = read_log(&self.ring_log());
            let subs: BTreeMap<String, u64> = facts.iter().filter_map(|f| f.last_submit.as_ref().map(|(t, _)| (f.name.clone(), *t))).collect();
            lk(&self.db).restore(&log, &|s| subs.get(s).copied());
        }
        self.bind_all(&facts);
        let hs = lk(&self.harness).clone();
        let now_ms = crate::store::now_ms();
        let withhold_all = self.deliverer.withhold_all();
        let sf: Vec<SessFacts> = facts
            .iter()
            .map(|f| {
                let harness = hs.get(&f.name).cloned().flatten().filter(|h| Some(h.pid) == f.harness_pid);
                let not_main = self.lease.as_ref().is_some_and(|l| l.not_main(&self.agent, &f.name, now_ms).is_some());
                SessFacts {
                    name: f.name.clone(),
                    ended: f.ended,
                    registered: harness.as_ref().is_some_and(|h| registered(h, &self.registry)),
                    harness,
                    pty_launched: self.rpc.is_bound(&f.name) && f.s1.is_some(),
                    s1: match f.s1 {
                        Some(c) => S1In::Known(c),
                        None => S1In::Unknown,
                    },
                    s2_known: f.s2_known,
                    acts: f.acts,
                    ready: f.ready.as_ref().map(|(r, cur)| ReadyIn { state: r.state.clone(), seq: r.seq, current_epoch: *cur, rx_mono: r.rx_mono }),
                    conversation: f.conversation.clone(),
                    last_submit: f.last_submit.clone(),
                    role_mail: !not_main,
                    bound_mail: !(not_main && withhold_all),
                }
            })
            .collect();
        let pending = self.deliverer.doorbell_pending(now_ms, now);
        let ports = LivePorts { r: self };
        let d = &self.deliverer;
        let rep = lk(&self.db).tick(&ports, &sf, &pending, &|id| d.is_handed(id));
        Some(rep)
    }

    pub fn status_view(&self) -> Value {
        lk(&self.db).status_view(switch_on(&self.store))
    }
}

/// [`Ports`] on the real sidecar.
pub struct LivePorts<'a> {
    r: &'a Runner,
}

impl Ports for LivePorts<'_> {
    fn mono(&self) -> u64 {
        self.r.meter.now()
    }
    fn switch_on(&self) -> bool {
        switch_on(&self.r.store)
    }
    fn ready(&self, session: &str) -> Option<ReadyIn> {
        self.r.meter.ready(session).map(|r| ReadyIn { state: r.state, seq: r.seq, current_epoch: true, rx_mono: r.rx_mono })
    }
    fn conversation(&self, session: &str, now: u64) -> Conversation {
        self.r.meter.conversation(session, now)
    }
    fn input_activity(&self, session: &str) -> Result<Fg, String> {
        let v = self
            .r
            .rpc
            .call(session, termlink_protocol::control::method::SESSION_INPUT_ACTIVITY, json!({}), crate::meter::S1_TIMEOUT_MS)
            .map_err(|e| e.unwrap_or_else(|| "input_activity timed out".into()))?;
        let fg = &v["foreground"];
        Ok(Fg {
            leader_pid: fg["leader_pid"].as_u64().and_then(|p| u32::try_from(p).ok()),
            exe: fg["exe_realpath"].as_str().map(str::to_owned),
            script: fg["script_path"].as_str().map(str::to_owned),
            client_last: v["sources"]["client"]["last_mono"].as_u64(),
        })
    }
    fn veto(&self, session: &str, harness_pid: u32) -> (Veto, bool) {
        let out = if Path::new("/proc/self").exists() {
            (Veto::from(crate::activity::measure_tree(harness_pid)), false)
        } else if cfg!(target_os = "macos") {
            (measure_ps(harness_pid, &self.r.helpers), true)
        } else {
            (Veto::Unmeasurable("no /proc and not macOS".into()), false)
        };
        let _ = session;
        out
    }
    fn append(&self, row: &Value, admin: bool) -> Result<(), String> {
        if admin {
            use sha2::{Digest, Sha256};
            let digest: String = Sha256::digest(row.to_string().as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
            let id = row["ring_id"].as_str().map(str::to_owned).unwrap_or_else(|| format!("wd-{}", self.mono()));
            let who = row["session"].as_str().unwrap_or("unknown");
            let reason = format!("phase={} outcome={} via=watchdog", row["phase"].as_str().unwrap_or("hold"), row["outcome"].as_str().unwrap_or("pending"));
            let extra = crate::store::LineExtra { reason: Some(reason), ..Default::default() };
            match self.r.store.append_event_unless(crate::api::ADMIN_CONV, WATCHDOG_EVENT, &id, who, &digest, extra, |_| false) {
                Ok(Some(_)) => {}
                Ok(None) => return Err("admin row not written".into()),
                Err(e) => return Err(format!("admin chain: {e}")),
            }
        }
        append_row(&self.r.ring_log(), row).map_err(|e| format!("ring log: {e}"))
    }
    fn notify(&self, session: &str, ids: &[String], count: u64) -> Notified {
        let n = match self.r.rpc.call(session, termlink_protocol::control::method::COMMAND_NOTIFY, json!({"ids": ids, "count": count}), 5_000) {
            Ok(v) if v["status"] == "injected" => Notified::Typed,
            Ok(v) => Notified::Failed(format!("not typed (status={})", v["status"])),
            Err(None) => Notified::Unknown,
            Err(Some(e)) => Notified::Failed(e),
        };
        if !matches!(n, Notified::Failed(_)) {
            self.r.meter.note_typed(session, ids); // FCW-5: a prompt naming these ids is automated
        }
        n
    }
    fn attempted(&self, ids: &[String], via: Via, ring_id: &str) {
        for id in ids {
            if let Err(e) = self.r.deliverer.mark_stage(id, ATTEMPTED, &format!("via={} ring_id={ring_id}", via.name())) {
                let _ = self.r.store.attention("attempted-unrecorded", &format!("{id}: {e}"));
            }
        }
    }
    fn unconfirmed(&self, id: &str) {
        let _ = self.r.deliverer.mark_stage(id, HANDOVER_UNCONFIRMED, "no hand-over evidence 90 s after the line; still owed");
    }
    fn attention(&self, kind: &str, key: &str, detail: &str) {
        let _ = self.r.store.attention_coalesced(kind, key, detail);
    }
    fn new_id(&self) -> String {
        crate::wire::mint_msg_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_match_06_4_2() {
        let v: Vec<u64> = (0..8).map(offset).collect();
        assert_eq!(v, [0, 4, 12, 32, 52, 72, 92, 112]);
    }

    #[test]
    fn ps_time_parses_all_forms() {
        assert_eq!(ps_time("0:00.03"), Some(30));
        assert_eq!(ps_time("1:02:03"), Some(3_723_000));
        assert_eq!(ps_time("2-00:00:01.50"), Some(2 * 86_400_000 + 1500));
        assert_eq!(ps_time("x"), None);
    }

    #[test]
    fn choose_takes_largest_activity_then_lowest_id() {
        let c = vec![("b".to_string(), Some(5)), ("a".to_string(), Some(5)), ("c".to_string(), Some(3))];
        assert_eq!(choose(&c), Some(1));
        let c = vec![("b".to_string(), None), ("a".to_string(), None)];
        assert_eq!(choose(&c), Some(1));
        assert_eq!(choose(&[]), None);
    }
}
