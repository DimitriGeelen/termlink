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

/// Iterator that reads records at specific byte positions. Used by
/// `Bus::subscribe` after the SQLite index tells us which records survive.
pub(crate) struct ReaderIter {
    file: File,
    records: std::vec::IntoIter<crate::meta::RecordLoc>,
}

impl ReaderIter {
    pub(crate) fn new(file: File, records: Vec<crate::meta::RecordLoc>) -> Self {
        Self {
            file,
            records: records.into_iter(),
        }
    }
}

impl Iterator for ReaderIter {
    type Item = Result<(Offset, Envelope)>;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop so a single poison record (truncated payload / undecodable bytes)
        // becomes a logged GAP, not a WALL: the append path fsyncs the SQLite index
        // but not the payload (T-2464), so a crash mid-append can leave the index
        // pointing at bytes that never fully hit disk. Before this loop, that one bad
        // record made `next()` return `Some(Err(..))`, and every consumer's `item?`
        // aborted the whole topic's replay — permanently, on every re-subscribe. We
        // now skip the poison offset and continue. Every skip is LOUD (eprintln to
        // stderr, no new dep — directive #2: no silent failures). Genuinely systemic
        // faults (seek failure, index length overflow, non-EOF read errors) still
        // propagate as `Err` — only per-record data corruption is skippable.
        loop {
            let loc = self.records.next()?;
            // Skip the 8-byte length prefix — we have the length in `loc`.
            let payload_start = loc.byte_pos + 8;
            if let Err(e) = self.file.seek(SeekFrom::Start(payload_start)) {
                return Some(Err(BusError::Io(e)));
            }
            let Ok(len) = usize::try_from(loc.length) else {
                return Some(Err(BusError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "record length overflows usize",
                ))));
            };
            let mut buf = vec![0u8; len];
            if let Err(e) = self.file.read_exact(&mut buf) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    // Truncated record — the payload on disk is shorter than the
                    // index says. Skip it and replay the rest of the topic.
                    eprintln!(
                        "termlink-bus: WARN skipping truncated record at offset {} \
                         (byte_pos {}, declared {} bytes): {e}",
                        loc.offset, loc.byte_pos, len
                    );
                    continue;
                }
                // Non-EOF read error is systemic (e.g. disk I/O) — surface it.
                return Some(Err(BusError::Io(e)));
            }
            match decode_envelope(&buf) {
                Ok(env) => return Some(Ok((loc.offset, env))),
                Err(e) => {
                    // Undecodable payload — corrupt bytes at a valid position. Skip.
                    eprintln!(
                        "termlink-bus: WARN skipping undecodable record at offset {} \
                         (byte_pos {}, {} bytes): {e}",
                        loc.offset, loc.byte_pos, len
                    );
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // T-2487 — reader-resilience: a single poison record (undecodable payload or a
    // truncated tail from a crash mid-append) must become a logged GAP, not a WALL
    // that bricks the whole topic's replay for every consumer forever.
    use super::*;
    use crate::meta::RecordLoc;
    use std::io::Write as _;

    fn valid_payload(marker: &str) -> Vec<u8> {
        encode_envelope(&Envelope {
            topic: "t".into(),
            sender_id: "s".into(),
            msg_type: "note".into(),
            payload: marker.as_bytes().to_vec(),
            artifact_ref: None,
            ts_unix_ms: 0,
            metadata: std::collections::BTreeMap::new(),
        })
        .unwrap()
    }

    // Write payloads as [8-byte BE length][payload] each, returning the reader-ready
    // RecordLoc index (length excludes the prefix, matching the SQLite offsets table).
    fn write_raw_log(payloads: &[&[u8]]) -> (tempfile::TempDir, std::path::PathBuf, Vec<RecordLoc>) {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("test.log");
        let mut f = std::fs::File::create(&path).expect("create");
        let mut records = Vec::new();
        let mut pos: u64 = 0;
        for (i, p) in payloads.iter().enumerate() {
            let len = p.len() as u64;
            f.write_all(&len.to_be_bytes()).unwrap();
            f.write_all(p).unwrap();
            records.push(RecordLoc { offset: i as u64, byte_pos: pos, length: len });
            pos += 8 + len;
        }
        f.flush().unwrap();
        (dir, path, records)
    }

    #[test]
    fn reader_skips_undecodable_middle_record_and_yields_before_and_after() {
        let a = valid_payload("A");
        let c = valid_payload("C");
        // Non-JSON garbage of a plausible length — decode_envelope() will fail.
        let garbage: &[u8] = b"}{ this is not a valid envelope at all";
        let (_dir, path, records) = write_raw_log(&[&a, garbage, &c]);
        let file = std::fs::File::open(&path).unwrap();
        let got: Vec<(Offset, Envelope)> = ReaderIter::new(file, records)
            .map(|r| r.expect("a poison record must be SKIPPED, never surfaced as Err"))
            .collect();
        let offsets: Vec<Offset> = got.iter().map(|(o, _)| *o).collect();
        assert_eq!(offsets, vec![0, 2], "middle poison record must be skipped, not wall the topic");
        assert_eq!(got[0].1.payload, b"A");
        assert_eq!(got[1].1.payload, b"C");
    }

    #[test]
    fn reader_skips_truncated_tail_record() {
        // Realistic crash-mid-append: the last record's index entry declares more
        // bytes than the file actually contains → read_exact hits UnexpectedEof.
        let a = valid_payload("A");
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("trunc.log");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(&(a.len() as u64).to_be_bytes()).unwrap();
        f.write_all(&a).unwrap();
        let rec1_pos = 8 + a.len() as u64;
        f.write_all(&100u64.to_be_bytes()).unwrap(); // declares 100 bytes…
        f.write_all(&[0u8; 5]).unwrap(); // …but only 5 exist, then the file ends.
        f.flush().unwrap();
        let records = vec![
            RecordLoc { offset: 0, byte_pos: 0, length: a.len() as u64 },
            RecordLoc { offset: 1, byte_pos: rec1_pos, length: 100 },
        ];
        let file = std::fs::File::open(&path).unwrap();
        let got: Vec<(Offset, Envelope)> = ReaderIter::new(file, records)
            .map(|r| r.expect("a truncated tail must be SKIPPED, never surfaced as Err"))
            .collect();
        let offsets: Vec<Offset> = got.iter().map(|(o, _)| *o).collect();
        assert_eq!(offsets, vec![0], "valid record before a truncated tail must still replay");
        assert_eq!(got[0].1.payload, b"A");
    }

    #[test]
    fn reader_all_valid_records_unaffected() {
        // Regression guard: the loop must not change happy-path behaviour.
        let a = valid_payload("A");
        let b = valid_payload("B");
        let c = valid_payload("C");
        let (_dir, path, records) = write_raw_log(&[&a, &b, &c]);
        let file = std::fs::File::open(&path).unwrap();
        let offsets: Vec<Offset> = ReaderIter::new(file, records)
            .map(|r| r.unwrap().0)
            .collect();
        assert_eq!(offsets, vec![0, 1, 2]);
    }
}
