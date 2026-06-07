//! Exclusive-delivery claim types — T-2029 (arc-parallel-substrate Slice 1).
//!
//! A claim is a short-lived lease over a specific `(topic, offset)`. While
//! a claim is active, no other worker can claim that offset; on release with
//! `ack=true` the worker's cursor advances past the offset (so they don't
//! re-receive it on subscribe); on release with `ack=false` the cursor is
//! unchanged, and another worker can pick up the work. If the worker dies
//! without releasing, the claim expires at `claimed_until` and is lazily
//! reaped on the next claim attempt for that `(topic, offset)`.
//!
//! Per T-2019 inception (GO 2026-06-07): chose §4.2 lease-with-renewal +
//! lazy expiry over §4.1 (dying-worker hole) and §4.3 (cursor-independence
//! + non-idempotent work hole). T-1155 invariant: no background threads —
//! reapers run inline on access.

/// Successful claim of `(topic, offset)` by `claimer`.
#[derive(Debug, Clone)]
pub struct ClaimInfo {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub claimer: String,
    pub claimed_at: i64,
    pub claimed_until: i64,
}

/// Result of releasing a claim. `ack=true` means the claimer's cursor was
/// advanced past `offset`; `ack=false` means the slot is free for another
/// worker to claim.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub ack: bool,
}
