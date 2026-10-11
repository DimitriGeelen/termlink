//! C-9 claim and deliver (architecture note sections 4.6, 6.4, 7.1, D-4; SI-30,
//! SI-17, FC-4, FC-11, FC-13, CR-17).
//!
//! `pull` is what the agent's hook calls at a tool call. It reads the delivery
//! switch and re-runs the receiver checks on every call (the second run of 7.1:
//! it catches a switch flipped or a policy changed since the message was
//! stored). It offers only messages that are STORED here, allow-listed, not yet
//! handed over and without an open attempt, urgent first. For each it takes an
//! `O_CREAT|O_EXCL` claim (`claims/<msg_id>.<attempt>`) holding the nonce and the
//! time *before* any content leaves the store, records CLAIMED, and only then
//! builds the frame. Exactly one caller wins a claim, so concurrent pulls cannot
//! hand out one message twice, and a killed sidecar's claim still blocks a
//! second hand-over after restart.
//!
//! `finalize` decides what happened to an attempt from the agent's transcript
//! (`adapter::claude::find`); nothing the hook or the agent says can set
//! HANDED_OVER:
//! - evidence found: HANDED_OVER, reported to the sender as a signed stage;
//! - unreadable: UNCERTAIN plus needs-attention, the attempt stays open;
//! - proven absent (completeness point reached): ABSENT closes the attempt, and
//!   the message is offered again as attempt n+1;
//! - absent without a completeness point: nothing yet; on the final check
//!   UNCERTAIN, and the attempt stays open for the operator. A timeout alone
//!   never re-delivers.
//!
//! Attempt bookkeeping is on the chain: an attempt is open from its claim until
//! an ABSENT line for it (reason `attempt=<n>`), or forever once HANDED_OVER.

use crate::adapter::claude::{self, Evidence};
use crate::frame::{self, Frame, Item};
use crate::policy::{self, Assessment, Policy};
use crate::send::Sender;
use crate::store::{ChainLine, ClaimOutcome, LineExtra, Store};
use crate::wire::{self, Msg, Stage};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

pub const CLAIMED: &str = "CLAIMED";
pub const HANDED_OVER: &str = "HANDED_OVER";
pub const UNCERTAIN: &str = "UNCERTAIN";
pub const ABSENT: &str = "ABSENT";

/// Conversations that hold no messages (switch log, unparseable refusals).
const NOT_MAIL: [&str; 2] = [crate::api::ADMIN_CONV, crate::pull::UNPARSED_CONV];

/// One claimed message inside an attempt record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Claimed {
    pub conv: String,
    pub msg_id: String,
    pub sender: String,
    pub attempt: u32,
    pub digest: String,
}

/// What one `pull` handed out, kept until the finalizer is done with it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttemptRecord {
    pub nonce: String,
    pub session: String,
    pub hook: String,
    pub tool_use_id: Option<String>,
    pub ts_ms: u64,
    pub items: Vec<Claimed>,
}

#[derive(Debug)]
pub enum Pulled {
    /// Nothing to deliver; `why` says whether that is because delivery is off.
    Nothing { why: &'static str },
    Frame { frame: Frame, waiting: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    HandedOver,
    Absent,
    Uncertain(String),
    /// No conclusion yet; ask again.
    Pending,
}

struct Candidate {
    conv: String,
    msg_id: String,
    sender: String,
    digest: String,
    seq: u64,
    next_attempt: u32,
    msg: Msg,
    assessment: Assessment,
}

pub struct Deliverer {
    agent: String,
    store: Arc<Store>,
    policy: Box<dyn Policy>,
    sender: Option<Arc<Sender>>,
}

fn attempt_reason(attempt: u32, nonce: &str) -> String {
    format!("attempt={attempt} nonce={nonce}")
}

fn attempt_of(l: &ChainLine) -> Option<u32> {
    l.reason
        .as_deref()?
        .split_whitespace()
        .find_map(|w| w.strip_prefix("attempt="))?
        .parse()
        .ok()
}

/// An inbound record line: STORED written by this store, not a peer's report.
fn own(l: &ChainLine) -> bool {
    l.peer_ts_ms.is_none()
}

impl Deliverer {
    pub fn new(agent: &str, store: Arc<Store>, policy: Box<dyn Policy>, sender: Option<Arc<Sender>>) -> Self {
        Deliverer { agent: agent.to_owned(), store, policy, sender }
    }

    /// Bind a session id to its transcript (the SessionStart hook does this in B10).
    pub fn register_session(&self, session: &str, transcript: &std::path::Path) -> Result<(), String> {
        let p = transcript.to_str().ok_or("transcript path is not UTF-8")?;
        self.store.write_state(&format!("session-{session}"), p).map_err(|e| e.to_string())
    }

    fn transcript_of(&self, session: &str) -> Option<PathBuf> {
        self.store
            .read_state(&format!("session-{session}"))
            .ok()
            .flatten()
            .map(|s| PathBuf::from(s.trim_end()))
    }

    /// Everything that may be offered now, in delivery order (urgent first, then
    /// oldest first).
    fn candidates(&self) -> Vec<Candidate> {
        let mut out = vec![];
        for conv in self.store.conversations() {
            if NOT_MAIL.contains(&conv.as_str()) {
                continue;
            }
            let Ok(lines) = self.store.chain_lines(&conv) else { continue };
            for st in lines.iter().filter(|l| l.event == "STORED" && own(l)) {
                let mine = |ev: &'static str| {
                    lines
                        .iter()
                        .filter(move |l| l.event == ev && own(l) && l.msg_id == st.msg_id && l.sender == st.sender)
                };
                if mine(HANDED_OVER).next().is_some() || mine("REFUSED").next().is_some() {
                    continue;
                }
                let Ok(attempts) = self.store.claims_for(&st.msg_id) else { continue };
                let closed: BTreeSet<u32> = mine(ABSENT).filter_map(attempt_of).collect();
                if attempts.iter().any(|a| !closed.contains(a)) {
                    continue; // an attempt is open: never a second frame (FC-4)
                }
                // A record that fails its checksum is quarantined by the read and skipped.
                let Ok((_, payload)) = self.store.read_record(&conv, st.seq, &st.msg_id) else { continue };
                let Ok(msg) = wire::parse_payload(&payload) else { continue };
                if msg.from != st.sender || msg.msg_id != st.msg_id {
                    let _ = self.store.attention(
                        "record-mismatch",
                        &format!("{conv}/{}: stored payload does not match its chain line", st.msg_id),
                    );
                    continue;
                }
                // Second run of the receiver checks (7.1).
                let assessment = self.policy.assess(&msg);
                if assessment.withheld.is_some() {
                    continue;
                }
                out.push(Candidate {
                    conv: conv.clone(),
                    msg_id: st.msg_id.clone(),
                    sender: st.sender.clone(),
                    digest: st.digest.clone(),
                    seq: st.seq,
                    next_attempt: attempts.last().copied().unwrap_or(0) + 1,
                    msg,
                    assessment,
                });
            }
        }
        out.sort_by_key(|c| (!c.assessment.urgent, c.msg.ts_ms, c.seq));
        out
    }

    /// Messages that would be offered now (for status and tests).
    pub fn pending(&self) -> Vec<String> {
        self.candidates().into_iter().map(|c| c.msg_id).collect()
    }

    /// The hook's call at a tool call: claim and frame what is deliverable.
    pub fn pull(&self, session: &str, hook: &str, tool_use_id: Option<&str>) -> Result<Pulled, String> {
        // Read now, never cached (SI-23).
        if !policy::delivery_enabled(&self.store) {
            return Ok(Pulled::Nothing { why: "delivery-off" });
        }
        let cands = self.candidates();
        if cands.is_empty() {
            return Ok(Pulled::Nothing { why: "none-pending" });
        }
        let nonce = wire::mint_msg_id();
        let now_ms = crate::store::now_ms();
        let mut won: Vec<&Candidate> = vec![];
        let mut size = 400usize;
        let mut waiting = 0usize;
        for c in &cands {
            let add = c.msg.body.len() + 300;
            if !won.is_empty() && size + add > frame::FRAME_CAP {
                waiting += 1;
                continue;
            }
            match self.store.claim(&c.msg_id, c.next_attempt, &nonce, now_ms / 1000) {
                Ok(ClaimOutcome::Won) => {
                    size += add;
                    won.push(c);
                }
                Ok(ClaimOutcome::Held) => {} // another pull has it
                Err(e) => {
                    let _ = self.store.attention("claim-failed", &format!("{}: {e}", c.msg_id));
                }
            }
        }
        if won.is_empty() {
            return Ok(Pulled::Nothing { why: "none-pending" });
        }
        let rec = AttemptRecord {
            nonce: nonce.clone(),
            session: session.to_owned(),
            hook: hook.to_owned(),
            tool_use_id: tool_use_id.map(str::to_owned),
            ts_ms: now_ms,
            items: won
                .iter()
                .map(|c| Claimed {
                    conv: c.conv.clone(),
                    msg_id: c.msg_id.clone(),
                    sender: c.sender.clone(),
                    attempt: c.next_attempt,
                    digest: c.digest.clone(),
                })
                .collect(),
        };
        // The claims are taken; if what follows fails, they stay open and block a
        // second frame (fail closed) and the operator is told.
        let json = serde_json::to_string(&rec).map_err(|e| e.to_string())?;
        if let Err(e) = self.store.write_state(&format!("delivery-{nonce}"), &json) {
            let _ = self.store.attention("attempt-unrecorded", &format!("nonce {nonce}: {e}; claims stay open"));
            return Err(format!("attempt record: {e}"));
        }
        for c in &won {
            let extra = LineExtra { reason: Some(attempt_reason(c.next_attempt, &nonce)), ..Default::default() };
            if let Err(e) = self.store.append_event_unless(&c.conv, CLAIMED, &c.msg_id, &c.sender, &c.digest, extra, |_| false) {
                let _ = self.store.attention("claim-unlogged", &format!("{}: {e}", c.msg_id));
                return Err(format!("CLAIMED not recorded: {e}"));
            }
        }
        let items: Vec<Item<'_>> = won.iter().map(|c| Item { msg: &c.msg, assessment: &c.assessment }).collect();
        let frame = frame::build(&items, &nonce, waiting);
        Ok(Pulled::Frame { frame, waiting })
    }

    /// Claim one message by id (the `claim` op): the atomic claim alone.
    pub fn claim_one(&self, msg_id: &str) -> Result<(u32, String), String> {
        if !policy::delivery_enabled(&self.store) {
            return Err("delivery-off".into());
        }
        let c = self
            .candidates()
            .into_iter()
            .find(|c| c.msg_id == msg_id)
            .ok_or("not-deliverable")?;
        let nonce = wire::mint_msg_id();
        let now_ms = crate::store::now_ms();
        match self.store.claim(&c.msg_id, c.next_attempt, &nonce, now_ms / 1000) {
            Ok(ClaimOutcome::Won) => {}
            Ok(ClaimOutcome::Held) => return Err("held".into()),
            Err(e) => return Err(e.to_string()),
        }
        let rec = AttemptRecord {
            nonce: nonce.clone(),
            session: "claim-op".into(),
            hook: "claim".into(),
            tool_use_id: None,
            ts_ms: now_ms,
            items: vec![Claimed {
                conv: c.conv.clone(),
                msg_id: c.msg_id.clone(),
                sender: c.sender.clone(),
                attempt: c.next_attempt,
                digest: c.digest.clone(),
            }],
        };
        let json = serde_json::to_string(&rec).map_err(|e| e.to_string())?;
        self.store.write_state(&format!("delivery-{nonce}"), &json).map_err(|e| e.to_string())?;
        let extra = LineExtra { reason: Some(attempt_reason(c.next_attempt, &nonce)), ..Default::default() };
        self.store
            .append_event_unless(&c.conv, CLAIMED, &c.msg_id, &c.sender, &c.digest, extra, |_| false)
            .map_err(|e| e.to_string())?;
        Ok((c.next_attempt, nonce))
    }

    pub fn attempt(&self, nonce: &str) -> Option<AttemptRecord> {
        let s = self.store.read_state(&format!("delivery-{nonce}")).ok()??;
        serde_json::from_str(&s).ok()
    }

    /// Decide an attempt from the registered transcript of its session.
    /// `final_check`: the finalizer's last look; absence without a completeness
    /// point then becomes UNCERTAIN.
    pub fn finalize(&self, nonce: &str, final_check: bool) -> Result<Verdict, String> {
        let rec = self.attempt(nonce).ok_or("unknown nonce")?;
        let ev = match self.transcript_of(&rec.session) {
            Some(p) => claude::find(&p, &frame::header_prefix(nonce), rec.tool_use_id.as_deref()),
            None => Evidence::Unreadable(format!("no transcript registered for session {}", rec.session)),
        };
        match ev {
            Evidence::Found { .. } => {
                for it in &rec.items {
                    self.line_once(it, HANDED_OVER, &attempt_reason(it.attempt, nonce))?;
                    self.report_handed_over(it);
                }
                Ok(Verdict::HandedOver)
            }
            Evidence::Absent { complete: true } => {
                for it in &rec.items {
                    if !self.handed(it) {
                        self.line_once(it, ABSENT, &attempt_reason(it.attempt, nonce))?;
                    }
                }
                Ok(Verdict::Absent)
            }
            Evidence::Unreadable(why) => {
                self.uncertain(&rec, nonce, &why)?;
                Ok(Verdict::Uncertain(why))
            }
            Evidence::Absent { complete: false } if final_check => {
                let why = "no evidence and no completeness point: the hand-over cannot be proven either way".to_owned();
                self.uncertain(&rec, nonce, &why)?;
                Ok(Verdict::Uncertain(why))
            }
            Evidence::Absent { complete: false } => Ok(Verdict::Pending),
        }
    }

    fn handed(&self, it: &Claimed) -> bool {
        self.store
            .chain_lines(&it.conv)
            .map(|ls| ls.iter().any(|l| l.event == HANDED_OVER && own(l) && l.msg_id == it.msg_id))
            .unwrap_or(false)
    }

    fn uncertain(&self, rec: &AttemptRecord, nonce: &str, why: &str) -> Result<(), String> {
        for it in &rec.items {
            if self.handed(it) {
                continue;
            }
            if self.line_once(it, UNCERTAIN, &attempt_reason(it.attempt, nonce))? {
                let _ = self.store.attention(
                    "hand-over-uncertain",
                    &format!("{}/{} attempt {}: {why}; the attempt stays open", it.conv, it.msg_id, it.attempt),
                );
            }
        }
        Ok(())
    }

    /// Append `event` for this attempt unless already there. True when written.
    fn line_once(&self, it: &Claimed, event: &'static str, reason: &str) -> Result<bool, String> {
        let attempt = it.attempt;
        let id = it.msg_id.clone();
        let extra = LineExtra { reason: Some(reason.to_owned()), ..Default::default() };
        self.store
            .append_event_unless(&it.conv, event, &it.msg_id, &it.sender, &it.digest, extra, move |ls| {
                ls.iter().any(|l| {
                    l.event == event
                        && own(l)
                        && l.msg_id == id
                        && (event == HANDED_OVER || attempt_of(l) == Some(attempt))
                })
            })
            .map(|r| r.is_some())
            .map_err(|e| format!("{event} not recorded: {e}"))
    }

    fn report_handed_over(&self, it: &Claimed) {
        let Some(sender) = &self.sender else { return };
        let Ok(lines) = self.store.chain_lines(&it.conv) else { return };
        let Some(st) = lines.iter().find(|l| l.event == "STORED" && own(l) && l.msg_id == it.msg_id) else { return };
        let Some(line) = lines.iter().find(|l| l.event == HANDED_OVER && own(l) && l.msg_id == it.msg_id) else { return };
        let Ok((_, payload)) = self.store.read_record(&it.conv, st.seq, &it.msg_id) else { return };
        let Ok(msg) = wire::parse_payload(&payload) else { return };
        let head = self.store.chain_head(&it.conv).unwrap_or_default();
        let stage = Stage {
            from: sender.fingerprint().to_owned(),
            from_agent: self.agent.clone(),
            to_agent: msg.from_agent.clone(),
            conv: it.conv.clone(),
            msg_id: it.msg_id.clone(),
            stage: HANDED_OVER.into(),
            ts_ms: line.ts_ms.unwrap_or(0) as i64,
            chain_head: head,
        };
        if let Err(e) = sender.post_stage(&msg.from_agent, &stage) {
            let _ = self.store.attention("stage-not-queued", &format!("HANDED_OVER for {}: {e}", it.msg_id));
        }
    }

    /// Open attempts per message (for status): msg_id -> attempts not closed.
    pub fn open_attempts(&self) -> BTreeMap<String, Vec<u32>> {
        let mut out = BTreeMap::new();
        for conv in self.store.conversations() {
            let Ok(lines) = self.store.chain_lines(&conv) else { continue };
            for l in lines.iter().filter(|l| l.event == CLAIMED && own(l)) {
                let Some(a) = attempt_of(l) else { continue };
                let closed = lines.iter().any(|x| {
                    own(x)
                        && x.msg_id == l.msg_id
                        && (x.event == HANDED_OVER || (x.event == ABSENT && attempt_of(x) == Some(a)))
                });
                if !closed {
                    out.entry(l.msg_id.clone()).or_insert_with(Vec::new).push(a);
                }
            }
        }
        out
    }
}
