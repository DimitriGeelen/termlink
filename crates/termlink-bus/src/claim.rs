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
///
/// `forced_from` and `forced_reason` are populated only by
/// `Bus::force_release_claim` (T-2044, Slice 11) — operator-Tier-0 path that
/// bypasses the `claimed_by == claimer` ownership check. The regular
/// `release_claim` path leaves both `None`.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub ack: bool,
    /// Original claimer of the row, populated only when this release was a
    /// force-release (T-2044). `None` for ordinary owner-initiated release.
    pub forced_from: Option<String>,
    /// Operator-supplied audit reason, populated only when this release was a
    /// force-release (T-2044). `None` when no reason was provided OR when the
    /// release was owner-initiated.
    pub forced_reason: Option<String>,
}

/// Result of transferring claim ownership — T-2046 (T-2021 GO build,
/// arc-parallel-substrate primitive #3). Distinct from `ReleaseInfo`:
/// transfer is an ownership transition, not a terminal release; the
/// `claim_id`, `topic`, `offset`, `claimed_at`, and `claimed_until` are
/// preserved, and `claimed_by` advances from `from_owner` to `to_owner` in a
/// single atomic UPDATE.
///
/// Cooperative + owner-checked: `from_owner` must equal the row's
/// `claimed_by` at transfer time (`ClaimNotOwned` otherwise). Use
/// `force_release_claim` + `claim_offset` only when bypassing ownership is
/// intentional (operator-Tier-0); `transfer_claim` is the
/// orchestrator-to-worker handoff path that preserves the lease.
#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub claim_id: String,
    pub topic: String,
    pub offset: u64,
    pub from_owner: String,
    pub to_owner: String,
    pub claimed_at: i64,
    pub claimed_until: i64,
    /// Optional audit reason supplied by the caller — returned verbatim so
    /// higher layers can surface it without persisting it in the claims
    /// table (which is current-state only, mirroring T-2044's
    /// `ReleaseInfo.forced_reason` convention).
    pub reason: Option<String>,
}

/// Aggregate view of claim state on a topic — T-2039 (arc-parallel-substrate
/// Slice 6). Computed via a single SQL aggregate over the `claims` table
/// using `idx_claims_topic_until`. Pairs with [`ClaimInfo`] (per-row detail
/// from `Bus::list_claims`) as the observability companion: `claims` answers
/// "what's claimed right now?", `claims_summary` answers "how busy is this
/// topic, and is anything stuck?".
///
/// `active_count` + `expired_count` may sum to less than the total rows ever
/// claimed on the topic — rows are lazily reaped on the next claim attempt
/// for the same `(topic, offset)`, so an expired claim only persists in
/// `claims` until someone tries that offset again.
/// T-2045 (T-2020 GO): one entry in the `agent.find_idle` result — a LIVE
/// agent on the local hub's `agent-presence` topic that is NOT currently
/// holding any claim. Derivation, not persistent state — recomputed per
/// call from heartbeat + claims. `capabilities` is parsed from the
/// comma-separated `metadata.capabilities` heartbeat field (empty when
/// the field is absent on older workers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdleAgent {
    pub agent_id: String,
    pub last_heartbeat_ms: i64,
    pub role: Option<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimsSummary {
    /// Number of rows where `claimed_until > now_ms`.
    pub active_count: u64,
    /// Number of rows where `claimed_until <= now_ms` (lazy-reaped).
    pub expired_count: u64,
    /// Wall-clock unix-ms of the oldest still-active claim's `claimed_at`.
    /// `None` when `active_count == 0`.
    pub oldest_active_at_ms: Option<i64>,
    /// `now_ms - oldest_active_at_ms` — how long the oldest live claim has
    /// been held. `None` when `active_count == 0`. Operator signal: if this
    /// approaches the TTL, the worker is either stuck or about to need to
    /// `channel.renew`.
    pub oldest_active_age_ms: Option<i64>,
    /// Wall-clock unix-ms of the next-soonest `claimed_until` among active
    /// claims — i.e. when the next slot will free up without `release`.
    /// `None` when `active_count == 0`.
    pub next_active_expiry_ms: Option<i64>,
}
