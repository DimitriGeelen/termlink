use thiserror::Error;

pub type Result<T> = std::result::Result<T, BusError>;

#[derive(Debug, Error)]
pub enum BusError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("topic {name:?} already exists with a different retention policy (existing={existing:?}, requested={requested:?})")]
    TopicPolicyMismatch {
        name: String,
        existing: crate::Retention,
        requested: crate::Retention,
    },

    // T-2500: an existing topic row's stored (kind, value) does not parse into
    // any known Retention. Surfaced LOUD instead of fabricating Retention::Forever
    // (the old `unwrap_or(Forever)` silently masked corruption / schema drift /
    // cross-version rows and let a bounded topic masquerade as unbounded).
    #[error("topic {name:?} has a corrupt/unrecognized stored retention policy (kind={kind:?}, value={value})")]
    CorruptRetention {
        name: String,
        kind: String,
        value: i64,
    },

    #[error("topic {0:?} not found")]
    UnknownTopic(String),

    #[error("artifact {0:?} not found")]
    UnknownArtifact(String),

    #[error("artifact streaming offset mismatch: expected {expected}, got {got}")]
    ArtifactOffsetMismatch { expected: u64, got: u64 },

    #[error("artifact hash mismatch: declared {expected}, computed {got}")]
    ArtifactHashMismatch { expected: String, got: String },

    // T-2525: artifact streaming exceeded the per-artifact size ceiling. Loud
    // typed refusal (not a silent truncation) so a peer streaming an unbounded
    // blob cannot exhaust hub disk / OOM the hub at finalize. Cap is tunable via
    // TERMLINK_MAX_ARTIFACT_BYTES (default 2 GiB).
    #[error("artifact too large: staged {got} bytes exceeds limit {limit} (raise TERMLINK_MAX_ARTIFACT_BYTES for legitimate large transfers)")]
    ArtifactTooLarge { limit: u64, got: u64 },

    // T-2029 (arc-parallel-substrate Slice 1): claim semantics errors.
    #[error("offset {offset} of topic {topic:?} is already claimed by another worker")]
    ClaimConflict { topic: String, offset: u64 },

    #[error("claim {0:?} not found (expired, never created, or already released)")]
    ClaimNotFound(String),

    #[error("claim {claim_id:?} is held by {claimed_by:?}, not {attempted_by:?}")]
    ClaimNotOwned {
        claim_id: String,
        claimed_by: String,
        attempted_by: String,
    },

    // T-2030 (arc-parallel-substrate Slice 2): renew-after-expiry.
    #[error("claim {claim_id:?} has expired (claimed_until <= now); cannot renew")]
    ClaimExpired { claim_id: String },
}
