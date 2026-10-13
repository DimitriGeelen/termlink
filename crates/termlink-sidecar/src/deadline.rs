//! The tick and the R-44.e deadlines (PH-2 row P2; operator rulings OQ-d A and OQ-c A).
//!
//! One tick is 30 s (R-17): every PH-2 bound counts in it. The 500 ms loop that
//! pulls, hands over and reads the kill switch stays inside it; the tick paces
//! only the slower checks (here: the receiver's overdue notices).
//!
//! STUCK means "the next step is overdue AND the responsible party is reachable
//! and alive" (R-44.a). Until a liveness signal exists (row P3), the sender only
//! calls a message STUCK on evidence from the responsible side itself: the
//! recipient reported RECEIVED but never STORED, or the recipient's sidecar sent
//! an overdue notice. Without that evidence an overdue step reads UNKNOWN.

/// One tick, in milliseconds (R-17).
pub const TICK_MS: u64 = 30_000;

/// Receiver notice: stored, and not handed over within the deadline after the
/// agent became ready.
pub const OVERDUE_HANDOVER: &str = "OVERDUE-HANDOVER";
/// Receiver notice: handed over, and not answered within the deadline while the
/// agent is ready.
pub const OVERDUE_ANSWER: &str = "OVERDUE-ANSWER";

/// Needs-attention kind for an urgent message overdue at the receiver: the one
/// immediate alarm R-37 allows. Kinds starting with `urgent-` are alarms.
pub const URGENT_OVERDUE: &str = "urgent-overdue";

/// The step a STUCK label names.
pub const STEP_NOT_ACCEPTED: &str = "not accepted";
pub const STEP_NOT_HANDED_OVER: &str = "not handed over";
pub const STEP_NOT_ANSWERED: &str = "not answered";

/// R-44.e deadlines, normal and urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadlines {
    pub accept_ms: u64,
    pub accept_urgent_ms: u64,
    pub handover_ticks: u64,
    pub handover_urgent_ticks: u64,
    pub answer_ms: u64,
    pub answer_urgent_ms: u64,
}

impl Deadlines {
    /// As R-44.e states them: accepted within 1 min (urgent 15 s); handed over
    /// within 2 ticks of the agent becoming ready (urgent: at the next tool call
    /// or turn end, counted as 1 tick); answered within 1 h (urgent 5 min).
    pub const R44E: Deadlines = Deadlines {
        accept_ms: 60_000,
        accept_urgent_ms: 15_000,
        handover_ticks: 2,
        handover_urgent_ticks: 1,
        answer_ms: 3_600_000,
        answer_urgent_ms: 300_000,
    };

    pub fn accept(&self, urgent: bool) -> u64 {
        if urgent { self.accept_urgent_ms } else { self.accept_ms }
    }
    pub fn handover(&self, urgent: bool) -> u64 {
        TICK_MS * if urgent { self.handover_urgent_ticks } else { self.handover_ticks }
    }
    pub fn answer(&self, urgent: bool) -> u64 {
        if urgent { self.answer_urgent_ms } else { self.answer_ms }
    }
}

impl Default for Deadlines {
    fn default() -> Self {
        Deadlines::R44E
    }
}
