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
         INSERT OR IGNORE INTO schema_version (version) VALUES (1);",
    )?;
    Ok(())
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
