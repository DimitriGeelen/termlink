use std::path::Path;

use rusqlite::{Connection, params};

use crate::{BusError, Result, Retention};

/// SQLite sidecar tracking topics, cursors, and per-topic offset counters.
/// Schema version is baked into a `schema_version` table — bump when the
/// shape changes.
pub(crate) struct Meta {
    conn: std::sync::Mutex<Connection>,
}

impl Meta {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        init_schema(&conn)?;
        Ok(Self {
            conn: std::sync::Mutex::new(conn),
        })
    }

    pub(crate) fn create_topic(&self, name: &str, retention: Retention) -> Result<()> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let existing: Option<(String, i64)> = conn
            .query_row(
                "SELECT retention_kind, retention_value FROM topics WHERE name = ?1",
                params![name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        if let Some((kind, value)) = existing {
            let got = Retention::from_parts(&kind, value).unwrap_or(Retention::Forever);
            if got != retention {
                return Err(BusError::TopicPolicyMismatch {
                    name: name.to_string(),
                    existing: got,
                    requested: retention,
                });
            }
            return Ok(());
        }
        let now_ms = now_unix_ms();
        conn.execute(
            "INSERT INTO topics (name, retention_kind, retention_value, created_at) \
             VALUES (?1, ?2, ?3, ?4)",
            params![name, retention.kind(), retention.value(), now_ms],
        )?;
        conn.execute(
            "INSERT INTO offsets (topic, next_offset) VALUES (?1, 0)",
            params![name],
        )?;
        Ok(())
    }

    pub(crate) fn list_topics(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare("SELECT name FROM topics ORDER BY name")?;
        let names = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(names)
    }

    pub(crate) fn topic_retention(&self, name: &str) -> Result<Option<Retention>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let row: rusqlite::Result<(String, i64)> = conn.query_row(
            "SELECT retention_kind, retention_value FROM topics WHERE name = ?1",
            params![name],
            |r| Ok((r.get(0)?, r.get(1)?)),
        );
        match row {
            Ok((k, v)) => Ok(Retention::from_parts(&k, v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    pub(crate) fn topic_exists(&self, name: &str) -> Result<bool> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM topics WHERE name = ?1",
            params![name],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    /// Atomically reserve the next offset for `topic` and insert a
    /// `records` row pointing at the given byte position. Returns the
    /// offset assigned.
    pub(crate) fn record_append(
        &self,
        topic: &str,
        byte_pos: u64,
        length: u64,
        ts_unix_ms: i64,
    ) -> Result<u64> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let current: i64 = tx.query_row(
            "SELECT next_offset FROM offsets WHERE topic = ?1",
            params![topic],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE offsets SET next_offset = ?1 WHERE topic = ?2",
            params![current + 1, topic],
        )?;
        tx.execute(
            "INSERT INTO records (topic, offset, byte_pos, length, ts_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![topic, current, byte_pos as i64, length as i64, ts_unix_ms],
        )?;
        tx.commit()?;
        Ok(current as u64)
    }

    /// Fetch record locators for `topic` with `offset >= cursor`, ordered.
    pub(crate) fn records_from(&self, topic: &str, cursor: u64) -> Result<Vec<RecordLoc>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT offset, byte_pos, length FROM records \
             WHERE topic = ?1 AND offset >= ?2 ORDER BY offset",
        )?;
        let rows = stmt.query_map(params![topic, cursor as i64], |r| {
            Ok(RecordLoc {
                offset: r.get::<_, i64>(0)? as u64,
                byte_pos: r.get::<_, i64>(1)? as u64,
                length: r.get::<_, i64>(2)? as u64,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(BusError::Sqlite)
    }

    /// Persist a subscriber cursor for the given (subscriber, topic).
    pub(crate) fn put_cursor(
        &self,
        subscriber_id: &str,
        topic: &str,
        offset: u64,
    ) -> Result<()> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        conn.execute(
            "INSERT INTO cursors (subscriber_id, topic, last_offset) VALUES (?1, ?2, ?3) \
             ON CONFLICT(subscriber_id, topic) DO UPDATE SET last_offset = excluded.last_offset",
            params![subscriber_id, topic, offset as i64],
        )?;
        Ok(())
    }

    /// Read a subscriber cursor. Returns `None` if never persisted.
    pub(crate) fn get_cursor(&self, subscriber_id: &str, topic: &str) -> Result<Option<u64>> {
        let conn = self.conn.lock().expect("meta mutex poisoned");
        let v: rusqlite::Result<i64> = conn.query_row(
            "SELECT last_offset FROM cursors WHERE subscriber_id = ?1 AND topic = ?2",
            params![subscriber_id, topic],
            |r| r.get(0),
        );
        match v {
            Ok(n) => Ok(Some(n as u64)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(BusError::Sqlite(e)),
        }
    }

    /// Delete records matching a retention policy. Returns the set of
    /// record locators that were removed (so the caller can rewrite the
    /// on-disk log, if desired). Current callers leave dead bytes in the
    /// file — compaction is a follow-up.
    pub(crate) fn sweep_records(
        &self,
        topic: &str,
        keep_after_ts_ms: Option<i64>,
        keep_last_n: Option<u64>,
    ) -> Result<u64> {
        let mut conn = self.conn.lock().expect("meta mutex poisoned");
        let tx = conn.transaction()?;
        let mut deleted: u64 = 0;
        if let Some(ts) = keep_after_ts_ms {
            let n = tx.execute(
                "DELETE FROM records WHERE topic = ?1 AND ts_unix_ms < ?2",
                params![topic, ts],
            )?;
            deleted += n as u64;
        }
        if let Some(n) = keep_last_n {
            let total: i64 = tx.query_row(
                "SELECT COUNT(*) FROM records WHERE topic = ?1",
                params![topic],
                |r| r.get(0),
            )?;
            if (total as u64) > n {
                let drop_n = (total as u64) - n;
                let removed = tx.execute(
                    "DELETE FROM records WHERE rowid IN (\
                        SELECT rowid FROM records WHERE topic = ?1 \
                        ORDER BY offset ASC LIMIT ?2\
                     )",
                    params![topic, drop_n as i64],
                )?;
                deleted += removed as u64;
            }
        }
        tx.commit()?;
        Ok(deleted)
    }
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
         );
         CREATE TABLE IF NOT EXISTS topics (
            name           TEXT PRIMARY KEY,
            retention_kind TEXT NOT NULL,
            retention_value INTEGER NOT NULL,
            created_at     INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS cursors (
            subscriber_id TEXT NOT NULL,
            topic         TEXT NOT NULL,
            last_offset   INTEGER NOT NULL,
            PRIMARY KEY (subscriber_id, topic)
         );
         CREATE TABLE IF NOT EXISTS offsets (
            topic       TEXT PRIMARY KEY,
            next_offset INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS records (
            topic       TEXT NOT NULL,
            offset      INTEGER NOT NULL,
            byte_pos    INTEGER NOT NULL,
            length      INTEGER NOT NULL,
            ts_unix_ms  INTEGER NOT NULL,
            PRIMARY KEY (topic, offset)
         );
         CREATE INDEX IF NOT EXISTS idx_records_topic_ts
            ON records (topic, ts_unix_ms);
         INSERT OR IGNORE INTO schema_version (version) VALUES (1);",
    )?;
    Ok(())
}

/// Pre-fetched record locator for streaming subscribe reads.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecordLoc {
    pub offset: u64,
    pub byte_pos: u64,
    pub length: u64,
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
