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
}
