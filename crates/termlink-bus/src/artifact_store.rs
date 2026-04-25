//! Content-addressed blob store (T-1248 / T-1164a).
//!
//! Layout: `<root>/<first-2-hex>/<full-sha256>`. Sharding by the first two
//! hex chars keeps any single directory below ~256 entries even at scale,
//! avoiding the directory-walk pathology when an inbox accumulates 100k+
//! artifacts.
//!
//! Idempotent put: writing the same bytes twice is a no-op on the second
//! call. Get returns the bytes if the hash exists; `exists` is the cheap
//! probe used by senders to skip uploads of already-known blobs (dedup).
//!
//! Streaming put/get over JSON-RPC happens at a higher layer
//! (`termlink-hub::artifact`) — this module only owns the on-disk shape.

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::error::{BusError, Result};

/// Content-addressed blob store rooted at a directory.
#[derive(Clone, Debug)]
pub struct ArtifactStore {
    root: PathBuf,
}

impl ArtifactStore {
    /// Open (or create) an artifact store at `root`. Creates the directory
    /// if it doesn't exist.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Root directory of the store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Compute the on-disk path for a given hex sha256, regardless of whether
    /// the blob actually exists. Used by callers that need the path before a
    /// streaming write.
    pub fn path_for(&self, sha256_hex: &str) -> PathBuf {
        let prefix = if sha256_hex.len() >= 2 {
            &sha256_hex[..2]
        } else {
            sha256_hex
        };
        self.root.join(prefix).join(sha256_hex)
    }

    /// True if a blob with this sha256 is already stored.
    pub fn exists(&self, sha256_hex: &str) -> bool {
        self.path_for(sha256_hex).is_file()
    }

    /// Hash `bytes`, write them to the content-addressed location atomically,
    /// and return the hex sha256. If the blob already exists at that hash, no
    /// rewrite happens (idempotent).
    pub fn put(&self, bytes: &[u8]) -> Result<String> {
        let sha = hex_sha256(bytes);
        let dst = self.path_for(&sha);
        if dst.is_file() {
            return Ok(sha);
        }
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        write_atomic(&dst, bytes)?;
        Ok(sha)
    }

    /// Read the full blob for `sha256_hex`. Returns `BusError::UnknownArtifact`
    /// if not present.
    pub fn get(&self, sha256_hex: &str) -> Result<Vec<u8>> {
        let path = self.path_for(sha256_hex);
        match fs::read(&path) {
            Ok(b) => Ok(b),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(BusError::UnknownArtifact(sha256_hex.to_string()))
            }
            Err(e) => Err(BusError::Io(e)),
        }
    }

    /// Streaming put. The caller drives chunked appends with monotonically
    /// non-decreasing offsets and an `is_final` marker on the last chunk. The
    /// store buffers under a per-sha staging file so a partial transfer cannot
    /// expose a half-written blob via `exists` / `get`.
    ///
    /// On `is_final == true` the store hashes the buffer, verifies it matches
    /// `expected_sha256` (if provided), and atomically renames into the
    /// content-addressed slot. Returns the final hex sha256.
    pub fn put_streaming(
        &self,
        staging_id: &str,
        offset: u64,
        chunk: &[u8],
        is_final: bool,
        expected_sha256: Option<&str>,
    ) -> Result<StreamingPutOutcome> {
        let staging_path = self.staging_path(staging_id);
        if let Some(parent) = staging_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let current_len = match fs::metadata(&staging_path) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => 0,
            Err(e) => return Err(BusError::Io(e)),
        };
        if offset != current_len {
            return Err(BusError::ArtifactOffsetMismatch {
                expected: current_len,
                got: offset,
            });
        }
        // Append chunk
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&staging_path)?;
        f.write_all(chunk)?;
        f.flush()?;
        drop(f);

        if !is_final {
            return Ok(StreamingPutOutcome::InProgress {
                bytes_received: current_len + chunk.len() as u64,
            });
        }

        // Final chunk — hash, verify, promote.
        let bytes = fs::read(&staging_path)?;
        let sha = hex_sha256(&bytes);
        if let Some(expected) = expected_sha256
            && expected != sha
        {
            let _ = fs::remove_file(&staging_path);
            return Err(BusError::ArtifactHashMismatch {
                expected: expected.to_string(),
                got: sha,
            });
        }
        let dst = self.path_for(&sha);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        if dst.is_file() {
            // Already present — drop the staging copy.
            let _ = fs::remove_file(&staging_path);
        } else {
            fs::rename(&staging_path, &dst).or_else(|_| {
                // cross-fs fallback
                fs::copy(&staging_path, &dst)?;
                fs::remove_file(&staging_path)?;
                Ok::<_, io::Error>(())
            })?;
        }
        Ok(StreamingPutOutcome::Complete {
            sha256: sha,
            total_bytes: bytes.len() as u64,
        })
    }

    fn staging_path(&self, staging_id: &str) -> PathBuf {
        self.root.join(".staging").join(staging_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamingPutOutcome {
    InProgress { bytes_received: u64 },
    Complete { sha256: String, total_bytes: u64 },
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

fn write_atomic(dst: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = match dst.parent() {
        Some(parent) => {
            let mut t = parent.to_path_buf();
            t.push(format!(
                ".tmp.{}.{}",
                std::process::id(),
                fastrand_hex_suffix()
            ));
            t
        }
        None => return Err(io::Error::other("dst has no parent")),
    };
    {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        f.write_all(bytes)?;
        f.flush()?;
    }
    fs::rename(&tmp, dst).or_else(|_| {
        fs::copy(&tmp, dst)?;
        fs::remove_file(&tmp)?;
        Ok::<_, io::Error>(())
    })?;
    Ok(())
}

fn fastrand_hex_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn put_and_get_roundtrip() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let payload = b"hello artifact world";
        let sha = store.put(payload).unwrap();
        assert_eq!(sha.len(), 64);
        assert!(store.exists(&sha));
        let back = store.get(&sha).unwrap();
        assert_eq!(back, payload);
    }

    #[test]
    fn put_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let payload = b"same bytes twice";
        let sha1 = store.put(payload).unwrap();
        let sha2 = store.put(payload).unwrap();
        assert_eq!(sha1, sha2);
    }

    #[test]
    fn sharded_layout() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let sha = store.put(b"layout test").unwrap();
        let path = store.path_for(&sha);
        // <root>/<2-hex>/<full-sha>
        assert_eq!(path.parent().unwrap().file_name().unwrap(), &sha[..2]);
        assert!(path.is_file());
    }

    #[test]
    fn get_unknown_returns_error() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let bogus = "0".repeat(64);
        let err = store.get(&bogus).unwrap_err();
        match err {
            BusError::UnknownArtifact(s) => assert_eq!(s, bogus),
            other => panic!("expected UnknownArtifact, got {other:?}"),
        }
    }

    #[test]
    fn streaming_put_complete_match() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let payload = b"streaming hello world";
        let expected = hex_sha256(payload);
        let outcome = store
            .put_streaming("stream-1", 0, payload, true, Some(&expected))
            .unwrap();
        match outcome {
            StreamingPutOutcome::Complete { sha256, total_bytes } => {
                assert_eq!(sha256, expected);
                assert_eq!(total_bytes, payload.len() as u64);
            }
            other => panic!("expected Complete, got {other:?}"),
        }
        assert!(store.exists(&expected));
        assert_eq!(store.get(&expected).unwrap(), payload);
    }

    #[test]
    fn streaming_put_chunked() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let mut payload = Vec::new();
        for i in 0..10_000u32 {
            payload.extend_from_slice(&i.to_be_bytes());
        }
        let expected = hex_sha256(&payload);
        let chunk_size = 4096;
        let mut offset = 0u64;
        let mut iter = payload.chunks(chunk_size).peekable();
        while let Some(chunk) = iter.next() {
            let is_final = iter.peek().is_none();
            let _ = store
                .put_streaming(
                    "stream-2",
                    offset,
                    chunk,
                    is_final,
                    if is_final { Some(&expected) } else { None },
                )
                .unwrap();
            offset += chunk.len() as u64;
        }
        assert!(store.exists(&expected));
        assert_eq!(store.get(&expected).unwrap(), payload);
    }

    #[test]
    fn streaming_put_offset_mismatch() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let _ = store
            .put_streaming("stream-3", 0, b"first", false, None)
            .unwrap();
        // Wrong offset on second chunk
        let err = store
            .put_streaming("stream-3", 99, b"second", true, None)
            .unwrap_err();
        match err {
            BusError::ArtifactOffsetMismatch { expected, got } => {
                assert_eq!(expected, 5);
                assert_eq!(got, 99);
            }
            other => panic!("expected ArtifactOffsetMismatch, got {other:?}"),
        }
    }

    #[test]
    fn streaming_put_hash_mismatch_rejects() {
        let dir = TempDir::new().unwrap();
        let store = ArtifactStore::open(dir.path()).unwrap();
        let payload = b"genuine payload";
        let bogus = "0".repeat(64);
        let err = store
            .put_streaming("stream-4", 0, payload, true, Some(&bogus))
            .unwrap_err();
        match err {
            BusError::ArtifactHashMismatch { expected, got } => {
                assert_eq!(expected, bogus);
                assert_eq!(got, hex_sha256(payload));
            }
            other => panic!("expected ArtifactHashMismatch, got {other:?}"),
        }
    }
}
