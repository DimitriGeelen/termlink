use serde::{Deserialize, Serialize};

/// Per-topic retention policy. Enforced explicitly by `Bus::sweep` (to land
/// in a follow-up wedge); the library does not run a background thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Retention {
    /// Never trim. Topic grows forever. Suitable for low-volume audit logs.
    Forever,
    /// Trim messages whose `ts_unix_ms` is older than N days.
    Days(u32),
    /// Keep at most N most-recent messages; drop the tail beyond that.
    Messages(u64),
    /// Keep only the single most-recent envelope. Durable-storage counterpart
    /// to the in-memory cv_index (T-2103): for broadcast-with-replay topics
    /// where only the freshest envelope per topic matters (presence summaries,
    /// single-value state). Semantically equivalent to `Messages(1)` with an
    /// explicit name — disambiguates intent at topic creation time.
    Latest,
}

impl Retention {
    /// Stable kind discriminant for SQLite storage. Keep these string
    /// values stable across schema versions.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Retention::Forever => "forever",
            Retention::Days(_) => "days",
            Retention::Messages(_) => "messages",
            Retention::Latest => "latest",
        }
    }

    /// Numeric payload for SQLite storage. `Forever` and `Latest` store 0.
    pub(crate) fn value(&self) -> i64 {
        match self {
            Retention::Forever => 0,
            Retention::Days(d) => i64::from(*d),
            Retention::Messages(m) => *m as i64,
            Retention::Latest => 0,
        }
    }

    pub(crate) fn from_parts(kind: &str, value: i64) -> Option<Self> {
        match kind {
            "forever" => Some(Retention::Forever),
            "days" => u32::try_from(value).ok().map(Retention::Days),
            "messages" => u64::try_from(value).ok().map(Retention::Messages),
            "latest" => Some(Retention::Latest),
            _ => None,
        }
    }
}
