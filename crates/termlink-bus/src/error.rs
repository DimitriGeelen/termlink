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
}
