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
}

impl Retention {
    /// Stable kind discriminant for SQLite storage. Keep these string
    /// values stable across schema versions.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Retention::Forever => "forever",
            Retention::Days(_) => "days",
            Retention::Messages(_) => "messages",
        }
    }

    /// Numeric payload for SQLite storage. `Forever` stores 0.
    pub(crate) fn value(&self) -> i64 {
        match self {
            Retention::Forever => 0,
            Retention::Days(d) => i64::from(*d),
            Retention::Messages(m) => *m as i64,
        }
    }

    pub(crate) fn from_parts(kind: &str, value: i64) -> Option<Self> {
        match kind {
            "forever" => Some(Retention::Forever),
            "days" => u32::try_from(value).ok().map(Retention::Days),
            "messages" => u64::try_from(value).ok().map(Retention::Messages),
            _ => None,
        }
    }
}
