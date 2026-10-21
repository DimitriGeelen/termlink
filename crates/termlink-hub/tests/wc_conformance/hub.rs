//! Hub backend adapter for AEF's shared worker-contract vectors (T-3517, wc/0.3).
//!
//! Each vector's `given` is seeded into a fresh temp-dir bus; each `steps` op is applied through
//! the bus-level worker-contract API (`Bus::wc_*`, `resolve_at`) with injected time. Vector
//! symbols (n1, c1, E1, ...) are bound to the values the store returns. The adapter never
//! decides a verdict itself: it reports what the store did, and an op or surface the store does
//! not expose is `Fail::Missing` (reported not-implemented), never a pass.
//!
//! Mapping decisions:
//! - Time: vector seconds become `BASE_MS + s * 1000`. `G`, `deadline_dispatched` and
//!   `deadline_running` are the hub's own durations. The constants are private to the crate, so
//!   they are mirrored here: spawn 60 s + G 120 s, worker timeout 3600 s + slack 600 s.
//! - `advance` moves the clock, then runs the periodic sweeps (state sweep with the registry as
//!   last observed, and the answer sweep). The vectors model the sweep as the writer of T4/T6.
//! - History is the state sequence from the state the attempt had when seeded. Refusal rows,
//!   question rows and answer rows are not state changes and are left out. A `running -> running`
//!   reclaim row is a state entry, as the vectors expect.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde_json::{Map, Value, json};
use termlink_bus::Bus;
use termlink_bus::worker_contract::answer::{AckRequest, AnswerRequest, AskRequest, ResolveRequest, resolve_at};
use termlink_bus::worker_contract::sweep::{SweepObservation, SweepReport};
use termlink_bus::worker_contract::{AllocateRequest, Allocation, Handback, StopRequest, TransitionRequest, WcError};
use termlink_hub::dispatch_contract::{AGENT_SESSION_REFUSAL, ResolveSession, resolve_gate};

/// Capabilities this backend offers (vocabulary.capabilities). `lease` and `real_worker` are not.
pub const CAPABILITIES: [&str; 4] = ["registry_epoch", "answer_channel", "caller_chosen_ids", "supervisor_gen"];

const BASE_MS: i64 = 1_000_000_000;
const GRACE_MS: i64 = 120_000;
const SPAWN_MS: i64 = 60_000;
const WORKER_TIMEOUT_MS: i64 = 3_600_000;
const RUNNING_SLACK_MS: i64 = 600_000;
const ANSWER_TTL_S: i64 = 600;

/// Why a step or check did not produce a verdict.
#[derive(Debug)]
pub enum Fail {
    /// The backend answered, and the answer differs from the vector.
    Mismatch(String),
    /// The backend cannot express the op or surface; names what is missing.
    Missing(String),
}

pub type Res<T> = Result<T, Fail>;

fn missing<T>(what: &str) -> Res<T> {
    Err(Fail::Missing(what.to_string()))
}

fn at(sec: i64) -> i64 {
    BASE_MS + sec * 1000
}

fn obj(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// The result shape for a refused or failed call. Fence reasons use their wire names.
fn error_map(e: &WcError) -> Map<String, Value> {
    match e {
        WcError::IdTaken(_) => obj(json!({"error": "id_taken", "values": {"substituted_id": null}})),
        WcError::BadId(_) => obj(json!({"error": "malformed_id"})),
        WcError::NotFound(_) => obj(json!({"error": "not_found"})),
        WcError::Invalid(_) => obj(json!({"error": "invalid"})),
        WcError::Fence { reason, .. } => obj(json!({"fence_reject": reason.as_str()})),
        WcError::Bus(_) => obj(json!({"error": "bus"})),
    }
}

/// Transition code for a `to: done` write, by the state it was made against (§3.3).
fn done_code(prior: &str) -> &'static str {
    match prior {
        "dispatched" => "T3b",
        "running" => "T3",
        "lost" => "T5",
        "unknown" => "T7b",
        _ => "?",
    }
}

/// Sweep transition code (§3.3), from the state change and its cause.
fn sweep_code(from: &str, to: &str) -> &'static str {
    match (from, to) {
        ("running" | "dispatched", "lost") => "T4",
        ("unknown", "lost") => "T7c",
        ("running", "unknown") => "T6",
        ("dispatched", "unknown") => "T6b",
        ("unknown", "running") => "T7a",
        _ => "?",
    }
}

/// A duration parameter (`G`, `deadline_running+1`) in seconds, as the hub defines it.
fn duration_ms(param: &str) -> Res<i64> {
    let (name, off) = match param.split_once('+') {
        Some((n, o)) => match o.parse::<i64>() {
            Ok(v) => (n, v),
            Err(_) => return missing(&format!("parameter offset {param}")),
        },
        None => (param, 0),
    };
    let secs = match name {
        "G" => 120,
        "deadline_dispatched" => SPAWN_MS / 1000 + 120,
        "deadline_running" => (WORKER_TIMEOUT_MS + RUNNING_SLACK_MS) / 1000,
        other => return missing(&format!("parameter {other} (no hub duration for it)")),
    };
    Ok((secs + off) * 1000)
}

/// One supervisor write, fully resolved to the values the store checks.
#[derive(Clone, Debug)]
pub struct Complete {
    dispatch: String,
    attempt: i64,
    nonce: String,
    token: Option<String>,
    sgen: Option<i64>,
    exit_code: Option<i64>,
    refused: bool,
    digest: String,
    handback: Option<Handback>,
    persisted_at: Option<i64>,
}

/// Runs one completion write against `bus` and maps the answer to a result.
fn complete_once(bus: &Bus, c: &Complete, now: i64) -> Map<String, Value> {
    let Some(view) = attempt_view(bus, &c.dispatch, c.attempt) else {
        return error_map(&WcError::NotFound(c.dispatch.clone()));
    };
    let prior = view["state"].as_str().unwrap_or("").to_string();
    let req = TransitionRequest {
        dispatch_id: c.dispatch.clone(),
        attempt: c.attempt,
        nonce: c.nonce.clone(),
        supervisor_token: c.token.clone(),
        expected_prior: prior.clone(),
        to: "done".into(),
        supervisor_gen: c.sgen.unwrap_or_else(|| view["supervisor_gen"].as_i64().unwrap_or(1)),
        evidence_digest: Some(c.digest.clone()),
        exit_code: c.exit_code,
        refused: c.refused,
        persisted_at: c.persisted_at,
        handback: c.handback.clone(),
        ..Default::default()
    };
    match bus.wc_transition(&req, now) {
        Ok(out) if out.deduped => obj(json!({"response": "T8"})),
        Ok(_) => obj(json!({"accepted": done_code(&prior)})),
        Err(e) => error_map(&e),
    }
}

/// The attempt object as `dispatch.get` shows it.
fn attempt_view(bus: &Bus, dispatch: &str, n: i64) -> Option<Value> {
    let d = bus.wc_get(dispatch).ok()?;
    d["attempts"].as_array()?.iter().find(|a| a["attempt"].as_i64() == Some(n)).cloned()
}

/// A lifecycle history row is a state entry. Refusals, questions and answers are not.
fn lifecycle_row(row: &Value) -> bool {
    if row.get("fence_reason").is_some_and(|f| !f.is_null()) {
        return false;
    }
    let cause = row["cause"].as_str().unwrap_or("");
    !(cause.starts_with("question_asked") || cause.starts_with("answer_"))
}

/// The per-process bus this adapter drives, one per vector run.
pub struct Hub {
    dir: tempfile::TempDir,
    bus: Bus,
    clock: i64,
    dispatch: Option<String>,
    last_created: Option<String>,
    /// Vector symbol -> bound value (nonce, credential).
    sym: HashMap<String, String>,
    nonce_of: HashMap<i64, String>,
    token_of: HashMap<i64, String>,
    nonces_seen: Vec<String>,
    evidence: HashMap<String, Value>,
    /// Registrations the registry currently shows live, as (dispatch, attempt).
    live: HashSet<(String, i64)>,
    last_listing: bool,
    /// attempt -> (state it was seeded or created in, history rows at that point).
    base: HashMap<i64, (String, usize)>,
    /// Operator notices the seeded record already carries (reaching `unknown` or `lost` raises
    /// some). The vectors count what the steps add, so this is subtracted.
    base_notices: i64,
}

impl Hub {
    /// A fresh hub in a temp dir, seeded from one vector's `given`.
    pub fn reset(given: &Value) -> Res<Hub> {
        let dir = tempfile::tempdir().map_err(|e| Fail::Mismatch(format!("temp dir: {e}")))?;
        let bus = Bus::open(dir.path().join("bus")).map_err(|e| Fail::Mismatch(format!("bus open: {e}")))?;
        let mut h = Hub {
            dir,
            bus,
            clock: at(0),
            dispatch: None,
            last_created: None,
            sym: HashMap::new(),
            nonce_of: HashMap::new(),
            token_of: HashMap::new(),
            nonces_seen: Vec::new(),
            evidence: HashMap::new(),
            live: HashSet::new(),
            last_listing: true,
            base: HashMap::new(),
            base_notices: 0,
        };
        h.seed(given)?;
        Ok(h)
    }

    fn root(&self) -> PathBuf {
        self.dir.path().join("bus")
    }

    fn resolve_sym(&self, s: &str) -> String {
        self.sym.get(s).cloned().unwrap_or_else(|| s.to_string())
    }

    fn dispatch_id(&self) -> Res<String> {
        self.dispatch.clone().ok_or_else(|| Fail::Missing("a dispatch (none was given or created)".into()))
    }

    /// Records a newly allocated attempt: its nonce and credential, and where its history starts.
    fn bind_new(&mut self, a: &Allocation) {
        self.nonces_seen.push(a.nonce.clone());
        self.nonce_of.insert(a.attempt, a.nonce.clone());
        self.token_of.insert(a.attempt, a.supervisor_token.clone());
        let rows = self.rows(&a.dispatch_id, a.attempt);
        self.base.insert(a.attempt, ("dispatched".into(), rows));
    }

    fn rows(&self, dispatch: &str, n: i64) -> usize {
        attempt_view(&self.bus, dispatch, n).and_then(|v| v["history"].as_array().map(Vec::len)).unwrap_or(0)
    }

    fn state_of(&self, n: i64) -> Res<String> {
        let d = self.dispatch_id()?;
        attempt_view(&self.bus, &d, n)
            .and_then(|v| v["state"].as_str().map(str::to_string))
            .ok_or_else(|| Fail::Missing(format!("attempt {n} of the case dispatch")))
    }

    fn seed(&mut self, given: &Value) -> Res<()> {
        let now = at(given["clock"].as_i64().unwrap_or(0));
        self.clock = now;
        if let Some(ev) = given["evidence"].as_object() {
            for (k, v) in ev {
                self.evidence.insert(k.clone(), v.clone());
            }
        }
        let Some(dispatch) = given.get("dispatch") else { return Ok(()) };
        let id = dispatch["dispatch_id"].as_str().unwrap_or_default().to_string();
        let session = dispatch["dispatcher_session"].as_str().map(str::to_string);
        let mut attempts: Vec<&Value> = given["attempts"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
        attempts.sort_by_key(|a| a["attempt"].as_i64().unwrap_or(0));
        for a in attempts {
            let n = a["attempt"].as_i64().unwrap_or(1);
            let state = a["state"].as_str().unwrap_or("dispatched");
            // Each seeded end is reached on the clock, so the seeded record is internally consistent.
            let t0 = match state {
                "lost" => at(a["lost_at"].as_i64().unwrap_or(0)) - GRACE_MS - 10_000,
                "unknown" => now - (WORKER_TIMEOUT_MS + RUNNING_SLACK_MS) - 1_000,
                _ => now,
            };
            let req = if n == 1 {
                AllocateRequest { dispatch_id: Some(id.clone()), dispatcher_session: session.clone(), ..Default::default() }
            } else {
                AllocateRequest { attempt_of: Some(id.clone()), ..Default::default() }
            };
            let alloc = self
                .bus
                .wc_allocate(&req, t0)
                .map_err(|e| Fail::Mismatch(format!("seed attempt {n}: allocate refused: {e:?}")))?;
            if alloc.attempt != n {
                return Err(Fail::Mismatch(format!("seed attempt {n}: store allocated attempt {}", alloc.attempt)));
            }
            self.dispatch = Some(id.clone());
            if let Some(s) = a["nonce"].as_str() {
                self.sym.insert(s.to_string(), alloc.nonce.clone());
            }
            if let Some(s) = a["credential"].as_str() {
                self.sym.insert(s.to_string(), alloc.supervisor_token.clone());
            }
            self.bind_new(&alloc);
            match state {
                "dispatched" => {}
                "running" => {
                    self.start_seed(n, t0)?;
                    if a.get("seen_alive_epoch").is_some() {
                        self.live.insert((id.clone(), n));
                        self.sweep_seed(t0, true)?;
                    }
                }
                "done" => {
                    self.start_seed(n, t0)?;
                    let ev = a["evidence"].as_str().unwrap_or_default().to_string();
                    let c = self.complete_for(&ev, n, None, None)?;
                    let out = complete_once(&self.bus, &c, t0);
                    if out.get("accepted").is_none() {
                        return Err(Fail::Mismatch(format!("seed attempt {n}: completion refused: {out:?}")));
                    }
                }
                "lost" => {
                    let lost = at(a["lost_at"].as_i64().unwrap_or(0));
                    self.start_seed(n, t0)?;
                    self.live.insert((id.clone(), n));
                    self.sweep_seed(t0, true)?;
                    self.live.remove(&(id.clone(), n));
                    self.sweep_seed(lost - GRACE_MS, true)?;
                    self.sweep_seed(lost, true)?;
                }
                "unknown" => {
                    self.start_seed(n, t0)?;
                    self.live.remove(&(id.clone(), n));
                    self.sweep_seed(now, true)?;
                }
                other => return missing(&format!("given state {other}")),
            }
            let reached = self.state_of(n)?;
            if reached != state {
                return Err(Fail::Mismatch(format!("seed attempt {n}: store reached {reached}, vector says {state}")));
            }
            let rows = self.rows(&id, n);
            self.base.insert(n, (reached, rows));
        }
        self.clock = now;
        self.base_notices = self.notice_total();
        Ok(())
    }

    /// Operator notices on the case dispatch, as the store counts them (0 before a dispatch exists).
    fn notice_total(&self) -> i64 {
        match &self.dispatch {
            Some(d) => self.bus.wc_operator_notice_count(d, None).unwrap_or(0),
            None => 0,
        }
    }

    fn start_seed(&mut self, n: i64, t: i64) -> Res<()> {
        let d = self.dispatch_id()?;
        let req = TransitionRequest {
            dispatch_id: d.clone(),
            attempt: n,
            nonce: self.nonce_of[&n].clone(),
            supervisor_token: self.token_of.get(&n).cloned(),
            expected_prior: "dispatched".into(),
            to: "running".into(),
            supervisor_gen: 1,
            ..Default::default()
        };
        self.bus
            .wc_transition(&req, t)
            .map(|_| ())
            .map_err(|e| Fail::Mismatch(format!("seed attempt {n}: start refused: {e:?}")))
    }

    fn sweep_seed(&mut self, t: i64, listing: bool) -> Res<SweepReport> {
        let obs = SweepObservation { listing_complete: listing, live: self.live.clone() };
        self.bus.wc_sweep(&obs, t).map_err(|e| Fail::Mismatch(format!("seed sweep refused: {e:?}")))
    }

    /// A completion built from a vector evidence symbol, for seeding or a step.
    fn complete_for(&self, ev_sym: &str, n: i64, nonce: Option<String>, token: Option<Option<String>>) -> Res<Complete> {
        let ev = self
            .evidence
            .get(ev_sym)
            .ok_or_else(|| Fail::Mismatch(format!("vector names evidence {ev_sym} that is not in given")))?;
        let handback = ev.get("handback").and_then(Value::as_object).map(|h| Handback {
            attempt: h.get("attempt").and_then(Value::as_i64),
            nonce: h.get("nonce").and_then(Value::as_str).map(|s| self.resolve_sym(s)),
            status: h.get("status").and_then(Value::as_str).map(str::to_string),
            summary: None,
        });
        Ok(Complete {
            dispatch: self.dispatch_id()?,
            attempt: n,
            nonce: nonce.unwrap_or_else(|| self.nonce_of[&n].clone()),
            token: token.unwrap_or_else(|| self.token_of.get(&n).cloned()),
            sgen: None,
            exit_code: ev.get("exit_code").and_then(Value::as_i64),
            refused: ev.get("supervisor_refused").and_then(Value::as_bool).unwrap_or(false),
            digest: format!("ev:{ev_sym}"),
            handback,
            persisted_at: ev.get("persisted_at").and_then(Value::as_i64).map(at),
        })
    }

    /// Applies one vector step through the store and returns its result.
    ///
    /// `notices` in a step's result is how many operator notices the step raised: the difference of
    /// the store's durable count across the op. Nothing here decides whether a notice is due.
    pub fn apply(&mut self, step: &Value) -> Res<Map<String, Value>> {
        let before = self.notice_total();
        let mut res = self.apply_op(step)?;
        if self.dispatch.is_some() {
            res.insert("notices".into(), json!(self.notice_total() - before));
        }
        Ok(res)
    }

    fn apply_op(&mut self, step: &Value) -> Res<Map<String, Value>> {
        let op = step["op"].as_str().unwrap_or_default();
        let args = &step["args"];
        let n = step["attempt"].as_i64().unwrap_or(1);
        match op {
            "create" => self.create(args),
            "allocate" => self.allocate(step),
            "start" => self.start(step, n),
            "complete" => self.complete(step, n),
            "reclaim" => self.reclaim(step, n),
            "append_question" => self.ask(step, n),
            "append_answer" => self.answer(step, n),
            "ack_answer" => self.ack(step, n),
            "sweep" => {
                let d = self.dispatch_id()?;
                if args["observe"].as_str() == Some("alive") {
                    self.live.insert((d, n));
                } else {
                    self.live.remove(&(d, n));
                }
                self.last_listing = true;
                let r = self.sweep_now(true)?;
                Ok(r)
            }
            "probe_undecided" => {
                self.last_listing = false;
                self.sweep_now(false)
            }
            "hub_restart" => {
                self.bus.wc_begin_hub_epoch().map_err(|e| Fail::Mismatch(format!("hub restart: {e:?}")))?;
                Ok(obj(json!({"ok": true})))
            }
            "advance" => self.advance(args),
            // No write happens for these: the vector only says the world changed.
            "kill_supervisor" | "dispatcher_lost" => Ok(obj(json!({"ok": true}))),
            "concurrent" => self.concurrent(step),
            "resolve" => self.resolve(args),
            "partition" | "heal" => missing(&format!("op:{op} (the hub has no model of worker-host contact)")),
            "hold_lock" => missing("op:hold_lock (the per-write lock is SQLite IMMEDIATE; no hook to hold it)"),
            "confirm_end" => self.confirm_end(n),
            "stop" => self.stop(step, n),
            "ask_wait" | "wait" => missing(&format!("op:{op} (no worker-side wait or collect surface)")),
            "consume" => missing("op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store)"),
            "adopt" => missing("op:adopt (no dispatcher-ownership transfer)"),
            "inspect" => missing(&format!("op:inspect surface {}", args["surface"].as_str().unwrap_or("?"))),
            other => missing(&format!("op:{other}")),
        }
    }

    fn create(&mut self, args: &Value) -> Res<Map<String, Value>> {
        let req = AllocateRequest {
            dispatch_id: args["dispatch_id"].as_str().map(str::to_string),
            retry_of_dispatch_id: args["retry_of_dispatch_id"].as_str().map(str::to_string),
            ..Default::default()
        };
        match self.bus.wc_allocate(&req, self.clock) {
            Ok(a) => {
                self.last_created = Some(a.dispatch_id.clone());
                if self.dispatch.is_none() {
                    self.dispatch = Some(a.dispatch_id.clone());
                    self.bind_new(&a);
                }
                Ok(obj(json!({"accepted": "T1", "values": {"dispatch_id": a.dispatch_id}})))
            }
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn allocate(&mut self, step: &Value) -> Res<Map<String, Value>> {
        let d = self.dispatch_id()?;
        let req = AllocateRequest { attempt_of: Some(d), ..Default::default() };
        match self.bus.wc_allocate(&req, self.clock) {
            Ok(a) => {
                let fresh = !self.nonces_seen.contains(&a.nonce);
                self.bind_new(&a);
                if let Some(bind) = step["expect"]["bind"].as_object() {
                    for (field, sym) in bind {
                        let Some(sym) = sym.as_str() else { continue };
                        match field.as_str() {
                            "nonce" => self.sym.insert(sym.to_string(), a.nonce.clone()),
                            "credential" => self.sym.insert(sym.to_string(), a.supervisor_token.clone()),
                            _ => None,
                        };
                    }
                }
                Ok(obj(json!({"accepted": "T1", "values": {"attempt": a.attempt, "fresh_nonce": fresh}})))
            }
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn nonce_for(&self, step: &Value, n: i64) -> Res<String> {
        match step.get("nonce").and_then(Value::as_str) {
            Some(s) => Ok(self.resolve_sym(s)),
            None => self.nonce_of.get(&n).cloned().ok_or_else(|| Fail::Missing(format!("nonce of attempt {n}"))),
        }
    }

    /// T-3523 (CC-25): the backend's stop records its cause. `exit_status_captured` is the
    /// world's fact, not a store input: it decides which confirmed end follows (T3 or T4).
    fn stop(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        let req = StopRequest {
            dispatch_id: self.dispatch_id()?,
            attempt: n,
            supervisor_token: self.token_for(step, n),
            cause: step["args"]["cause"].as_str().unwrap_or_default().to_string(),
        };
        match self.bus.wc_stop(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"ok": true}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn token_for(&self, step: &Value, n: i64) -> Option<String> {
        match step.get("credential").and_then(Value::as_str) {
            Some("none") => None,
            Some(s) => Some(self.resolve_sym(s)),
            None => self.token_of.get(&n).cloned(),
        }
    }

    fn start(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        let d = self.dispatch_id()?;
        let prior = self.state_of(n)?;
        let req = TransitionRequest {
            dispatch_id: d,
            attempt: n,
            nonce: self.nonce_for(step, n)?,
            supervisor_token: self.token_for(step, n),
            expected_prior: prior,
            to: "running".into(),
            supervisor_gen: step["supervisor_gen"].as_i64().unwrap_or(1),
            ..Default::default()
        };
        match self.bus.wc_transition(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"accepted": "T2", "ok": true}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn complete(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        if let Some(b) = step["args"]["backend"].as_str() {
            if b != "hub" {
                return missing("refusal:backend_fixed (dispatch.transition has no backend address; every write is the hub's)");
            }
        }
        let ev = step["evidence"].as_str().unwrap_or_default().to_string();
        let nonce = self.nonce_for(step, n)?;
        let token = self.token_for(step, n);
        let mut c = self.complete_for(&ev, n, Some(nonce), Some(token))?;
        c.sgen = step["supervisor_gen"].as_i64();
        Ok(complete_once(&self.bus, &c, self.clock))
    }

    /// The hub's end confirmation (AEF `confirm_end`, §3.2). The store refuses a finished attempt;
    /// otherwise the sweep runs twice at the current epoch, then once more G later on the injected
    /// clock. Nothing is written here: a state change is only the sweep's own (T4).
    fn confirm_end(&mut self, n: i64) -> Res<Map<String, Value>> {
        let d = self.dispatch_id()?;
        // The attempt's own registration is absent: that is what the vector's end means.
        let live: HashSet<(String, i64)> = self.live.iter().filter(|k| !(k.0 == d && k.1 == n)).cloned().collect();
        let obs = SweepObservation { listing_complete: true, live };
        let mut report = match self.bus.wc_confirm_end(&d, n, &obs, self.clock) {
            Ok(r) => r,
            Err(e) => return Ok(error_map(&e)),
        };
        for later in [0, GRACE_MS] {
            self.clock += later;
            let r = self
                .bus
                .wc_sweep(&obs, self.clock)
                .map_err(|e| Fail::Mismatch(format!("confirm_end sweep refused: {e:?}")))?;
            report.transitions.extend(r.transitions);
            report.notices.extend(r.notices);
            report.reconciliation_requested.extend(r.reconciliation_requested);
        }
        Ok(sweep_result(&report))
    }

    fn reclaim(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        if step["args"].get("holder").is_some() {
            return missing("capability:lease (reclaim by lease holder; this backend has supervisor_gen fencing only)");
        }
        let d = self.dispatch_id()?;
        let prior = self.state_of(n)?;
        let req = TransitionRequest {
            dispatch_id: d,
            attempt: n,
            nonce: self.nonce_for(step, n)?,
            supervisor_token: self.token_for(step, n),
            expected_prior: prior,
            to: "running".into(),
            supervisor_gen: step["supervisor_gen"].as_i64().unwrap_or(0),
            ..Default::default()
        };
        match self.bus.wc_transition(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"accepted": "T-RECLAIM"}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn ask(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        let d = self.dispatch_id()?;
        let req = AskRequest {
            dispatch_id: d,
            attempt: n,
            nonce: self.nonce_for(step, n)?,
            question_id: step["args"]["question_id"].as_str().map(str::to_string),
            body: step["args"]["body"].as_str().unwrap_or_default().to_string(),
        };
        match self.bus.wc_ask(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"ok": true}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn answer(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        // The store's answer TTL is fixed; a vector deadline must agree with it or the step is not judged.
        if let Some(dl) = step["args"]["deadline"].as_i64() {
            if dl != self.clock / 1000 - BASE_MS / 1000 + ANSWER_TTL_S {
                return missing("answer deadline argument (the store fixes the answer TTL)");
            }
        }
        let d = self.dispatch_id()?;
        let req = AnswerRequest {
            dispatch_id: d,
            attempt: n,
            supervisor_token: self.token_for(step, n),
            cmd_id: step["args"]["cmd_id"].as_str().map(str::to_string),
            question_id: step["args"]["question_id"].as_str().map(str::to_string),
            body: step["args"]["body"].as_str().unwrap_or_default().to_string(),
        };
        match self.bus.wc_answer(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"ok": true}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn ack(&mut self, step: &Value, n: i64) -> Res<Map<String, Value>> {
        let d = self.dispatch_id()?;
        let req = AckRequest {
            dispatch_id: d,
            attempt: n,
            nonce: self.nonce_for(step, n)?,
            cmd_id: step["args"]["cmd_id"].as_str().unwrap_or_default().to_string(),
            state: step["args"]["state"].as_str().unwrap_or("applied").to_string(),
        };
        match self.bus.wc_ack(&req, self.clock) {
            Ok(_) => Ok(obj(json!({"ok": true}))),
            Err(e) => Ok(error_map(&e)),
        }
    }

    fn sweep_now(&mut self, listing: bool) -> Res<Map<String, Value>> {
        let report = self.sweep_seed_at(self.clock, listing)?;
        Ok(sweep_result(&report))
    }

    fn sweep_seed_at(&mut self, t: i64, listing: bool) -> Res<SweepReport> {
        let obs = SweepObservation { listing_complete: listing, live: self.live.clone() };
        self.bus.wc_sweep(&obs, t).map_err(|e| Fail::Mismatch(format!("sweep refused: {e:?}")))
    }

    fn advance(&mut self, args: &Value) -> Res<Map<String, Value>> {
        let ms = match &args["by"] {
            Value::Number(n) => n.as_i64().unwrap_or(0) * 1000,
            Value::String(p) => duration_ms(p)?,
            _ => return missing("advance without a duration"),
        };
        self.clock += ms;
        self.bus
            .wc_sweep_answers(self.clock)
            .map_err(|e| Fail::Mismatch(format!("answer sweep refused: {e:?}")))?;
        self.sweep_now(self.last_listing)
    }

    fn concurrent(&mut self, step: &Value) -> Res<Map<String, Value>> {
        let count = step["args"]["count"].as_u64().unwrap_or(0) as usize;
        let inner = &step["args"]["step"];
        if inner["op"].as_str() != Some("complete") {
            return missing("concurrent over an op other than complete");
        }
        let n = inner["attempt"].as_i64().unwrap_or(1);
        let c = self.complete_for(
            inner["evidence"].as_str().unwrap_or_default(),
            n,
            Some(self.nonce_for(inner, n)?),
            Some(self.token_for(inner, n)),
        )?;
        let root = self.root();
        let now = self.clock;
        let results: Vec<Value> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..count)
                .map(|_| {
                    let root = root.clone();
                    let c = c.clone();
                    s.spawn(move || -> Value {
                        // One connection per writer, as separate supervisors would have.
                        match Bus::open(&root) {
                            Ok(bus) => Value::Object(complete_once(&bus, &c, now)),
                            Err(e) => json!({"error": format!("bus open: {e}")}),
                        }
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap_or_else(|_| json!({"error": "writer panicked"}))).collect()
        });
        Ok(obj(json!({"results": results})))
    }

    fn resolve(&mut self, args: &Value) -> Res<Map<String, Value>> {
        if args.get("material").is_some() {
            return missing("T7d material binding (ResolveRequest carries no material or outcome)");
        }
        if args["via"].as_str() == Some("remote") {
            // resolve is not an RPC: resolve_on_host is reached only from the host CLI, so no remote
            // caller can reach it. The refusal holds by construction.
            return Ok(obj(json!({"refused": "not_available"})));
        }
        // T-3527: the host CLI's speed bump, the same function `resolve_on_host` runs. A vector
        // `override: false` (or none) is no override; a string is the recorded reason.
        let session = ResolveSession {
            agent: args["session"].as_str() == Some("agent"),
            override_reason: args["override"].as_str().map(str::to_string),
        };
        let actor = match resolve_gate(&session) {
            Ok(a) => a,
            Err(_) => return Ok(obj(json!({"refused": AGENT_SESSION_REFUSAL}))),
        };
        let d = self.dispatch_id()?;
        let n = args["attempt"].as_i64().unwrap_or(1);
        let reason = args["reason"].as_str().unwrap_or_default().to_string();
        let req = ResolveRequest {
            dispatch_id: d.clone(),
            attempt: n,
            as_state: args["as"].as_str().unwrap_or_default().to_string(),
            reason: reason.clone(),
            actor,
        };
        match resolve_at(&self.root(), &req, self.clock) {
            Ok(_) => {
                // What the history row recorded, read back rather than echoed.
                let row = attempt_view(&self.bus, &d, n)
                    .and_then(|a| a["history"].as_array().and_then(|h| h.last().cloned()))
                    .unwrap_or(Value::Null);
                let cause = row["cause"].as_str().unwrap_or_default();
                let logged_reason = cause.rsplit_once(": ").map(|(_, r)| r.to_string());
                Ok(obj(json!({
                    "accepted": "T7d",
                    "values": {
                        "logged_actor": row["writer"].as_str().is_some_and(|w| !w.is_empty()),
                        "logged_reason": logged_reason,
                    },
                })))
            }
            Err(e) => Ok(error_map(&e)),
        }
    }

    /// The observable record: per-attempt fields and history, plus dispatch-level facts.
    pub fn snapshot(&self) -> Res<Value> {
        let mut attempts = Map::new();
        let mut dispatcher_session = Value::Null;
        let mut attempt_count = 0;
        // T-3528 (CC-12): dispatch-level bounce facts, read from the record as served.
        let (mut bounced, mut ladder) = (Value::Null, Value::Null);
        let mut causes = Map::new();
        if let Some(d) = &self.dispatch {
            let v = self.bus.wc_get(d).map_err(|e| Fail::Mismatch(format!("get refused: {e:?}")))?;
            dispatcher_session = v["dispatcher"]["session"].clone();
            bounced = v["bounced_to_dispatcher"].clone();
            ladder = v["retry_ladder_entries"].clone();
            let list = v["attempts"].as_array().cloned().unwrap_or_default();
            attempt_count = list.len();
            for a in &list {
                let n = a["attempt"].as_i64().unwrap_or(0);
                let (base_state, base_rows) = self.base.get(&n).cloned().unwrap_or(("dispatched".into(), 0));
                let mut history = vec![Value::String(base_state)];
                let mut last_cause = Value::Null;
                for row in a["history"].as_array().map(Vec::as_slice).unwrap_or_default().iter().skip(base_rows) {
                    if lifecycle_row(row) {
                        history.push(row["to"].clone());
                        last_cause = row["cause"].clone();
                    }
                }
                // T-3523 (CC-25): the cause on the attempt's latest lifecycle entry.
                causes.insert(format!("A{n}"), last_cause);
                let answers: Map<String, Value> = a["answers"]
                    .as_array()
                    .map(|xs| {
                        xs.iter()
                            .filter_map(|x| Some((x["cmd_id"].as_str()?.to_string(), x["state"].clone())))
                            .collect()
                    })
                    .unwrap_or_default();
                attempts.insert(
                    format!("A{n}"),
                    json!({
                        "state": a["state"],
                        "outcome": a["outcome"],
                        "exit_code": a["exit_code"],
                        "supervisor_gen": a["supervisor_gen"],
                        "attested": a["attested"],
                        "attested_by": a["attested_by"],
                        "result_kind": a["result"]["kind"],
                        "evidence_digest": a["evidence_digest"],
                        "handback_status": a["handback_status"],
                        "questions": a["questions"].as_array().map_or(0, Vec::len),
                        "answers": answers,
                        "history": history,
                    }),
                );
            }
        }
        let retry_of = match &self.last_created {
            Some(id) => self
                .bus
                .wc_get(id)
                .map(|v| v["retry_of_dispatch_id"].clone())
                .unwrap_or(Value::Null),
            None => Value::Null,
        };
        Ok(json!({
            "attempts": Value::Object(attempts),
            "attempt_count": attempt_count,
            "dispatch_count": self.dispatch_count()?,
            "dispatcher_session": dispatcher_session,
            "new_dispatch_retry_of": retry_of,
            "operator_notices": self.notice_total() - self.base_notices,
            "bounced_to_dispatcher": bounced,
            "retry_ladder_entries": ladder,
            "history_causes": Value::Object(causes),
        }))
    }

    fn dispatch_count(&self) -> Res<i64> {
        let conn = rusqlite::Connection::open_with_flags(
            self.root().join("meta.db"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| Fail::Mismatch(format!("open store read-only: {e}")))?;
        conn.query_row("SELECT COUNT(*) FROM wc_dispatches", [], |r| r.get(0))
            .map_err(|e| Fail::Mismatch(format!("count dispatches: {e}")))
    }
}

/// A sweep's result: a state change is a transition code; otherwise the notices it raised.
fn sweep_result(report: &SweepReport) -> Map<String, Value> {
    let mut out = obj(json!({"ok": true}));
    match report.transitions.first() {
        Some(t) if report.transitions.len() == 1 => {
            out.insert("accepted".into(), json!(sweep_code(&t.from, &t.to)));
        }
        Some(_) => {
            // More than one state write in a single sweep: the vectors never do this, so no
            // single code can stand for it. Left unreported rather than guessed.
            out.insert("accepted".into(), json!("multiple"));
        }
        None => {
            out.insert("no_transition".into(), json!(true));
        }
    }
    out.insert(
        "values".into(),
        json!({
            "observation_logged": !report.notices.is_empty(),
            "reconciliation_requested": !report.reconciliation_requested.is_empty(),
        }),
    );
    out
}
