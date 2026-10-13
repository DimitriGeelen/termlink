//! C-7 receiver checks (architecture note section 7; SI-18, SI-23, SI-8, allow-list).
//!
//! Inputs are deliberately few (SI-8): the verified sender fingerprint, the
//! built-in allow-list, the operator's own lists, and the signed priority.
//! Nothing here reads a message body, a conversation, or any field that could
//! carry a grant, an admission or a trust change; a static test
//! (`policy_reads_only_sender_and_priority`) holds this file to that.
//!
//! SI-18: priority is clamped to [-9, 9]. Trust class is computed, never read:
//! `own-project` when the sender is on the roster, `admitted` when the operator
//! admitted its key, `untrusted` otherwise. Urgent means priority >= 5 AND on the
//! roster; a would-be-urgent message from anyone else is downgraded to normal (a
//! DOWNGRADED chain line, never a drop).
//!
//! Allow-list (7.2, OQ-2 ruled E on 2026-10-12): the roster is fixed at start. A
//! sender outside it is held for the operator (`awaiting-operator`): stored,
//! shown, never framed. Only the operator changes that, through the admin socket
//! (`allow` / `deny`), which writes the two lists read here on every call. A
//! message can never change them: nothing in a message reaches this file except
//! its verified sender and its priority.
//!
//! SI-23: delivery can be switched off from the admin socket. The switch is a
//! flag file read on every call (`delivery_enabled`), so there is no cached copy
//! to go stale between a flip and the next delivery attempt.

use crate::store::Store;
use crate::wire::{Msg, Roster};
use std::sync::Arc;

/// Flag file in the store dir; present means delivery is off (SI-23).
pub const DELIVERY_OFF_FLAG: &str = "delivery.off";
/// The lowest priority that counts as urgent.
pub const URGENT_AT: i64 = 5;
/// B8 reason code for a sender outside the allow-list, before ruling E.
pub const NOT_ALLOWED: &str = "not-allowed";
/// Reason on the HELD line: the receiver's operator has not decided yet.
pub const AWAITING_OPERATOR: &str = "awaiting-operator";
/// Reason on the REFUSED line once the operator has refused the sender.
pub const REFUSED_BY_OPERATOR: &str = "refused-by-operator";
/// Chain event recording an urgency downgrade.
pub const DOWNGRADED: &str = "DOWNGRADED";
/// Store state: the public keys (64 hex, one per line) the operator admitted.
pub const ADMITTED_KEYS: &str = "admitted-keys";
/// Store state: the fingerprints (one per line) the operator refused.
pub const REFUSED_KEYS: &str = "refused-keys";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    OwnProject,
    /// Outside the roster, admitted by the operator through the admin socket.
    Admitted,
    Untrusted,
}

impl Trust {
    pub fn as_str(self) -> &'static str {
        match self {
            Trust::OwnProject => "own-project",
            Trust::Admitted => "admitted-by-operator",
            Trust::Untrusted => "untrusted",
        }
    }
}

/// What the receiver checks decided for one verified message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    pub trust: Trust,
    /// The signed priority, clamped to [-9, 9]; 0 when it is not an integer.
    pub priority: i64,
    pub urgent: bool,
    /// It asked for urgent (priority >= 5) and did not get it.
    pub downgraded: bool,
    /// `None`: may be delivered. `Some(code)`: stored and shown, never delivered
    /// ([`AWAITING_OPERATOR`] or [`REFUSED_BY_OPERATOR`]).
    pub withheld: Option<&'static str>,
}

/// The receiver-check hook point called by the receive path (C-6) before a
/// message is stored, and by the deliver path (C-9) again before a claim.
pub trait Policy: Send + Sync {
    fn assess(&self, msg: &Msg) -> Assessment;
}

/// The PH-1 policy: the roster fixed at start, plus the operator's lists.
pub struct BuiltIn {
    allow: Roster,
    store: Option<Arc<Store>>,
}

impl BuiltIn {
    pub fn new(allow: Roster) -> Self {
        BuiltIn { allow, store: None }
    }

    /// Also read the operator's admitted and refused lists from `store`, on every call.
    pub fn with_operator_lists(mut self, store: Arc<Store>) -> Self {
        self.store = Some(store);
        self
    }
}

/// Parse and clamp the signed priority field.
pub fn clamp_priority(p: &str) -> i64 {
    p.trim().parse::<i64>().map(|n| n.clamp(-9, 9)).unwrap_or(0)
}

/// The keys the operator admitted. A list that cannot be read admits nobody.
pub fn admitted_keys(store: &Store) -> Roster {
    let mut r = Roster::new();
    if let Ok(Some(s)) = store.read_state(ADMITTED_KEYS) {
        for line in s.lines() {
            if let Some(pk) = crate::wire::parse_public_key(line.trim()) {
                r.add(pk);
            }
        }
    }
    r
}

/// The fingerprints the operator refused. A list that cannot be read refuses
/// nobody, which still delivers nothing: an unknown sender stays held.
pub fn refused_keys(store: &Store) -> Vec<String> {
    match store.read_state(REFUSED_KEYS) {
        Ok(Some(s)) => s.lines().map(|l| l.trim().to_owned()).filter(|l| !l.is_empty()).collect(),
        _ => vec![],
    }
}

impl Policy for BuiltIn {
    fn assess(&self, msg: &Msg) -> Assessment {
        let sender = msg.from.as_str();
        let priority = clamp_priority(&msg.priority);
        let refused = self.store.as_deref().is_some_and(|s| refused_keys(s).iter().any(|k| k == sender));
        let admitted = self.store.as_deref().is_some_and(|s| admitted_keys(s).get(sender).is_some());
        let trust = if self.allow.get(sender).is_some() {
            Trust::OwnProject
        } else if admitted && !refused {
            Trust::Admitted
        } else {
            Trust::Untrusted
        };
        let asks_urgent = priority >= URGENT_AT;
        let urgent = asks_urgent && trust == Trust::OwnProject;
        Assessment {
            trust,
            priority,
            urgent,
            downgraded: asks_urgent && !urgent,
            withheld: match trust {
                Trust::OwnProject | Trust::Admitted => None,
                Trust::Untrusted if refused => Some(REFUSED_BY_OPERATOR),
                Trust::Untrusted => Some(AWAITING_OPERATOR),
            },
        }
    }
}

/// SI-23: read the switch now. Absent flag means on; anything else, including
/// a flag that cannot be checked, means off (fail closed).
/// Flag file in the store dir; present means the sidecar is drained (PH-2 row
/// P4, FP 6.2.3.e): it pulls no new mail and hands nothing over, so an old
/// receiver can take the inbox back while exactly one receiver serves it.
pub const DRAINED_FLAG: &str = "receive.drained";

/// False while drained. Read on every pass (no cached copy), like the delivery flag.
pub fn receiving_enabled(store: &Store) -> bool {
    match std::fs::symlink_metadata(store.dir().join(DRAINED_FLAG)) {
        Ok(_) => false,
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}

pub fn delivery_enabled(store: &Store) -> bool {
    match std::fs::symlink_metadata(store.dir().join(DELIVERY_OFF_FLAG)) {
        Ok(_) => false,
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}
