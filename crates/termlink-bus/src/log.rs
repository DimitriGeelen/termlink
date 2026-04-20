use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::{BusError, Envelope, Result};

/// Monotonic record offset within a topic (0-based).
pub type Offset = u64;

/// Resolve the on-disk file for a topic. Topic names can contain any UTF-8
/// (including `:` and `/`), so we hash to avoid accidentally nesting dirs.
pub(crate) fn topic_log_path(root: &Path, topic: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(topic.as_bytes());
    let digest = hasher.finalize();
    let hex = digest.iter().fold(String::with_capacity(64), |mut acc, b| {
        use std::fmt::Write;
        let _ = write!(&mut acc, "{:02x}", b);
        acc
    });
    root.join("topics").join(format!("{hex}.log"))
}

/// Append-only writer for one topic. Held behind a tokio mutex so async
/// post() across tasks serializes on the write path only (reads take no
/// lock — they open the file read-only and stream positionally).
pub(crate) struct LogAppender {
    inner: Mutex<File>,
}

impl LogAppender {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)?;
        Ok(Self {
            inner: Mutex::new(file),
        })
    }

    /// Append one record. Returns the byte offset the record starts at
    /// (i.e. position of the length prefix). Caller is responsible for
    /// mapping byte-offset → logical offset via the SQLite offsets table.
    pub(crate) async fn append(&self, payload: &[u8]) -> Result<u64> {
        let mut guard = self.inner.lock().await;
        let start = guard.seek(SeekFrom::End(0))?;
        let len = u64::try_from(payload.len()).map_err(|e| {
            BusError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("payload too large: {e}"),
            ))
        })?;
        guard.write_all(&len.to_be_bytes())?;
        guard.write_all(payload)?;
        guard.flush()?;
        Ok(start)
    }
}

/// Streaming reader over a topic's log file. Yields every framed record
/// from the start; callers advance past a cursor by draining the prefix.
pub(crate) struct LogReader {
    file: File,
}

impl LogReader {
    pub(crate) fn open(path: &Path) -> Result<Option<Self>> {
        match File::open(path) {
            Ok(file) => Ok(Some(Self { file })),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(BusError::Io(e)),
        }
    }
}

impl Iterator for LogReader {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut len_buf = [0u8; 8];
        match self.file.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(BusError::Io(e))),
        }
        let len = u64::from_be_bytes(len_buf);
        let Ok(len) = usize::try_from(len) else {
            return Some(Err(BusError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "record length overflows usize",
            ))));
        };
        let mut buf = vec![0u8; len];
        if let Err(e) = self.file.read_exact(&mut buf) {
            return Some(Err(BusError::Io(e)));
        }
        Some(Ok(buf))
    }
}

/// Serialize an envelope to the on-disk byte form. JSON for wedge 2 —
/// T-1155 §"Open questions deferred" leaves the codec choice open.
pub(crate) fn encode_envelope(env: &Envelope) -> Result<Vec<u8>> {
    serde_json::to_vec(env).map_err(|e| {
        BusError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })
}

pub(crate) fn decode_envelope(bytes: &[u8]) -> Result<Envelope> {
    serde_json::from_slice(bytes).map_err(|e| {
        BusError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    })
}
