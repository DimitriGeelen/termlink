//! TermLink agent channel bus (T-1155).
//!
//! Append-only per-topic log, per-recipient cursor store, and per-topic
//! retention. Hub embeds this crate as a passive library — net/RPC lives
//! in `termlink-hub` (T-1160 exposes the channel API).
//!
//! Wedge 1 (T-1158 scaffold): crate skeleton, `Envelope`, `Retention`,
//! `Bus::open`, `create_topic` + `list_topics` via SQLite metadata.
//! Log-append + subscribe + retention sweep land in follow-up wedges.

mod envelope;
mod error;
mod meta;
mod retention;

pub use envelope::Envelope;
pub use error::{BusError, Result};
pub use retention::Retention;

use std::path::{Path, PathBuf};

/// Channel bus instance. Backed by a per-topic append-only log under
/// `<root>/topics/` and a SQLite metadata sidecar at `<root>/meta.db`.
pub struct Bus {
    root: PathBuf,
    meta: meta::Meta,
}

impl Bus {
    /// Open (or create) a bus rooted at `path`. Creates the directory,
    /// the `topics/` subdir, and initializes the SQLite schema if missing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("topics"))?;
        let meta = meta::Meta::open(&root.join("meta.db"))?;
        Ok(Self { root, meta })
    }

    /// Root directory the bus lives under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Register a new topic with a retention policy. Idempotent — a second
    /// call with the same name and policy is a no-op; a second call with a
    /// different policy returns `BusError::TopicPolicyMismatch`.
    pub fn create_topic(&self, name: &str, retention: Retention) -> Result<()> {
        self.meta.create_topic(name, retention)
    }

    /// List all registered topic names, sorted.
    pub fn list_topics(&self) -> Result<Vec<String>> {
        self.meta.list_topics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp_bus() -> (TempDir, Bus) {
        let dir = TempDir::new().expect("tempdir");
        let bus = Bus::open(dir.path()).expect("open bus");
        (dir, bus)
    }

    #[test]
    fn open_creates_layout() {
        let (dir, bus) = tmp_bus();
        assert!(dir.path().join("topics").is_dir());
        assert!(dir.path().join("meta.db").is_file());
        assert_eq!(bus.root(), dir.path());
    }

    #[test]
    fn create_topic_roundtrip() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("broadcast:global", Retention::Forever).unwrap();
        bus.create_topic("channel:learnings", Retention::Days(30)).unwrap();
        let topics = bus.list_topics().unwrap();
        assert_eq!(topics, vec!["broadcast:global", "channel:learnings"]);
    }

    #[test]
    fn create_topic_idempotent_on_same_policy() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Messages(100)).unwrap();
        bus.create_topic("t", Retention::Messages(100)).unwrap();
        assert_eq!(bus.list_topics().unwrap().len(), 1);
    }

    #[test]
    fn create_topic_rejects_policy_mismatch() {
        let (_dir, bus) = tmp_bus();
        bus.create_topic("t", Retention::Forever).unwrap();
        let err = bus.create_topic("t", Retention::Days(7)).unwrap_err();
        assert!(matches!(err, BusError::TopicPolicyMismatch { .. }));
    }
}
