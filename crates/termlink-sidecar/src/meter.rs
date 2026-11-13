//! Per-session activity meter and conversation signal (arc-011 B-27; ARC4 6.14, 17.2..17.5; AL-1..AL-4).
//!
//! The doorbell (B-16), its watchdog and the lease need a measured answer to "is
//! anything happening in this session, and is a conversation under way". Three
//! sources, each with its own last time on this host's monotonic clock
//! ([`termlink_session::input_meter::mono_ms`]), kept per session:
//!
//! | signal | moves on | never moves on (6.14.2, FCW-14) |
//! |---|---|---|
//! | S1 | `client` PTY input the TermLink session counted (`session.input_activity`, notify scope) | `notify`, `inject`, `client_auto` input |
//! | S2 | (a) a turn event or a READY state change in a valid `termlink.adapter/2` report (or a built W1 hook call); (b) growth of the session's transcript | an unparseable report; a repeated `ready` state; `evidence`, `complete` |
//! | S3 | a message this session's agent sent, or one handed over to it | arrivals (STORED), receipts, notices |
//!
//! A source that cannot be read is `unknown`, never "silent" (FCW-12, FCW-13).
//! The conversation signal (AL-4) is computed from S1, S2 (a), S3, the prompt
//! origin and the owed-answer ledger, never from the READY flag. Status shows a
//! snapshot taken each tick; a snapshot older than two ticks reads `unknown`
//! (FCW-15). Nothing here types or rings: that is B-16.

use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The adapter contract this intake reads (ARC4 17.2), exactly.
pub const SCHEMA: &str = "termlink.adapter/2";
/// Largest report (FCW-2).
pub const MAX_REPORT: usize = 64 * 1024;
/// Live sessions per agent (FCW-7).
pub const MAX_SESSIONS: usize = 64;
/// Conversation window V-19.
pub const V19_MS: u64 = 120_000;
/// A status snapshot older than this reads unknown (FCW-15: 2 ticks).
pub const STALE_MS: u64 = 2 * crate::deadline::TICK_MS;
/// Bound on one S1 read (FCW-12).
pub const S1_TIMEOUT_MS: u64 = 2_000;
/// Time budget for all of one tick's S1 reads, which run in parallel; a read
/// still running at the budget reads unknown this tick (F-22, CR-6.20).
pub const TICK_BUDGET_MS: u64 = S1_TIMEOUT_MS + 500;
/// A report this recent keeps S2 known when the transcript cannot be read
/// (F-28, CR-6.25): the watchdog window V-18.
pub const S2_FRESH_MS: u64 = 300_000;
/// Strict JSON nesting limit (06 3.3).
pub const MAX_DEPTH: u32 = 8;
/// Largest integer a report may carry (06 3.3).
pub const MAX_INT: i64 = (1 << 53) - 1;
/// Answer events that end an owed message (answer 4.4).
const ANSWERS: [&str; 2] = ["REPLIED", "ACKNOWLEDGED_NO_ACTION"];
/// Ends of a message (answer 4.4).
const ENDS: [&str; 2] = ["REFUSED", "DEAD"];
/// Turn events (ARC4 17.4): with a READY state change, the only reports that move S2 (a) (F-3, CR-6.3).
const TURN_EVENTS: [&str; 6] = ["session_start", "prompt_submit", "tool_start", "tool_end", "turn_stop", "session_end"];
/// The built W1 hooks that are turn events (D-2).
const TURN_HOOKS: [&str; 4] = ["session-start", "stop", "session-end", "post-tool-use"];
/// The store state file holding every session's READY record and `last_prompt`
/// (AL-1 CR-6.23; P-1, T-3692).
pub const STATE_NAME: &str = "meter.ready";
const EVENTS: [&str; 9] = ["ready", "session_start", "prompt_submit", "tool_start", "tool_end", "turn_stop", "session_end", "evidence", "complete"];

// ---------------------------------------------------------------- strict JSON

/// Strict JSON (06 3.3): UTF-8, duplicate keys refused, depth <= 8, integers
/// only in +-(2^53-1) (no fractions, exponents or `-0`). Refusal is a value.
pub fn strict_json(raw: &[u8]) -> Result<Value, String> {
    use serde::de::DeserializeSeed;
    let mut de = serde_json::Deserializer::from_slice(raw);
    let v = Seed(0).deserialize(&mut de).map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    if has_negative_zero(raw) {
        return Err("-0 is not a canonical integer".into());
    }
    Ok(v)
}

/// `-0` outside strings (serde reads it as the integer 0).
fn has_negative_zero(raw: &[u8]) -> bool {
    let (mut in_str, mut esc) = (false, false);
    for (i, b) in raw.iter().enumerate() {
        if in_str {
            match (esc, b) {
                (true, _) => esc = false,
                (false, b'\\') => esc = true,
                (false, b'"') => in_str = false,
                _ => {}
            }
        } else if *b == b'"' {
            in_str = true;
        } else if *b == b'-' && raw.get(i + 1) == Some(&b'0') && !raw.get(i + 2).is_some_and(u8::is_ascii_digit) {
            return true;
        }
    }
    false
}

struct Seed(u32);

impl<'de> serde::de::DeserializeSeed<'de> for Seed {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(Vis(self.0))
    }
}

struct Vis(u32);

impl<'de> serde::de::Visitor<'de> for Vis {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("strict JSON")
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
        if !(-MAX_INT..=MAX_INT).contains(&v) {
            return Err(E::custom("integer out of range"));
        }
        Ok(json!(v))
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
        if v > MAX_INT as u64 {
            return Err(E::custom("integer out of range"));
        }
        Ok(json!(v))
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Value, E> {
        Err(E::custom("fractions and exponents are refused"))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        if self.0 >= MAX_DEPTH {
            return Err(serde::de::Error::custom("nested too deep"));
        }
        let mut v = vec![];
        while let Some(x) = a.next_element_seed(Seed(self.0 + 1))? {
            v.push(x);
        }
        Ok(Value::Array(v))
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        if self.0 >= MAX_DEPTH {
            return Err(serde::de::Error::custom("nested too deep"));
        }
        let mut m = serde_json::Map::new();
        while let Some(k) = a.next_key::<String>()? {
            if m.contains_key(&k) {
                return Err(serde::de::Error::custom(format!("duplicate key {k:?}")));
            }
            let v = a.next_value_seed(Seed(self.0 + 1))?;
            m.insert(k, v);
        }
        Ok(Value::Object(m))
    }
}

// ---------------------------------------------------------------- types

/// Prompt origin as the receiver records it (ARC4 17.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Human,
    Automated,
    Unknown,
}

impl Origin {
    pub fn name(self) -> &'static str {
        match self {
            Origin::Human => "human",
            Origin::Automated => "automated",
            Origin::Unknown => "unknown",
        }
    }
}

/// The order of a report (04 17.2.a, 06 AL-1 CR-6.24): the adapter's
/// `(boot_id, mono_ns)` when it sends both, else the receiver's own receive
/// stamp (domain [`RX_DOMAIN`]); never `at_ms`, which a wall-clock step moves.
/// Within one domain the higher `ns` is later; a report of another domain (a new
/// boot, or a switch between adapter stamps and receive stamps) is later than
/// every report of the previous one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportOrd {
    pub boot: String,
    pub ns: u64,
}

/// The domain of receive-stamp orders (`:` cannot occur in an adapter's `boot_id`).
pub const RX_DOMAIN: &str = "rx:";

impl ReportOrd {
    /// The receive-stamp order of a report received at `rx_ms` (mono).
    pub fn rx(rx_ms: u64) -> Self {
        ReportOrd { boot: RX_DOMAIN.into(), ns: rx_ms.saturating_mul(1_000_000) }
    }
    /// Strictly before `o` (same domain, lower `ns`).
    pub fn before(&self, o: &ReportOrd) -> bool {
        self.boot == o.boot && self.ns < o.ns
    }
    /// Strictly after `o` (another domain, or same domain and higher `ns`).
    pub fn after(&self, o: &ReportOrd) -> bool {
        self.boot != o.boot || self.ns > o.ns
    }
    fn view(&self) -> Value {
        json!({"boot": self.boot, "ns": self.ns.to_string()})
    }
    fn from_view(v: &Value) -> Option<ReportOrd> {
        Some(ReportOrd { boot: v.get("boot")?.as_str()?.to_owned(), ns: v.get("ns")?.as_str()?.parse().ok()? })
    }
}

/// The adapter's `(boot_id, mono_ns)` of a report or hook request, when both are
/// present and valid. `boot_id`: 1..=64 of `[A-Za-z0-9-]`; `mono_ns`: an integer
/// of at least 0, or a decimal string of one (a u64 in ns passes 2^53 after ~104 days of
/// uptime, beyond strict JSON's integers, so the string form is accepted).
pub fn adapter_ord(obj: &serde_json::Map<String, Value>) -> Option<ReportOrd> {
    let boot = obj.get("boot_id")?.as_str().filter(|b| (1..=64).contains(&b.len()) && b.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'))?;
    let ns = match obj.get("mono_ns")? {
        Value::Number(n) => n.as_u64()?,
        Value::String(t) if (1..=20).contains(&t.len()) && t.bytes().all(|c| c.is_ascii_digit()) => t.parse().ok()?,
        _ => return None,
    };
    Some(ReportOrd { boot: boot.to_owned(), ns })
}

/// The READY record of a session (AL-1, OBS-2). `epoch` is the sidecar process
/// that received it: another process's record reads absent for the doorbell (FCW-6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyRec {
    pub state: String,
    /// The adapter's wall time: display and audit only (CR-6.24).
    pub at_ms: u64,
    /// The order the record was applied under (CR-6.24).
    pub ord: ReportOrd,
    pub rx_mono: u64,
    pub seq: u64,
    pub epoch: u64,
}

/// The last prompt of a session (OBS-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub origin: Origin,
    pub basis: String,
    pub at_ms: u64,
    pub ord: ReportOrd,
    pub rx_mono: u64,
    pub doorbell_ids: Vec<String>,
}

/// One read of the session's PTY input counters (AL-2 reply).
#[derive(Debug, Clone, PartialEq)]
pub struct S1Reply {
    pub boot_id: Option<String>,
    /// `last_mono` per source: `client`, `client_auto`, `notify`, `inject`.
    pub last: BTreeMap<String, Option<u64>>,
    pub foreground: Value,
}

impl S1Reply {
    /// Parse a `session.input_activity` result. A session without a PTY has no
    /// sources: that is an error here, so S1 reads unknown.
    pub fn from_result(v: &Value) -> Result<S1Reply, String> {
        let src = v.get("sources").and_then(Value::as_object).ok_or("the session has no PTY input counters")?;
        let mut last = BTreeMap::new();
        for s in ["client", "client_auto", "notify", "inject"] {
            let e = src.get(s).ok_or(format!("no {s} counter"))?;
            last.insert(s.to_string(), e.get("last_mono").and_then(Value::as_u64));
        }
        Ok(S1Reply { boot_id: v.get("boot_id").and_then(Value::as_str).map(str::to_owned), last, foreground: v.get("foreground").cloned().unwrap_or(Value::Null) })
    }
    pub fn at(&self, src: &str) -> Option<u64> {
        self.last.get(src).copied().flatten()
    }
}

/// Where S1 comes from: the TermLink session serving a harness session.
pub trait S1Source: Send + Sync {
    fn read(&self, session: &str) -> Result<S1Reply, String>;
}

/// No TermLink session is bound to any harness session: S1 is unknown everywhere.
pub struct Unbound;

impl S1Source for Unbound {
    fn read(&self, _: &str) -> Result<S1Reply, String> {
        Err("no TermLink PTY session is bound to this session".into())
    }
}

/// S1 read through `session.input_activity` on a bound TermLink session socket
/// with a notify-scope token (AL-2, FCW-9). B-16 binds a harness session to the
/// TermLink session that hosts it ([`crate::doorbell::find_binding`]); an unbound
/// session reads unknown.
pub struct SessionRpc {
    bound: Mutex<HashMap<String, (PathBuf, Option<String>)>>,
    /// B-16: (TermLink session id, its token secret hex): a notify-scope token is
    /// minted per call, so no token outlives its use.
    secrets: Mutex<HashMap<String, (String, String)>>,
    /// T-3772: why a session is not bound (no harness_pid, no /proc, no hosting
    /// TermLink session), so an S1 read says why instead of a bare "not bound".
    unbound_why: Mutex<HashMap<String, String>>,
}

impl SessionRpc {
    pub fn new() -> std::io::Result<Self> {
        Ok(SessionRpc { bound: Mutex::new(HashMap::new()), secrets: Mutex::new(HashMap::new()), unbound_why: Mutex::new(HashMap::new()) })
    }
    pub fn bind(&self, session: &str, socket: &Path, token: Option<String>) {
        lock(&self.bound).insert(session.to_owned(), (socket.to_path_buf(), token));
        lock(&self.secrets).remove(session);
        lock(&self.unbound_why).remove(session);
    }
    /// B-16: bind with the TermLink session's token secret (notify scope minted per call).
    pub fn bind_secret(&self, session: &str, socket: &Path, tl_session: &str, secret_hex: Option<String>) {
        lock(&self.bound).insert(session.to_owned(), (socket.to_path_buf(), None));
        lock(&self.unbound_why).remove(session);
        match secret_hex {
            Some(s) => lock(&self.secrets).insert(session.to_owned(), (tl_session.to_owned(), s)),
            None => lock(&self.secrets).remove(session),
        };
    }
    pub fn is_bound(&self, session: &str) -> bool {
        lock(&self.bound).contains_key(session)
    }
    pub fn unbind(&self, session: &str) {
        lock(&self.bound).remove(session);
        lock(&self.secrets).remove(session);
    }
    /// T-3772: unbind and record why (read back by S1 errors and status).
    pub fn unbind_why(&self, session: &str, why: &str) {
        self.unbind(session);
        lock(&self.unbound_why).insert(session.to_owned(), why.to_owned());
    }
    /// T-3772: why `session` is not bound, when a binding attempt recorded it.
    pub fn why_unbound(&self, session: &str) -> Option<String> {
        if self.is_bound(session) {
            return None;
        }
        lock(&self.unbound_why).get(session).cloned()
    }
    fn target(&self, session: &str) -> Result<(PathBuf, Option<String>), String> {
        let Some((sock, token)) = lock(&self.bound).get(session).cloned() else {
            return Err(match self.why_unbound(session) {
                Some(why) => format!("no TermLink PTY session is bound to this session ({why})"),
                None => "no TermLink PTY session is bound to this session".into(),
            });
        };
        if let Some((tl, hex)) = lock(&self.secrets).get(session).cloned() {
            let bytes: Vec<u8> = (0..hex.len() / 2).filter_map(|i| u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()).collect();
            let secret: termlink_session::auth::TokenSecret = bytes.try_into().map_err(|_| "the session's token secret is not 32 bytes of hex".to_string())?;
            let t = termlink_session::auth::create_token(&secret, termlink_session::auth::PermissionScope::Notify, &tl, 60);
            return Ok((sock, Some(t.raw)));
        }
        Ok((sock, token))
    }
    /// One RPC with a timeout; `Err(None)` is a timeout, `Err(Some(e))` a failure.
    pub fn call(&self, session: &str, method: &str, params: Value, timeout_ms: u64) -> Result<Value, Option<String>> {
        let (sock, token) = self.target(session).map_err(Some)?;
        let call = termlink_session::client::rpc_call_with_token_timeout(&sock, token.as_deref(), method, params, std::time::Duration::from_millis(timeout_ms));
        // A runtime per call, built and dropped on the calling (plain) thread:
        // a stored one could be dropped inside an async context and panic.
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| Some(e.to_string()))?;
        let resp = rt.block_on(async { tokio::time::timeout(std::time::Duration::from_millis(timeout_ms), call).await });
        let resp = resp.map_err(|_| None)?.map_err(|e| Some(e.to_string()))?;
        termlink_session::client::unwrap_result(resp).map_err(Some)
    }
}

impl S1Source for SessionRpc {
    fn read(&self, session: &str) -> Result<S1Reply, String> {
        let v = self
            .call(session, termlink_protocol::control::method::SESSION_INPUT_ACTIVITY, json!({}), S1_TIMEOUT_MS)
            .map_err(|e| e.unwrap_or_else(|| "input_activity timed out".to_string()))?;
        S1Reply::from_result(&v)
    }
}

impl<T: S1Source + ?Sized> S1Source for Arc<T> {
    fn read(&self, session: &str) -> Result<S1Reply, String> {
        (**self).read(session)
    }
}

/// The owed-answer ledger (answer 4.4): one function, so P14's owed list can reuse it.
pub trait Ledger: Send + Sync {
    /// Does conversation `conv` hold a message with an owed answer at wall time `now_ms`?
    fn owed(&self, conv: &str, now_ms: u64) -> Result<bool, String>;
}

/// [`Ledger`] over this agent's stage records.
pub struct StoreLedger {
    pub store: Arc<crate::store::Store>,
    pub deadlines: crate::deadline::Deadlines,
}

impl Ledger for StoreLedger {
    fn owed(&self, conv: &str, now_ms: u64) -> Result<bool, String> {
        let lines = self.store.chain_lines(conv).map_err(|e| e.to_string())?;
        Ok(owed_in(&lines, now_ms, self.deadlines.answer(false)))
    }
}

/// Answer 4.4 over one conversation's chain lines. A message `m` is owed when
/// (i) it was handed over here (own HANDED_OVER) and this side recorded no
/// answer, or (ii) this side sent it (own SENT) and the peer reported no answer;
/// and in both cases it is not ended (REFUSED, DEAD, or older than the R-44.e
/// "not answered" deadline). Receipts and notices are other events: never `m`.
pub fn owed_in(lines: &[crate::store::ChainLine], now_ms: u64, answer_ms: u64) -> bool {
    let own = |l: &crate::store::ChainLine| l.peer_ts_ms.is_none();
    let ended = |m: &str, since: Option<u64>| {
        lines.iter().any(|l| l.msg_id == m && ENDS.contains(&l.event.as_str())) || since.is_some_and(|t| now_ms.saturating_sub(t) > answer_ms)
    };
    let handed = lines.iter().filter(|l| l.event == crate::deliver::HANDED_OVER && own(l)).any(|h| {
        !lines.iter().any(|l| own(l) && l.msg_id == h.msg_id && ANSWERS.contains(&l.event.as_str())) && !ended(&h.msg_id, h.ts_ms)
    });
    handed
        || lines.iter().filter(|l| l.event == "SENT" && own(l)).any(|s| {
            !lines.iter().any(|l| !own(l) && l.msg_id == s.msg_id && ANSWERS.contains(&l.event.as_str())) && !ended(&s.msg_id, s.ts_ms)
        })
}

/// A ledger that cannot be read (tests, and a sidecar without a store).
pub struct NoLedger;

impl Ledger for NoLedger {
    fn owed(&self, _: &str, _: u64) -> Result<bool, String> {
        Err("no owed-answer ledger".into())
    }
}

/// S1 of one session as last read.
#[derive(Debug, Clone, PartialEq)]
pub enum S1 {
    /// Never read, or the last read failed (FCW-12).
    Unknown(String),
    /// Read at `read_at`; `client` is the last client input (None: none yet).
    Known { client: Option<u64>, read_at: u64 },
}

#[derive(Debug, Clone, Default)]
struct Session {
    ready: Option<ReadyRec>,
    interactive: Option<bool>,
    transcript: Option<PathBuf>,
    tr_last: Option<(u64, i64, i64)>,
    /// The last tick's `lstat` of the transcript failed (F-28).
    tr_unreadable: bool,
    last_prompt: Option<Prompt>,
    /// rx of the first turn_stop or READY report after the last prompt.
    stopped: Option<u64>,
    tool: Option<(String, u64)>,
    reports: u64,
    /// rx of the last valid report of any kind (F-28 freshness).
    last_report: Option<u64>,
    s1: Option<S1>,
    /// Every source of the last good S1 read (for the origin check, AL-1 3).
    s1_sources: Option<S1Reply>,
    s2_at: Option<u64>,
    /// Last S2 (a) event (turn events only, not transcript growth).
    s2_event_at: Option<u64>,
    s3_at: Option<u64>,
    s3_by_conv: BTreeMap<String, u64>,
    evidence: Vec<(String, u64)>,
    complete_at: Option<u64>,
    ended: bool,
    snapshot: Option<(Value, u64)>,
    /// B-16 (CR-6.19, 04 17.4.a): the harness pid `session_start` reported.
    harness_pid: Option<u32>,
}

/// Counters the evidence runner reads (OBS-1).
#[derive(Debug, Clone, Default)]
pub struct Counters {
    pub unparseable: u64,
    pub ready_reordered: u64,
    /// `prompt_submit` reports older than the applied `last_prompt` (04 17.2.a).
    pub prompt_reordered: u64,
    pub origin_inconsistent: u64,
    pub origin_overridden: u64,
    pub transcript_refused: u64,
    pub session_table_full: u64,
    pub s3_unattributed: u64,
    /// Writes of [`STATE_NAME`] that failed (the record stays in memory).
    pub persist_failed: u64,
    /// Sessions read back from [`STATE_NAME`] at start.
    pub restored: u64,
    pub reasons: VecDeque<String>,
}

#[derive(Default)]
struct Inner {
    sessions: BTreeMap<String, Session>,
    counters: Counters,
    /// Doorbell lines typed per session, `(mono, ids)` (B-16 writes it; FCW-5).
    typed: HashMap<String, VecDeque<(u64, Vec<String>)>>,
}

pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// The meter of one agent's sessions.
pub struct Meter {
    agent: String,
    epoch: u64,
    clock: Clock,
    wall: Clock,
    transcript_root: Option<PathBuf>,
    s1: Arc<dyn S1Source>,
    /// Sessions whose S1 read has not returned yet (at most one read each).
    s1_inflight: Arc<Mutex<std::collections::HashSet<String>>>,
    tick_budget_ms: u64,
    ledger: Box<dyn Ledger>,
    /// Where READY and `last_prompt` are persisted (CR-6.23); none: memory only.
    state: Option<Arc<crate::store::Store>>,
    /// This host's boot id: persisted records are read back on the same boot only.
    boot: Option<String>,
    inner: Mutex<Inner>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

fn valid_session(s: &str) -> bool {
    (1..=128).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
}

/// What an intake call did, for the reply and the tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intake {
    Applied,
    /// Valid but not applied (an older `ready`, FCW-3).
    Dropped(&'static str),
    Ignored(String),
}

impl Meter {
    pub fn new(agent: &str, s1: Box<dyn S1Source>, ledger: Box<dyn Ledger>) -> Self {
        Meter {
            agent: agent.to_owned(),
            epoch: rand_epoch(),
            clock: Arc::new(termlink_session::input_meter::mono_ms),
            wall: Arc::new(crate::store::now_ms),
            transcript_root: None,
            s1: Arc::from(s1),
            s1_inflight: Arc::new(Mutex::new(std::collections::HashSet::new())),
            tick_budget_ms: TICK_BUDGET_MS,
            ledger,
            state: None,
            boot: termlink_session::input_meter::boot_id(),
            inner: Mutex::new(Inner::default()),
        }
    }

    /// Test seam: the boot id the persisted records are checked against.
    pub fn with_boot(mut self, boot: Option<String>) -> Self {
        self.boot = boot;
        self
    }

    /// Persist each session's READY record and `last_prompt` in `store`, and read
    /// back what an earlier sidecar process of this boot left there (AL-1, AL-4,
    /// CR-6.23: QR-2 closed). A restored READY record keeps its old epoch, so the
    /// doorbell still reads it absent (FCW-6); a restored `last_prompt` keeps a
    /// person mid-conversation protected. Records of another boot are not used.
    pub fn with_state(mut self, store: Arc<crate::store::Store>) -> Self {
        let read = store.read_state(STATE_NAME);
        self.state = Some(store);
        let mut guard = lock(&self.inner);
        let g = &mut *guard;
        match read {
            Ok(None) => {}
            Ok(Some(raw)) => match restore(&raw, self.boot.as_deref()) {
                Ok(list) => {
                    for (k, sess) in list.into_iter().take(2 * MAX_SESSIONS) {
                        g.sessions.insert(k, sess);
                        g.counters.restored += 1;
                    }
                }
                Err(why) => g.counters.reasons.push_back(format!("persisted meter state not used: {why}")),
            },
            Err(e) => g.counters.reasons.push_back(format!("persisted meter state unreadable: {e}")),
        }
        drop(guard);
        self
    }

    /// Write every session's READY record and `last_prompt` (under the inner
    /// lock, so writes land in intake order). A failed write is counted.
    fn persist(&self, inner: &mut Inner) {
        let Some(store) = &self.state else { return };
        let mut sessions = serde_json::Map::new();
        for (k, s) in &inner.sessions {
            if s.ready.is_none() && s.last_prompt.is_none() {
                continue;
            }
            sessions.insert(
                k.clone(),
                json!({
                    "ready": s.ready.as_ref().map(|r| json!({"state": r.state, "at_ms": r.at_ms, "ord": r.ord.view(), "rx_mono": r.rx_mono, "seq": r.seq, "epoch": r.epoch.to_string()})),
                    "last_prompt": s.last_prompt.as_ref().map(|p| json!({"origin": p.origin.name(), "basis": p.basis, "at_ms": p.at_ms, "ord": p.ord.view(), "rx_mono": p.rx_mono, "doorbell_ids": p.doorbell_ids})),
                    "stopped": s.stopped,
                    "ended": s.ended,
                }),
            );
        }
        // The epoch is a full u64 (above strict JSON's 2^53): written as a string.
        let doc = json!({"v": 1, "boot_id": self.boot, "sessions": sessions});
        if let Err(e) = store.write_state(STATE_NAME, &doc.to_string()) {
            inner.counters.persist_failed += 1;
            inner.counters.reasons.push_back(format!("persist {STATE_NAME}: {e}"));
            while inner.counters.reasons.len() > 16 {
                inner.counters.reasons.pop_front();
            }
        }
    }

    /// Test seam: simulated clocks (mono and wall).
    pub fn with_clocks(mut self, mono: Clock, wall: Clock) -> Self {
        self.clock = mono;
        self.wall = wall;
        self
    }

    /// Test seam: the tick's S1 read budget (F-22).
    pub fn with_tick_budget(mut self, ms: u64) -> Self {
        self.tick_budget_ms = ms;
        self
    }

    /// Transcripts are accepted only under this directory (FCW-8).
    pub fn with_transcript_root(mut self, root: PathBuf) -> Self {
        self.transcript_root = Some(root);
        self
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn counters(&self) -> Counters {
        lock(&self.inner).counters.clone()
    }

    fn ignore(&self, c: &mut Counters, reason: String) -> Intake {
        c.unparseable += 1;
        c.reasons.push_back(reason.clone());
        while c.reasons.len() > 16 {
            c.reasons.pop_front();
        }
        Intake::Ignored(reason)
    }

    /// AL-1: one adapter report, as received on the owner-only socket (the
    /// peer check is the socket's, FCW-1). Never fails; refusal is counted.
    pub fn intake(&self, raw: &[u8]) -> Intake {
        if raw.len() > MAX_REPORT {
            return self.ignore(&mut lock(&self.inner).counters, "too-large".into());
        }
        let obj = match strict_json(raw) {
            Ok(Value::Object(o)) => o,
            Ok(_) => return self.ignore(&mut lock(&self.inner).counters, "not an object".into()),
            Err(e) => return self.ignore(&mut lock(&self.inner).counters, format!("not strict JSON: {e}")),
        };
        let s = |k: &str| obj.get(k).and_then(Value::as_str);
        let bad = |why: &str| self.ignore(&mut lock(&self.inner).counters, why.to_owned());
        if s("schema") != Some(SCHEMA) {
            return bad("schema is not termlink.adapter/2");
        }
        let Some(event) = s("event").filter(|e| EVENTS.contains(e)) else { return bad("unknown event") };
        if s("agent_id") != Some(self.agent.as_str()) {
            return bad("another agent's id");
        }
        let Some(session) = s("session_id").filter(|x| valid_session(x)) else { return bad("bad session_id") };
        let Some(at_ms) = obj.get("at_ms").and_then(Value::as_u64) else { return bad("at_ms is not an integer >= 0") };
        // Event-specific shape, checked before anything is applied.
        match event {
            "ready" if !matches!(s("state"), Some("READY" | "BUSY" | "NOT_RUNNING")) => return bad("ready needs state READY|BUSY|NOT_RUNNING"),
            "session_start" if !obj.get("interactive").is_some_and(Value::is_boolean) => return bad("session_start needs interactive (bool)"),
            "session_start" if obj.get("transcript_path").is_some_and(|v| !v.is_string() && !v.is_null()) => return bad("transcript_path must be a string"),
            "prompt_submit" if !obj.get("origin").is_some_and(Value::is_string) => return bad("prompt_submit needs origin"),
            // T-3779: a basis of the wrong type is FCW-2; missing or null is `absent` (below).
            "prompt_submit" if obj.get("basis").is_some_and(|v| !v.is_string() && !v.is_null()) => return bad("basis must be a string"),
            "tool_start" | "tool_end" if s("tool").is_none_or(|t| t.len() > 128) => return bad("tool needs a name of at most 128 bytes"),
            "evidence" if s("nonce").is_none() => return bad("evidence needs nonce"),
            _ => {}
        }
        let rx = (self.clock)();
        // CR-6.24: (boot_id, mono_ns), or the receive stamp; never at_ms.
        let ord = adapter_ord(&obj).unwrap_or_else(|| ReportOrd::rx(rx));
        // AL-1 check_origin 3 needs the session's input counters as of now, for
        // every `human` prompt, whatever its basis (F-4, CR-6.4).
        let origin_s1 = (event == "prompt_submit" && s("origin") == Some("human")).then(|| self.s1.read(session));
        let transcript = if event == "session_start" { s("transcript_path").map(|p| self.validate_transcript(p)) } else { None };

        let mut g = lock(&self.inner);
        let inner = &mut *g;
        if !inner.sessions.contains_key(session) {
            let live = inner.sessions.values().filter(|x| !x.ended).count();
            if live >= MAX_SESSIONS {
                inner.counters.session_table_full += 1;
                return Intake::Ignored("session-table-full".into());
            }
            // Bound the table: ended sessions go first.
            if inner.sessions.len() >= 2 * MAX_SESSIONS
                && let Some(k) = inner.sessions.iter().filter(|(_, x)| x.ended).min_by_key(|(_, x)| x.s2_at).map(|(k, _)| k.clone())
            {
                inner.sessions.remove(&k);
            }
        }
        let typed_ids: Vec<String> = inner
            .typed
            .get(session)
            .map(|q| q.iter().filter(|(t, _)| rx.saturating_sub(*t) <= crate::deadline::TICK_MS).flat_map(|(_, ids)| ids.clone()).collect())
            .unwrap_or_default();
        // FCW-195: the newest line the receiver typed into this session within a tick.
        let last_typed: Option<u64> = inner
            .typed
            .get(session)
            .and_then(|q| q.iter().map(|(t, _)| *t).filter(|t| *t <= rx && rx.saturating_sub(*t) <= crate::deadline::TICK_MS).max());
        let counters = &mut inner.counters;
        let sess = inner.sessions.entry(session.to_owned()).or_default();
        let mut out = Intake::Applied;
        // S2 (a) moves on turn events; a `ready` moves it only when it changes the state (F-3).
        let mut moves_s2 = TURN_EVENTS.contains(&event);
        match event {
            "ready" => {
                let state = s("state").unwrap_or_default().to_owned();
                match &sess.ready {
                    Some(r) if ord.before(&r.ord) => {
                        counters.ready_reordered += 1;
                        out = Intake::Dropped("ready-reordered");
                    }
                    Some(r) if ord == r.ord && r.state == "BUSY" && state != "BUSY" => out = Intake::Dropped("tie keeps BUSY"),
                    _ => {
                        let seq = sess.ready.as_ref().map_or(1, |r| r.seq + 1);
                        moves_s2 = sess.ready.as_ref().map(|r| r.state.as_str()) != Some(state.as_str());
                        if state == "READY" {
                            mark_stopped(sess, &ord, rx);
                        }
                        sess.ended = state == "NOT_RUNNING";
                        sess.ready = Some(ReadyRec { state, at_ms, ord: ord.clone(), rx_mono: rx, seq, epoch: self.epoch });
                    }
                }
            }
            "session_start" => {
                sess.interactive = obj.get("interactive").and_then(Value::as_bool);
                sess.ended = false;
                // B-16: a new harness instance is unregistered until it says which pid it is.
                sess.harness_pid = obj.get("harness_pid").and_then(Value::as_u64).and_then(|p| u32::try_from(p).ok()).filter(|p| *p > 1);
                match transcript {
                    Some(Ok(p)) => {
                        sess.transcript = Some(p);
                        sess.tr_last = None;
                    }
                    Some(Err(why)) => {
                        counters.transcript_refused += 1;
                        counters.reasons.push_back(format!("transcript: {why}"));
                        sess.transcript = None;
                    }
                    None => sess.transcript = None,
                }
            }
            // 04 17.2.a: a prompt older than the applied one is ignored and counted.
            "prompt_submit" if sess.last_prompt.as_ref().is_some_and(|p| ord.before(&p.ord)) => {
                counters.prompt_reordered += 1;
                out = Intake::Dropped("prompt-reordered");
            }
            "prompt_submit" => {
                let ids = valid_ids(obj.get("doorbell_ids"));
                let prev = sess.last_prompt.as_ref().map(|p| p.rx_mono);
                // T-3779 (04 17.5.c, 06 AL-1): no basis field (or null) IS basis `absent`,
                // the only basis of an `unknown` report; it is not an inconsistent pair.
                let (origin, basis) = check_origin(counters, s("origin").unwrap_or(""), s("basis").unwrap_or("absent"), &ids, &typed_ids, origin_s1, rx, prev, last_typed);
                sess.last_prompt = Some(Prompt { origin, basis, at_ms, ord: ord.clone(), rx_mono: rx, doorbell_ids: ids });
                sess.stopped = None;
            }
            "tool_start" | "tool_end" => sess.tool = Some((s("tool").unwrap_or_default().to_owned(), rx)),
            "turn_stop" => mark_stopped(sess, &ord, rx),
            "session_end" => {
                let seq = sess.ready.as_ref().map_or(1, |r| r.seq + 1);
                sess.ready = Some(ReadyRec { state: "NOT_RUNNING".into(), at_ms, ord: ord.clone(), rx_mono: rx, seq, epoch: self.epoch });
                sess.ended = true;
            }
            // AL-9 (B-16/B-20) decides hand-over from these; here they are recorded only.
            "evidence" => {
                sess.evidence.push((s("nonce").unwrap_or_default().to_owned(), rx));
                if sess.evidence.len() > 16 {
                    sess.evidence.remove(0);
                }
            }
            "complete" => sess.complete_at = Some(rx),
            _ => {}
        }
        while counters.reasons.len() > 16 {
            counters.reasons.pop_front();
        }
        // S2 (a): turn events and READY state changes only (AL-1, CR-6.3). An
        // adapter repeating `ready {BUSY}`, `evidence` or `complete` is not activity.
        sess.reports += 1;
        sess.last_report = Some(rx);
        if moves_s2 {
            stamp_s2(sess, rx);
        }
        // CR-6.23: READY and last_prompt (and the turn stop that ends the prompt's
        // window) are persisted together, so a restart does not lose them.
        if out == Intake::Applied && matches!(event, "ready" | "prompt_submit" | "turn_stop" | "session_end") {
            self.persist(inner);
        }
        out
    }

    /// The built W1 hooks (`ready`/`pull` ops: session-start, stop, session-end,
    /// post-tool-use) are turn events too: they move S2 (a) for the session.
    /// `transcript` is the path the deliverer resolved for it, checked like
    /// an adapter's (FCW-8). Deviation D-2 in T-3687. `ord` is the hook's
    /// `(boot_id, mono_ns)` when it sent both (P-2, CR-6.24), else the receive stamp.
    pub fn legacy_hook(&self, session: &str, hook: &str, transcript: Option<&Path>, ord: Option<ReportOrd>) {
        if !valid_session(session) {
            return;
        }
        let rx = (self.clock)();
        let ord = ord.unwrap_or_else(|| ReportOrd::rx(rx));
        let tr = transcript.map(|p| self.validate_transcript(&p.to_string_lossy()));
        let mut g = lock(&self.inner);
        let inner = &mut *g;
        if !inner.sessions.contains_key(session) && inner.sessions.values().filter(|x| !x.ended).count() >= MAX_SESSIONS {
            inner.counters.session_table_full += 1;
            return;
        }
        let sess = inner.sessions.entry(session.to_owned()).or_default();
        match hook {
            "session-start" => {
                sess.ended = false;
                if let Some(Ok(p)) = tr
                    && sess.transcript.is_none()
                {
                    sess.transcript = Some(p);
                }
            }
            "stop" => mark_stopped(sess, &ord, rx),
            "session-end" => sess.ended = true,
            _ => {}
        }
        sess.reports += 1;
        sess.last_report = Some(rx);
        if TURN_HOOKS.contains(&hook) {
            stamp_s2(sess, rx);
        }
        if matches!(hook, "stop" | "session-end") {
            self.persist(inner);
        }
    }

    /// S3: this session's agent sent a message in `conv` (the send API).
    pub fn on_sent(&self, session: Option<&str>, conv: &str) {
        self.s3(session, conv);
    }

    /// S3: a message of `conv` was handed over to `session` (HANDED_OVER).
    pub fn on_handed_over(&self, session: &str, conv: &str) {
        self.s3(Some(session), conv);
    }

    fn s3(&self, session: Option<&str>, conv: &str) {
        let now = (self.clock)();
        let mut g = lock(&self.inner);
        let inner = &mut *g;
        // Only a session this meter knows: an unnamed sender, or a pseudo
        // session such as the `claim` op's, attributes to nobody.
        let Some(sess) = session.and_then(|s| inner.sessions.get_mut(s)) else {
            inner.counters.s3_unattributed += 1;
            return;
        };
        sess.s3_at = Some(now);
        sess.s3_by_conv.insert(conv.to_owned(), now);
        sess.s3_by_conv.retain(|_, t| now.saturating_sub(*t) <= V19_MS);
    }

    /// B-16 records each doorbell line it types (FCW-5: a `human` prompt that
    /// names those ids within a tick is recorded `automated`).
    pub fn note_typed(&self, session: &str, ids: &[String]) {
        let now = (self.clock)();
        let mut g = lock(&self.inner);
        let q = g.typed.entry(session.to_owned()).or_default();
        q.push_back((now, ids.to_vec()));
        q.retain(|(t, _)| now.saturating_sub(*t) <= crate::deadline::TICK_MS);
        while q.len() > 16 {
            q.pop_front();
        }
    }

    fn validate_transcript(&self, p: &str) -> Result<PathBuf, String> {
        let root = self.transcript_root.as_ref().ok_or("no harness project directory configured")?;
        let path = Path::new(p);
        if !path.is_absolute() {
            return Err("not absolute".into());
        }
        let md = std::fs::symlink_metadata(path).map_err(|e| format!("cannot stat: {e}"))?;
        if md.file_type().is_symlink() {
            return Err("a symlink".into());
        }
        if !md.is_file() {
            return Err("not a regular file".into());
        }
        let real = path.canonicalize().map_err(|e| e.to_string())?;
        let root = root.canonicalize().map_err(|e| format!("project directory: {e}"))?;
        if !real.starts_with(&root) {
            return Err("outside the harness's project directory".into());
        }
        Ok(real)
    }

    /// AL-3 meter_tick for every session, then the status snapshot.
    pub fn tick(&self) {
        let names: Vec<String> = lock(&self.inner).sessions.iter().filter(|(_, s)| !s.ended).map(|(k, _)| k.clone()).collect();
        // Reads happen outside the lock (an RPC must not hold up intake), in
        // parallel, under one per-tick budget (F-22, CR-6.20).
        let reads = self.read_s1_all(&names);
        let my_boot = termlink_session::input_meter::boot_id();
        let now = (self.clock)();
        let mut g = lock(&self.inner);
        for (name, r) in reads {
            let Some(sess) = g.sessions.get_mut(&name) else { continue };
            match r {
                Ok(r) if my_boot.is_some() && r.boot_id == my_boot => {
                    sess.s1 = Some(S1::Known { client: r.at("client"), read_at: now });
                    sess.s1_sources = Some(r);
                }
                Ok(_) => {
                    sess.s1 = Some(S1::Unknown("the session answered from another boot".into()));
                    sess.s1_sources = None;
                }
                Err(e) => {
                    sess.s1 = Some(S1::Unknown(e));
                    sess.s1_sources = None;
                }
            }
            if let Some(t) = &sess.transcript {
                match std::fs::symlink_metadata(t) {
                    Ok(md) => {
                        use std::os::unix::fs::MetadataExt;
                        let cur = (md.size(), md.mtime(), md.mtime_nsec());
                        if sess.tr_last.is_some_and(|l| l != cur) {
                            sess.s2_at = Some(now);
                        }
                        sess.tr_last = Some(cur);
                        sess.tr_unreadable = false;
                    }
                    // A failed lstat is not "no growth" (F-28).
                    Err(_) => sess.tr_unreadable = true,
                }
            }
        }
        let names: Vec<String> = g.sessions.keys().cloned().collect();
        drop(g);
        for n in names {
            let v = self.view(&n, now);
            if let Some(s) = lock(&self.inner).sessions.get_mut(&n) {
                s.snapshot = Some((v, now));
            }
        }
    }

    /// One tick's S1 reads: one thread per session, all bounded together by
    /// `tick_budget_ms`. A read still running at the budget reads unknown this
    /// tick; a session whose previous read has not returned is not read again
    /// (at most one read in flight per session, so threads stay bounded by the
    /// session table).
    fn read_s1_all(&self, names: &[String]) -> Vec<(String, Result<S1Reply, String>)> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut got: HashMap<String, Result<S1Reply, String>> = HashMap::new();
        let mut waiting = 0usize;
        for n in names {
            if !lock(&self.s1_inflight).insert(n.clone()) {
                got.insert(n.clone(), Err("the previous S1 read has not returned".into()));
                continue;
            }
            let (src, inflight, tx, name) = (self.s1.clone(), self.s1_inflight.clone(), tx.clone(), n.clone());
            let spawned = std::thread::Builder::new().name("meter-s1".into()).spawn(move || {
                let r = src.read(&name);
                lock(&inflight).remove(&name);
                let _ = tx.send((name, r));
            });
            match spawned {
                Ok(_) => waiting += 1,
                Err(e) => {
                    lock(&self.s1_inflight).remove(n);
                    got.insert(n.clone(), Err(format!("cannot start the S1 read: {e}")));
                }
            }
        }
        drop(tx);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(self.tick_budget_ms);
        while waiting > 0 {
            match rx.recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())) {
                Ok((n, r)) => {
                    waiting -= 1;
                    got.insert(n, r);
                }
                Err(_) => break,
            }
        }
        let budget = self.tick_budget_ms;
        names.iter().map(|n| (n.clone(), got.remove(n).unwrap_or_else(|| Err(format!("the S1 read missed the tick budget ({budget} ms)"))))).collect()
    }

    /// AL-4: the conversation signal of `session` at `now` (mono).
    pub fn conversation(&self, session: &str, now: u64) -> Conversation {
        let (prompt, stopped, s2_ev, s2_any, s1, convs) = {
            let g = lock(&self.inner);
            let Some(s) = g.sessions.get(session) else { return Conversation::None };
            (s.last_prompt.clone(), s.stopped, s.s2_event_at, s.s2_at, s.s1.clone(), s.s3_by_conv.clone())
        };
        // `last_prompt` is persisted (CR-6.23, P-1), so after a restart on the
        // same boot it is read back. A session that never reported a prompt
        // (the built hooks send none until B-29) with S2 within 120 s still reads
        // as a person: unknown fails toward a person (T-3692 decision).
        if prompt.is_none() && s2_any.is_some_and(|t| now.saturating_sub(t) <= V19_MS) {
            return Conversation::Operator;
        }
        if let Some(p) = prompt
            && p.origin != Origin::Automated
        {
            // FCW-17: no stop yet -> 120 s after the last S2 event after the prompt.
            let base = stopped.unwrap_or_else(|| p.rx_mono.max(s2_ev.unwrap_or(0)));
            let mut end = base + V19_MS;
            if let Some(S1::Known { client: Some(c), .. }) = s1
                && c > p.rx_mono
            {
                end = end.max(c + V19_MS);
            }
            if now <= end {
                return Conversation::Operator;
            }
        }
        let wall = (self.wall)();
        let mut owed = vec![];
        for (c, t) in convs {
            if now.saturating_sub(t) > V19_MS {
                continue;
            }
            match self.ledger.owed(&c, wall) {
                Ok(true) => owed.push(c),
                Ok(false) => {}
                Err(e) => return Conversation::Unknown(e), // FCW-18
            }
        }
        if owed.is_empty() { Conversation::None } else { Conversation::Agent(owed) }
    }

    /// The READY record as the doorbell may read it: absent if received by
    /// another sidecar process (FCW-6).
    pub fn ready(&self, session: &str) -> Option<ReadyRec> {
        lock(&self.inner).sessions.get(session)?.ready.clone().filter(|r| r.epoch == self.epoch)
    }

    /// CR-6.4 (T-3694, E-153): the receiver-checked origin of an applied `prompt_submit`
    /// report, for the reply the hook claims with (AL-11). `None` for any other report.
    pub fn checked_origin_of(&self, raw: &[u8]) -> Option<(&'static str, String)> {
        let v: Value = serde_json::from_slice(raw).ok()?;
        if v.get("event").and_then(Value::as_str) != Some("prompt_submit") {
            return None;
        }
        let p = self.last_prompt(v.get("session_id").and_then(Value::as_str)?)?;
        Some((p.origin.name(), p.basis))
    }

    pub fn last_prompt(&self, session: &str) -> Option<Prompt> {
        lock(&self.inner).sessions.get(session)?.last_prompt.clone()
    }

    /// The live view of one session (AL-3 step 5..6).
    fn view(&self, session: &str, now: u64) -> Value {
        let conv = self.conversation(session, now);
        let g = lock(&self.inner);
        let Some(s) = g.sessions.get(session) else { return Value::Null };
        let age = |t: Option<u64>| t.map(|t| now.saturating_sub(t));
        let (s1_at, s1_state) = match &s.s1 {
            Some(S1::Known { client, .. }) => (json!(client), json!("known")),
            Some(S1::Unknown(why)) => (json!("unknown"), json!(format!("unknown: {why}"))),
            None => (json!("unknown"), json!("unknown: not read yet")),
        };
        let s1_known = matches!(s.s1, Some(S1::Known { .. }));
        let tr_unreadable = s.transcript.is_some() && s.tr_unreadable;
        let s2_known = s2_known(s, now);
        // last = max of the KNOWN sources; an unknown source never reads as silence.
        let mut cands: Vec<(u64, &str)> = vec![];
        if let Some(S1::Known { client: Some(c), .. }) = s.s1 {
            cands.push((c, "s1"));
        }
        if s2_known && let Some(t) = s.s2_at {
            cands.push((t, "s2"));
        }
        if let Some(t) = s.s3_at {
            cands.push((t, "s3"));
        }
        let last = cands.iter().max_by_key(|(t, _)| *t);
        let watchdog = match (s1_known, s2_known) {
            (true, true) => "available".to_string(),
            (false, _) => "unavailable: S1 unknown".into(),
            (true, false) if tr_unreadable => "unavailable: S2 unknown (transcript unreadable and no fresh report)".into(),
            (true, false) => "unavailable: S2 unknown (no adapter report and no transcript)".into(),
        };
        json!({
            "s1_at": s1_at, "s1_age_ms": if s1_known { json!(age(match s.s1 { Some(S1::Known { client, .. }) => client, _ => None })) } else { json!("unknown") },
            "s1": s1_state,
            "s2_at": if s2_known { json!(s.s2_at) } else { json!("unknown") },
            "s2_age_ms": if s2_known { json!(age(s.s2_at)) } else { json!("unknown") },
            "s3_at": s.s3_at, "s3_age_ms": age(s.s3_at),
            "last": last.map(|(t, _)| *t), "last_age_ms": last.map(|(t, _)| now.saturating_sub(*t)),
            "source": last.map(|(_, s)| *s),
            "conversation": conv.view(),
            "watchdog": watchdog,
            "ready": s.ready.as_ref().map(|r| json!({"state": r.state, "at_ms": r.at_ms, "ord": r.ord.view(), "rx_mono": r.rx_mono, "seq": r.seq, "sidecar_epoch": r.epoch, "current_epoch": r.epoch == self.epoch})),
            "last_prompt": s.last_prompt.as_ref().map(|p| json!({"origin": p.origin.name(), "basis": p.basis, "at_ms": p.at_ms, "ord": p.ord.view(), "rx_mono": p.rx_mono, "doorbell_ids": p.doorbell_ids})),
            "turn_stopped": s.stopped,
            "interactive": s.interactive,
            "transcript": s.transcript.is_some(),
            "foreground": s.s1_sources.as_ref().map(|r| r.foreground.clone()),
            "ended": s.ended,
        })
    }

    /// Status `activity` (6.14.5, OBS-5): per session the tick's snapshot with
    /// its stamp; one older than [`STALE_MS`] reads `unknown` in every field,
    /// never its last value (FCW-15).
    pub fn status_view(&self) -> Value {
        let now = (self.clock)();
        let g = lock(&self.inner);
        let mut sessions = serde_json::Map::new();
        for (k, s) in &g.sessions {
            let v = match &s.snapshot {
                Some((v, at)) if now.saturating_sub(*at) <= STALE_MS => {
                    let mut v = v.clone();
                    v["stamped_at"] = json!(at);
                    v["stamp_age_ms"] = json!(now.saturating_sub(*at));
                    v
                }
                Some((v, at)) => {
                    let mut u = serde_json::Map::new();
                    for key in v.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
                        u.insert(key, json!("unknown"));
                    }
                    u.insert("stamped_at".into(), json!(at));
                    u.insert("stamp_age_ms".into(), json!(now.saturating_sub(*at)));
                    u.insert("stale".into(), json!(true));
                    Value::Object(u)
                }
                None => json!({"state": "unknown", "why": "no meter tick since this session was seen"}),
            };
            sessions.insert(k.clone(), v);
        }
        let c = &g.counters;
        json!({
            "sessions": sessions,
            "mono_now": now,
            "sidecar_epoch": self.epoch,
            "counters": {
                "adapter_reports_unparseable": c.unparseable, "ready_reordered": c.ready_reordered, "prompt_reordered": c.prompt_reordered,
                "origin_inconsistent": c.origin_inconsistent, "origin_overridden": c.origin_overridden,
                "transcript_refused": c.transcript_refused, "session_table_full": c.session_table_full,
                "s3_unattributed": c.s3_unattributed, "persist_failed": c.persist_failed,
                "restored": c.restored, "last_reasons": c.reasons,
            },
        })
    }
}

/// FCW-13; F-28 (CR-6.25): a transcript whose lstat failed is not a known S2
/// source, so S2 is known then only on a fresh report.
fn s2_known(s: &Session, now: u64) -> bool {
    let report_fresh = s.last_report.is_some_and(|t| now.saturating_sub(t) <= S2_FRESH_MS);
    if s.transcript.is_some() && s.tr_unreadable { report_fresh } else { s.reports > 0 || s.transcript.is_some() }
}

/// What the doorbell reads of one session on a tick (B-16).
#[derive(Debug, Clone, PartialEq)]
pub struct DoorbellFacts {
    pub name: String,
    pub ended: bool,
    pub harness_pid: Option<u32>,
    /// `None`: S1 unknown; `Some(last client input)` otherwise.
    pub s1: Option<Option<u64>>,
    pub s2_known: bool,
    /// (s1, s2, s3) last event, mono.
    pub acts: (Option<u64>, Option<u64>, Option<u64>),
    /// The READY record with "received by this process" (FCW-6).
    pub ready: Option<(ReadyRec, bool)>,
    /// The last prompt submit: (rx mono, the doorbell ids it named).
    pub last_submit: Option<(u64, Vec<String>)>,
    pub conversation: Conversation,
}

impl Meter {
    /// T-3772: what session binding reads, without the doorbell's facts:
    /// `(session, ended, harness_pid)` for every session.
    pub fn binding_facts(&self) -> Vec<(String, bool, Option<u32>)> {
        lock(&self.inner).sessions.iter().map(|(k, s)| (k.clone(), s.ended, s.harness_pid)).collect()
    }

    /// B-16: every session's facts for the doorbell tick, read under one lock
    /// each (the conversation signal outside it, AL-4).
    pub fn doorbell_facts(&self, now: u64) -> Vec<DoorbellFacts> {
        let names: Vec<String> = lock(&self.inner).sessions.keys().cloned().collect();
        let mut out = vec![];
        for n in names {
            let conversation = self.conversation(&n, now);
            let g = lock(&self.inner);
            let Some(s) = g.sessions.get(&n) else { continue };
            let s1 = match &s.s1 {
                Some(S1::Known { client, .. }) => Some(*client),
                _ => None,
            };
            out.push(DoorbellFacts {
                name: n.clone(),
                ended: s.ended,
                harness_pid: s.harness_pid,
                s1,
                s2_known: s2_known(s, now),
                acts: (s1.flatten(), s.s2_at, s.s3_at),
                ready: s.ready.clone().map(|r| {
                    let cur = r.epoch == self.epoch;
                    (r, cur)
                }),
                last_submit: s.last_prompt.as_ref().map(|p| (p.rx_mono, p.doorbell_ids.clone())),
                conversation,
            });
        }
        out
    }

    /// B-16: the monotonic clock the meter stamps with.
    pub fn now(&self) -> u64 {
        (self.clock)()
    }
}

/// The conversation signal (AL-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conversation {
    None,
    Operator,
    /// The conversations under way (their mail is rung as normal; others held).
    Agent(Vec<String>),
    /// The ledger could not be read (FCW-18): the doorbell treats it as `operator`.
    Unknown(String),
}

impl Conversation {
    pub fn view(&self) -> Value {
        match self {
            Conversation::None => json!("none"),
            Conversation::Operator => json!("operator"),
            Conversation::Agent(c) => json!({"agent": c}),
            Conversation::Unknown(why) => json!({"unknown": why}),
        }
    }
}

/// S2 (a): a turn event at `rx` (also S2 overall).
fn stamp_s2(sess: &mut Session, rx: u64) {
    sess.s2_at = Some(sess.s2_at.map_or(rx, |t| t.max(rx)));
    sess.s2_event_at = Some(sess.s2_event_at.map_or(rx, |t| t.max(rx)));
}

/// The first turn stop (or READY) ordered after the last prompt (AL-4 line 510, CR-6.24).
fn mark_stopped(sess: &mut Session, ord: &ReportOrd, rx: u64) {
    if let Some(p) = &sess.last_prompt
        && sess.stopped.is_none()
        && ord.after(&p.ord)
    {
        sess.stopped = Some(rx);
    }
}

fn valid_ids(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .filter(|s| (1..=128).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
                .take(256)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// AL-1 check_origin: the receiver only ever downgrades an origin.
///
/// T-3694 (E-153, FCW-195, CR-6.4): for a session whose PTY counters read (same boot), a
/// `human` prompt with no `client` byte since the previous submit of the session (`prev`), or
/// none since the last line the receiver typed into it (`last_typed`), is `automated`;
/// `client_auto` (terminal replies) is never a client byte.
#[allow(clippy::too_many_arguments)]
fn check_origin(
    c: &mut Counters,
    o: &str,
    b: &str,
    ids: &[String],
    typed: &[String],
    s1: Option<Result<S1Reply, String>>,
    rx: u64,
    prev: Option<u64>,
    last_typed: Option<u64>,
) -> (Origin, String) {
    let ok = match o {
        "human" => ["pty-input", "inferred"].contains(&b),
        "automated" => ["doorbell", "non-interactive", "harness-mark", "pty-input"].contains(&b),
        "unknown" => b == "absent",
        _ => false,
    };
    if !ok {
        c.origin_inconsistent += 1; // FCW-4 (case-sensitive: "Human" lands here)
        return (Origin::Unknown, "absent".into());
    }
    let origin = match o {
        "human" => Origin::Human,
        "automated" => Origin::Automated,
        _ => Origin::Unknown,
    };
    if origin == Origin::Human && ids.iter().any(|i| typed.contains(i)) {
        c.origin_overridden += 1; // FCW-5
        return (Origin::Automated, "doorbell".into());
    }
    // Every `human` origin of a PTY session is checked against the counters,
    // basis `inferred` included (F-4, CR-6.4); `client_auto` is not input.
    if origin == Origin::Human {
        let my_boot = termlink_session::input_meter::boot_id();
        let counted = match &s1 {
            Some(Ok(r)) if my_boot.is_some() && r.boot_id == my_boot => Some(r),
            _ => None,
        };
        let src = counted.and_then(|r| {
            ["client", "notify", "inject"]
                .iter()
                .filter_map(|s| r.at(s).filter(|t| *t <= rx).map(|t| (t, *s)))
                .max_by_key(|(t, _)| *t)
                .map(|(_, s)| s)
        });
        if let Some(r) = counted {
            let client = r.at("client").filter(|t| *t <= rx);
            let none_since = |mark: Option<u64>| mark.is_some_and(|m| client.is_none_or(|t| t <= m));
            if none_since(last_typed) {
                c.origin_overridden += 1; // FCW-195: follows a line the receiver typed, no client byte after it
                return (Origin::Automated, "doorbell".into());
            }
            if client.is_none() || none_since(prev) {
                c.origin_overridden += 1; // FCW-195: no client byte since the previous submit
                return (Origin::Automated, "pty-input".into());
            }
        }
        match src {
            Some("notify" | "inject") => {
                c.origin_overridden += 1; // FCW-5
                return (Origin::Automated, "pty-input".into());
            }
            Some(_) => {}
            None if b == "pty-input" => return (Origin::Human, "inferred".into()), // cannot confirm pty-input
            None => {} // no PTY counters (or no input yet): `inferred` stays as reported
        }
    }
    (origin, b.to_owned())
}

/// Read back [`STATE_NAME`]: the sessions of this boot, or why none are used.
fn restore(raw: &str, boot: Option<&str>) -> Result<Vec<(String, Session)>, String> {
    let v = strict_json(raw.as_bytes())?;
    if v.get("v").and_then(Value::as_u64) != Some(1) {
        return Err("unknown version".into());
    }
    let Some(boot) = boot else { return Err("this host has no boot id".into()) };
    if v.get("boot_id").and_then(Value::as_str) != Some(boot) {
        return Err("written on another boot".into());
    }
    let sessions = v.get("sessions").and_then(Value::as_object).ok_or("no sessions")?;
    let mut out = vec![];
    for (k, e) in sessions {
        if !valid_session(k) {
            return Err(format!("bad session id {k:?}"));
        }
        let u = |o: &Value, f: &str| o.get(f).and_then(Value::as_u64).ok_or(format!("{k}: {f} is not an integer"));
        let ready = match e.get("ready") {
            None | Some(Value::Null) => None,
            Some(r) => {
                let state = r.get("state").and_then(Value::as_str).filter(|x| ["READY", "BUSY", "NOT_RUNNING"].contains(x)).ok_or(format!("{k}: bad ready state"))?;
                let rx_mono = u(r, "rx_mono")?;
                // A record written before P-2 has no `ord`: its receive stamp orders it.
                let ord = r.get("ord").map_or(Some(ReportOrd::rx(rx_mono)), ReportOrd::from_view).ok_or(format!("{k}: bad ord"))?;
                Some(ReadyRec { state: state.into(), at_ms: u(r, "at_ms")?, ord, rx_mono, seq: u(r, "seq")?, epoch: r.get("epoch").and_then(Value::as_str).and_then(|x| x.parse().ok()).ok_or(format!("{k}: bad epoch"))? })
            }
        };
        let last_prompt = match e.get("last_prompt") {
            None | Some(Value::Null) => None,
            Some(p) => {
                let origin = match p.get("origin").and_then(Value::as_str) {
                    Some("human") => Origin::Human,
                    Some("automated") => Origin::Automated,
                    Some("unknown") => Origin::Unknown,
                    _ => return Err(format!("{k}: bad origin")),
                };
                let basis = p.get("basis").and_then(Value::as_str).ok_or(format!("{k}: no basis"))?.to_owned();
                let rx_mono = u(p, "rx_mono")?;
                let ord = p.get("ord").map_or(Some(ReportOrd::rx(rx_mono)), ReportOrd::from_view).ok_or(format!("{k}: bad ord"))?;
                Some(Prompt { origin, basis, at_ms: u(p, "at_ms")?, ord, rx_mono, doorbell_ids: valid_ids(p.get("doorbell_ids")) })
            }
        };
        let sess = Session {
            ready,
            last_prompt,
            stopped: e.get("stopped").and_then(Value::as_u64),
            ended: e.get("ended").and_then(Value::as_bool).unwrap_or(false),
            ..Session::default()
        };
        out.push((k.clone(), sess));
    }
    Ok(out)
}

fn rand_epoch() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut h = std::collections::hash_map::RandomState::new().build_hasher();
    h.write_u64(crate::store::now_ms());
    h.write_u32(std::process::id());
    h.finish()
}
