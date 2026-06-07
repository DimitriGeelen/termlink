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

    #[error("topic {0:?} not found")]
    UnknownTopic(String),

    #[error("artifact {0:?} not found")]
    UnknownArtifact(String),

    #[error("artifact streaming offset mismatch: expected {expected}, got {got}")]
    ArtifactOffsetMismatch { expected: u64, got: u64 },

    #[error("artifact hash mismatch: declared {expected}, computed {got}")]
    ArtifactHashMismatch { expected: String, got: String },

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
