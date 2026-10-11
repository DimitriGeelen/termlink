//! C-7 receiver checks (architecture note section 7; SI-18, SI-23, SI-8, allow-list).
//!
//! Inputs are deliberately few (SI-8): the verified sender fingerprint, the
//! built-in allow-list, and the signed priority. Nothing here reads a message
//! body, a conversation, or any field that could carry a grant, an approval or a
//! trust change; a static test (`policy_reads_only_sender_and_priority`) holds
//! this file to that.
//!
//! SI-18: priority is clamped to [-9, 9]. Trust class is computed, never read:
//! `own-project` when the sender is on the allow-list, `untrusted` otherwise.
//! Urgent means priority >= 5 AND allow-listed; a would-be-urgent message from
//! anyone else is downgraded to normal (a DOWNGRADED chain line, never a drop).
//!
//! Allow-list (7.2): PH-1 has a built-in, non-configurable one, the estate
//! roster. A sender outside it is stored, shown, and not delivered (REFUSED
//! `not-allowed`). This is the note's recommended answer to OQ-2, built pending
//! the operator's ruling.
//!
//! SI-23: delivery can be switched off from the admin socket. The switch is a
//! flag file read on every call (`delivery_enabled`), so there is no cached copy
//! to go stale between a flip and the next delivery attempt.

use crate::store::Store;
use crate::wire::{Msg, Roster};

/// Flag file in the store dir; present means delivery is off (SI-23).
pub const DELIVERY_OFF_FLAG: &str = "delivery.off";
/// The lowest priority that counts as urgent.
pub const URGENT_AT: i64 = 5;
/// Reason code on the REFUSED line of a sender outside the allow-list.
pub const NOT_ALLOWED: &str = "not-allowed";
/// Chain event recording an urgency downgrade.
pub const DOWNGRADED: &str = "DOWNGRADED";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    OwnProject,
    Untrusted,
}

impl Trust {
    pub fn as_str(self) -> &'static str {
        match self {
            Trust::OwnProject => "own-project",
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
    /// `None`: may be delivered. `Some(code)`: stored and shown, never delivered.
    pub withheld: Option<&'static str>,
}

/// The receiver-check hook point called by the receive path (C-6) before a
/// message is stored, and by the deliver path (C-9) again before a claim.
pub trait Policy: Send + Sync {
    fn assess(&self, msg: &Msg) -> Assessment;
}

/// The PH-1 policy: the allow-list is the roster, fixed at start.
pub struct BuiltIn {
    allow: Roster,
}

impl BuiltIn {
    pub fn new(allow: Roster) -> Self {
        BuiltIn { allow }
    }
}

/// Parse and clamp the signed priority field.
pub fn clamp_priority(p: &str) -> i64 {
    p.trim().parse::<i64>().map(|n| n.clamp(-9, 9)).unwrap_or(0)
}

impl Policy for BuiltIn {
    fn assess(&self, msg: &Msg) -> Assessment {
        let sender = msg.from.as_str();
        let priority = clamp_priority(&msg.priority);
        let trust = if self.allow.get(sender).is_some() { Trust::OwnProject } else { Trust::Untrusted };
        let asks_urgent = priority >= URGENT_AT;
        let urgent = asks_urgent && trust == Trust::OwnProject;
        Assessment {
            trust,
            priority,
            urgent,
            downgraded: asks_urgent && !urgent,
            withheld: match trust {
                Trust::OwnProject => None,
                Trust::Untrusted => Some(NOT_ALLOWED),
            },
        }
    }
}

/// SI-23: read the switch now. Absent flag means on; anything else, including
/// a flag that cannot be checked, means off (fail closed).
pub fn delivery_enabled(store: &Store) -> bool {
    match std::fs::symlink_metadata(store.dir().join(DELIVERY_OFF_FLAG)) {
        Ok(_) => false,
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}
