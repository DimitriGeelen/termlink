//! The project's main lease as this sidecar sees it (T-3654, OD-18 / R-67, R-67.a1 (4)).
//!
//! A background thread reads the lease (`main:<project>` at the home hub, see
//! `termlink_session::main_lease`) every [`READ_EVERY_MS`]. It only READS: the sidecar never
//! renews the lease, because a sidecar-renewed lease proves a process, not a turn (R-67.c).
//! The session renews it through its own hook (`termlink main claim`).
//!
//! What the sidecar does with it:
//! * `status` carries `main.lease`: holder session, generation, since, pin, when checked.
//! * `pull` withholds role mail from a session that is not main when the lease names another
//!   session OF THIS AGENT as holder: the message stays stored (unread) for the main session,
//!   and the non-main session gets one line, once per holder generation: "held by main ...".
//!   Fail direction: when the lease is unknown (hub down, no project, stale read), vacant, or
//!   held by another agent's session, delivery is unchanged (pre-lease behaviour), so a lease
//!   problem can never stop all delivery. `am-i` stays fail-closed for hooks.
//!
//! Not built here (needs the cross-host forward of R-67.a1 (4) and AEF's hook, AEF T-5014):
//! forwarding to a main behind another sidecar and telling the sender
//! "delivered via X; main is Y (generation N)".

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use termlink_session::main_lease::{LeaseClient, Timing, View};

/// How often the lease is read.
pub const READ_EVERY_MS: u64 = 15_000;
/// A read older than this many intervals is stale and treated as unknown.
pub const STALE_READS: u64 = 4;

/// Test seam, honoured only in a test estate: `TERMLINK_SIDECAR_TEST_LEASE_EVERY_MS`.
pub fn every_ms(mode: &crate::estate::Mode) -> u64 {
    if !matches!(mode, crate::estate::Mode::Estate(_)) {
        return READ_EVERY_MS;
    }
    std::env::var("TERMLINK_SIDECAR_TEST_LEASE_EVERY_MS").ok().and_then(|v| v.parse().ok()).filter(|n| *n > 0).unwrap_or(READ_EVERY_MS)
}

#[derive(Default)]
struct Inner {
    /// (read at, the view) or (read at, why it could not be read).
    last: Option<(u64, Result<View, String>)>,
    /// (session, generation) pairs already told who main is.
    told: BTreeSet<(String, u64)>,
}

pub struct Shared {
    pub project: Option<String>,
    pub every_ms: u64,
    inner: Mutex<Inner>,
}

impl Shared {
    pub fn new(project: Option<String>, every_ms: u64) -> Arc<Shared> {
        Arc::new(Shared { project, every_ms, inner: Mutex::new(Inner::default()) })
    }

    pub fn set(&self, at: u64, r: Result<View, String>) {
        if let Ok(mut g) = self.inner.lock() {
            g.last = Some((at, r));
        }
    }

    fn fresh(&self, now: u64) -> Option<View> {
        let g = self.inner.lock().ok()?;
        match &g.last {
            Some((at, Ok(v))) if now.saturating_sub(*at) <= STALE_READS * self.every_ms => Some(v.clone()),
            _ => None,
        }
    }

    /// The fresh lease as the waiting-mail signal reads it (K-34, AL-12); `None` = unknown
    /// (not read, unreadable or stale).
    pub fn facts(&self, now: u64) -> Option<crate::mail_waiting::LeaseFacts> {
        self.fresh(now).map(|v| crate::mail_waiting::LeaseFacts::from_view(&v))
    }

    /// `main.lease` for `status`.
    pub fn status_view(&self, now: u64) -> Value {
        let Some(project) = &self.project else {
            return json!({"state": "unknown", "why": "no main_project configured for this sidecar"});
        };
        let g = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return json!({"state": "unknown", "why": "lease state lock poisoned"}),
        };
        match &g.last {
            None => json!({"state": "unknown", "project": project, "why": "not read yet"}),
            Some((at, Err(e))) => json!({"state": "unknown", "project": project, "checked_at_ms": at, "why": e}),
            Some((at, Ok(_))) if now.saturating_sub(*at) > STALE_READS * self.every_ms => {
                json!({"state": "unknown", "project": project, "checked_at_ms": at, "why": "last read is stale"})
            }
            Some((at, Ok(v))) => json!({
                // B-25 (T-3694): held | provisional | vacant | lapsed, and the holder's origin
                // and last human prompt (AL-12 Δ, for the waiting-mail signal AL-13).
                "state": v.state,
                "provisional": v.provisional,
                "origin": v.origin,
                "last_human_at_ms": v.last_human_at_ms,
                "holder_human_age_s": v.holder_human_age_s,
                // CR-6.21 (T-3694): the hub-clock last human prompt and the hub's now.
                "last_human_rx_ms": v.last_human_rx_ms,
                "hub_now_ms": v.hub_now_ms,
                "project": project,
                "topic": v.topic,
                "holder": v.holder,
                "generation": v.generation,
                "since_ms": v.since_ms,
                "claimed_until_ms": v.claimed_until_ms,
                "pinned": v.pinned,
                "eligible_at_ms": v.eligible_at_ms,
                "checked_at_ms": at,
            }),
        }
    }

    /// Should a pull by `session` of agent `agent` be withheld? `Some((line, notify_now))` when
    /// the fresh lease names another session of this agent as main.
    pub fn withhold(&self, agent: &str, session: &str, now: u64) -> Option<(String, bool)> {
        let v = self.fresh(now)?;
        let h = v.holder.as_ref()?;
        if h.session == session || (!h.agent.is_empty() && h.agent != agent) {
            return None;
        }
        let line = format!(
            "TermLink: this session is not main of {}; peer mail is left unread for main {} (generation {}). Ask with `termlink main am-i`.",
            v.project,
            h.display(),
            v.generation
        );
        let first = self.inner.lock().map(|mut g| g.told.insert((session.to_string(), v.generation))).unwrap_or(false);
        Some((line, first))
    }
}

/// Read the lease every `every_ms` on its own thread (with its own runtime) until `stop`.
pub fn spawn(client: LeaseClient, shared: Arc<Shared>, stop: Arc<AtomicBool>) -> std::io::Result<std::thread::JoinHandle<()>> {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    Ok(std::thread::spawn(move || {
        let Some(project) = shared.project.clone() else { return };
        let every = Duration::from_millis(shared.every_ms);
        let mut waited = every;
        while !stop.load(Ordering::SeqCst) {
            if waited >= every {
                waited = Duration::ZERO;
                let o = rt.block_on(client.status(&project, None));
                let at = crate::store::now_ms();
                match o.view {
                    Some(v) => shared.set(at, Ok(v)),
                    None => shared.set(at, Err(o.message)),
                }
            }
            std::thread::sleep(Duration::from_millis(100));
            waited += Duration::from_millis(100);
        }
    }))
}

/// A read-only lease client for the sidecar's hub (no identity: it never writes).
pub fn client(hub: termlink_protocol::TransportAddr) -> LeaseClient {
    LeaseClient { addr: hub, auth: None, identity: None, timing: timing(), obligation_topics: vec![] }
}

/// T-3686 (AL-16): the sidecar's read-only view never obeys a `TERMLINK_MAIN_TEST_*` seam
/// outside a test estate. It keeps the default timings and says so once in its log.
fn timing() -> Timing {
    static WARNED: std::sync::Once = std::sync::Once::new();
    Timing::from_env().unwrap_or_else(|e| {
        WARNED.call_once(|| eprintln!("termlink-sidecar: main lease view ignores {e}"));
        Timing::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use termlink_session::main_lease::{Folded, Holder, Row, fold, view};

    fn held_by(session: &str, agent: &str, generation: u64) -> View {
        let h = Holder { session: session.into(), agent: agent.into(), host: "h".into(), pid: 1 };
        let mut f: Folded = fold(&[]);
        f.generation = generation;
        let row = Row { claim_id: "c".into(), claimer: h.claimer(), claimed_at: 0, claimed_until: i64::MAX };
        view("p", &f, Some(&row), 10, Timing::default())
    }

    #[test]
    fn withholds_only_from_other_sessions_of_this_agent_and_tells_once_per_generation() {
        let sh = Shared::new(Some("p".into()), 1000);
        sh.set(100, Ok(held_by("S-main", "agent-a", 3)));
        assert!(sh.withhold("agent-a", "S-main", 200).is_none(), "main itself is never withheld");
        let (line, first) = sh.withhold("agent-a", "S-other", 200).expect("withheld");
        assert!(first);
        assert!(line.contains("S-main") && line.contains("generation 3"), "{line}");
        let (_, again) = sh.withhold("agent-a", "S-other", 300).unwrap();
        assert!(!again, "told once per generation");
        assert!(sh.withhold("agent-b", "S-other", 200).is_none(), "another agent's main does not gate this agent's mail");
        sh.set(400, Ok(held_by("S-main", "agent-a", 4)));
        assert!(sh.withhold("agent-a", "S-other", 500).unwrap().1, "a new generation is told again");
    }

    #[test]
    fn unknown_stale_or_vacant_lease_never_withholds() {
        let sh = Shared::new(Some("p".into()), 1000);
        assert!(sh.withhold("a", "s", 0).is_none(), "not read yet");
        sh.set(0, Err("hub down".into()));
        assert!(sh.withhold("a", "s", 10).is_none());
        assert_eq!(sh.status_view(10)["state"], "unknown");
        sh.set(0, Ok(held_by("S-main", "a", 1)));
        assert!(sh.withhold("a", "s", STALE_READS * 1000 + 1).is_none(), "stale read");
        assert_eq!(sh.status_view(STALE_READS * 1000 + 1)["state"], "unknown");
        let vacant = view("p", &fold(&[]), None, 10, Timing::default());
        sh.set(10, Ok(vacant));
        assert!(sh.withhold("a", "s", 20).is_none());
        assert_eq!(sh.status_view(20)["state"], "vacant");
        assert_eq!(Shared::new(None, 1000).status_view(0)["state"], "unknown");
    }
}
