//! C-2 durable record store (architecture note section 4).
//!
//! Layout under `<store>/sidecar/<agent>/` (dirs 0700, files 0600):
//! `lock`, `log/<conv>.jsonl`, `msgs/<conv>/<seq>-<msg_id>.rec`,
//! `claims/<msg_id>.<attempt>`, `tmp/`, plus `attention.jsonl` (the
//! needs-attention list written by recovery and quarantine).
//!
//! Deviations from the note's field lists, both additive: the record header and
//! the chain line also carry `sender`, because the identity key of 4.4 is
//! `(sender fingerprint, conv, msg_id)` and recovery has no other source for it.
//!
//! `seq` is the position of a line in the conversation chain. A record's file
//! name carries the seq its STORED line will have.

use crate::estate::{self, EstateError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/// Stated by `status` so no one reads an intact walk as "nothing was cut off".
pub const TAIL_NOTE: &str = "tail truncation: not yet detectable (anchor comparison is PH-2)";

/// The named points of the 4.3 write sequence. A failpoint stops *before* the step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    WriteTmp,
    FsyncFile,
    Rename,
    FsyncDir,
    AppendChain,
    /// Half a chain line reaches the log, then the process "dies".
    AppendChainTorn,
    FsyncLog,
    Report,
}

impl Step {
    pub const ALL: [Step; 8] = [
        Step::WriteTmp,
        Step::FsyncFile,
        Step::Rename,
        Step::FsyncDir,
        Step::AppendChain,
        Step::AppendChainTorn,
        Step::FsyncLog,
        Step::Report,
    ];
}

#[derive(Debug)]
pub enum StoreError {
    Estate(EstateError),
    /// Another sidecar holds the lock (SI-16, FC-9).
    Locked,
    Io(io::Error),
    BadName(String),
    Failpoint(Step),
    /// The record failed its checksum and was renamed `.bad`.
    Quarantined(PathBuf),
    /// A chain line says STORED but the record is not there.
    MissingRecord { conv: String, seq: u64, msg_id: String },
    Corrupt(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Estate(e) => write!(f, "estate guard: {e}"),
            Self::Locked => write!(f, "store is locked by another sidecar"),
            Self::Io(e) => write!(f, "io: {e}"),
            Self::BadName(n) => write!(f, "unsafe name {n:?}"),
            Self::Failpoint(s) => write!(f, "failpoint at {s:?}"),
            Self::Quarantined(p) => write!(f, "record quarantined: {}", p.display()),
            Self::MissingRecord { conv, seq, msg_id } => {
                write!(f, "chain says STORED but record is missing: {conv}/{seq}-{msg_id}")
            }
            Self::Corrupt(s) => write!(f, "corrupt: {s}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

type Result<T> = std::result::Result<T, StoreError>;

#[cfg(feature = "failpoints")]
thread_local! {
    static ARMED: std::cell::Cell<Option<Step>> = const { std::cell::Cell::new(None) };
}

/// Arm one failpoint on the current thread; it fires once.
#[cfg(feature = "failpoints")]
pub fn arm(step: Step) {
    ARMED.with(|c| c.set(Some(step)));
}

#[cfg(feature = "failpoints")]
pub fn disarm() {
    ARMED.with(|c| c.set(None));
}

#[allow(unused_variables)]
fn fp(step: Step) -> Result<()> {
    #[cfg(feature = "failpoints")]
    if ARMED.with(|c| c.get()) == Some(step) {
        ARMED.with(|c| c.set(None));
        return Err(StoreError::Failpoint(step));
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ChainLine {
    pub conv: String,
    pub seq: u64,
    pub event: String,
    pub msg_id: String,
    pub sender: String,
    pub digest: String,
    pub prev: String,
    pub hash: String,
    /// When the line was written, ms since the unix epoch (B6). Absent on lines
    /// written before B6: such lines still verify, because the field is hashed
    /// only when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts_ms: Option<u64>,
    /// For a stage reported by the other side (B7): the time the peer stamped on
    /// its signed stage record. Hashed only when present, like `ts_ms`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_ts_ms: Option<u64>,
    /// Why, for a REFUSED, DOWNGRADED or switch line (B8). Hashed only when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Optional fields of a chain line beyond the fixed ones.
#[derive(Debug, Clone, Default)]
pub struct LineExtra {
    /// The peer's time on a stage it reported (B7).
    pub peer_ts_ms: Option<u64>,
    /// A short reason (B8). At most 200 bytes; longer is cut.
    pub reason: Option<String>,
}

#[derive(Serialize)]
struct Body<'a> {
    conv: &'a str,
    seq: u64,
    event: &'a str,
    msg_id: &'a str,
    sender: &'a str,
    digest: &'a str,
    prev: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_ts_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
}

/// Wall-clock now in ms since the epoch.
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn sha_hex(b: &[u8]) -> String {
    hex(&Sha256::digest(b))
}

fn line_hash(l: &ChainLine) -> String {
    let body = serde_json::to_string(&Body {
        conv: &l.conv,
        seq: l.seq,
        event: &l.event,
        msg_id: &l.msg_id,
        sender: &l.sender,
        digest: &l.digest,
        prev: &l.prev,
        ts_ms: l.ts_ms,
        peer_ts_ms: l.peer_ts_ms,
        reason: l.reason.as_deref(),
    })
    .expect("serialize body");
    let mut h = Sha256::new();
    h.update(l.prev.as_bytes());
    h.update(body.as_bytes());
    hex(&h.finalize())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub v: u32,
    pub len: u64,
    pub sha256: String,
    pub sender: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChainStatus {
    Intact { len: u64, head: String },
    Broken { seq: u64, reason: String },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PutOutcome {
    Stored { seq: u64 },
    /// Same key and digest: the earlier STORED, nothing written.
    Duplicate { seq: u64 },
    /// Same key, different digest: first record untouched, CONFLICT recorded.
    Conflict { first_seq: u64, conflict_seq: u64 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClaimOutcome {
    Won,
    Held,
}

#[derive(Debug, Default)]
pub struct RecoveryReport {
    pub tmp_removed: usize,
    pub torn_lines_truncated: Vec<String>,
    pub appended: Vec<(String, u64)>,
    pub quarantined: Vec<PathBuf>,
    pub missing: Vec<(String, u64, String)>,
}

pub struct Store {
    dir: PathBuf,
    _lock: File,
    recovery: RecoveryReport,
    /// Serialises chain writes inside this process: read-then-append must not
    /// interleave between the send pump and the receiver (B7).
    write: std::sync::Mutex<()>,
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn check_name(n: &str) -> Result<()> {
    let ok = !n.is_empty()
        && n != "."
        && n != ".."
        && n.len() <= 200
        && n
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':'));
    if ok { Ok(()) } else { Err(StoreError::BadName(n.to_string())) }
}

fn mkdir0700(p: &Path) -> io::Result<()> {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(p)
}

fn fsync_dir(p: &Path) -> io::Result<()> {
    File::open(p)?.sync_all()
}

fn create0600(p: &Path, excl: bool) -> io::Result<File> {
    let mut o = OpenOptions::new();
    o.write(true).mode(0o600);
    if excl {
        o.create_new(true);
    } else {
        o.create(true).truncate(true);
    }
    o.open(p)
}

impl Store {
    /// Open (and recover) the store of `agent` under `store_root`. Refuses a root
    /// outside a marked estate and a store already held by another sidecar.
    pub fn open(store_root: &Path, agent: &str) -> Result<Store> {
        check_name(agent)?;
        estate::guard(store_root, store_root, store_root).map_err(StoreError::Estate)?;
        let dir = store_root.join("sidecar").join(agent);
        mkdir0700(&dir)?;
        for d in ["log", "msgs", "claims", "tmp"] {
            mkdir0700(&dir.join(d))?;
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(dir.join("lock"))?;
        // SAFETY: valid fd owned by `lock`.
        let rc = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            let e = io::Error::last_os_error();
            return Err(if e.raw_os_error() == Some(libc::EWOULDBLOCK) {
                StoreError::Locked
            } else {
                StoreError::Io(e)
            });
        }
        let mut s = Store { dir, _lock: lock, recovery: RecoveryReport::default(), write: std::sync::Mutex::new(()) };
        s.recovery = s.recover()?;
        Ok(s)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn recovery(&self) -> &RecoveryReport {
        &self.recovery
    }

    fn log_path(&self, conv: &str) -> PathBuf {
        self.dir.join("log").join(format!("{conv}.jsonl"))
    }

    fn rec_path(&self, conv: &str, seq: u64, msg_id: &str) -> PathBuf {
        self.dir.join("msgs").join(conv).join(format!("{seq}-{msg_id}.rec"))
    }

    /// Append a needs-attention entry (durable).
    pub fn attention(&self, kind: &str, detail: &str) -> Result<()> {
        let line = serde_json::json!({ "kind": kind, "detail": detail });
        let p = self.dir.join("attention.jsonl");
        let new = !p.exists();
        let mut f = OpenOptions::new().create(true).append(true).mode(0o600).open(&p)?;
        writeln!(f, "{line}")?;
        f.sync_all()?;
        if new {
            fsync_dir(&self.dir)?;
        }
        Ok(())
    }

    pub fn attention_entries(&self) -> Vec<String> {
        fs::read_to_string(self.dir.join("attention.jsonl"))
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// Conversation ids that have a chain log (read-only view for `status`).
    pub fn conversations(&self) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(self.dir.join("log"))
            .map(|d| {
                d.flatten()
                    .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".jsonl").map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        v.sort();
        v
    }

    /// Complete chain lines of a conversation (read-only view for `status`).
    pub fn chain_lines(&self, conv: &str) -> Result<Vec<ChainLine>> {
        check_name(conv)?;
        Ok(self.read_chain(conv)?.0)
    }

    /// Complete lines of the chain plus whether a torn tail was present.
    fn read_chain(&self, conv: &str) -> Result<(Vec<ChainLine>, bool)> {
        let bytes = match fs::read(self.log_path(conv)) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok((vec![], false)),
            Err(e) => return Err(e.into()),
        };
        let torn = !bytes.is_empty() && bytes.last() != Some(&b'\n');
        let complete = if torn {
            match bytes.iter().rposition(|b| *b == b'\n') {
                Some(i) => &bytes[..=i],
                None => &bytes[..0],
            }
        } else {
            &bytes[..]
        };
        let mut out = vec![];
        for (i, raw) in complete.split(|b| *b == b'\n').enumerate() {
            if raw.is_empty() {
                continue;
            }
            let l: ChainLine = serde_json::from_slice(raw)
                .map_err(|e| StoreError::Corrupt(format!("{conv} chain line {i}: {e}")))?;
            out.push(l);
        }
        Ok((out, torn))
    }

    /// Walk the chain; names the first break. States the tail limitation.
    pub fn verify_chain(&self, conv: &str) -> Result<ChainStatus> {
        check_name(conv)?;
        let bytes = match fs::read(self.log_path(conv)) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(ChainStatus::Intact { len: 0, head: GENESIS.into() });
            }
            Err(e) => return Err(e.into()),
        };
        let mut prev = GENESIS.to_string();
        let mut n = 0u64;
        let text_end = bytes.iter().rposition(|b| *b == b'\n').map(|i| i + 1).unwrap_or(0);
        for raw in bytes[..text_end].split(|b| *b == b'\n') {
            if raw.is_empty() {
                continue;
            }
            let broken = |reason: String| Ok(ChainStatus::Broken { seq: n, reason });
            let l: ChainLine = match serde_json::from_slice(raw) {
                Ok(l) => l,
                Err(e) => return broken(format!("unparseable line: {e}")),
            };
            if l.seq != n {
                return broken(format!("sequence jump: expected {n}, found {}", l.seq));
            }
            if l.prev != prev {
                return broken("prev does not match the previous line's hash".into());
            }
            if line_hash(&l) != l.hash {
                return broken("hash does not match line content".into());
            }
            prev = l.hash;
            n += 1;
        }
        Ok(ChainStatus::Intact { len: n, head: prev })
    }

    fn guard(&self) -> std::sync::MutexGuard<'_, ()> {
        self.write.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn append_line(&self, conv: &str, event: &str, msg_id: &str, sender: &str, digest: &str) -> Result<u64> {
        self.append_line_at(conv, event, msg_id, sender, digest, LineExtra::default())
    }

    fn append_line_at(
        &self,
        conv: &str,
        event: &str,
        msg_id: &str,
        sender: &str,
        digest: &str,
        extra: LineExtra,
    ) -> Result<u64> {
        let (lines, _) = self.read_chain(conv)?;
        let seq = lines.len() as u64;
        let prev = lines.last().map(|l| l.hash.clone()).unwrap_or_else(|| GENESIS.into());
        let mut l = ChainLine {
            conv: conv.into(),
            seq,
            event: event.into(),
            msg_id: msg_id.into(),
            sender: sender.into(),
            digest: digest.into(),
            prev,
            hash: String::new(),
            ts_ms: Some(now_ms()),
            peer_ts_ms: extra.peer_ts_ms,
            reason: extra.reason.map(|r| {
                let mut r = r;
                if r.len() > 200 {
                    let mut cut = 200;
                    while !r.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    r.truncate(cut);
                }
                r
            }),
        };
        l.hash = line_hash(&l);
        let mut bytes = serde_json::to_vec(&l).expect("serialize line");
        bytes.push(b'\n');
        let p = self.log_path(conv);
        let first = !p.exists();
        fp(Step::AppendChain)?;
        let mut f = OpenOptions::new().create(true).append(true).mode(0o600).open(&p)?;
        if cfg!(feature = "failpoints") {
            // Simulates dying part-way through the write.
            let half = bytes.len() / 2;
            if fp(Step::AppendChainTorn).is_err() {
                f.write_all(&bytes[..half])?;
                f.sync_all()?;
                return Err(StoreError::Failpoint(Step::AppendChainTorn));
            }
        }
        f.write_all(&bytes)?;
        fp(Step::FsyncLog)?;
        f.sync_all()?;
        if first {
            fsync_dir(&self.dir.join("log"))?;
        }
        Ok(seq)
    }

    /// Record any other stage event (RECEIVED, CLAIMED, HANDED_OVER, REFUSED...).
    pub fn append_event(&self, conv: &str, event: &str, msg_id: &str, sender: &str, digest: &str) -> Result<u64> {
        for n in [conv, msg_id, sender] {
            check_name(n)?;
        }
        let _g = self.guard();
        self.append_line(conv, event, msg_id, sender, digest)
    }

    /// Append `event` unless `pred` already holds over the conversation's lines.
    /// Check and append happen under one lock, so two callers cannot both append.
    /// Returns the new seq, or `None` when `pred` held.
    #[allow(clippy::too_many_arguments)]
    pub fn append_event_unless(
        &self,
        conv: &str,
        event: &str,
        msg_id: &str,
        sender: &str,
        digest: &str,
        extra: LineExtra,
        pred: impl Fn(&[ChainLine]) -> bool,
    ) -> Result<Option<u64>> {
        for n in [conv, msg_id, sender] {
            check_name(n)?;
        }
        let _g = self.guard();
        let (lines, _) = self.read_chain(conv)?;
        if pred(&lines) {
            return Ok(None);
        }
        self.append_line_at(conv, event, msg_id, sender, digest, extra).map(Some)
    }

    /// The hash of the conversation's last line (GENESIS when empty).
    pub fn chain_head(&self, conv: &str) -> Result<String> {
        check_name(conv)?;
        let (lines, _) = self.read_chain(conv)?;
        Ok(lines.last().map(|l| l.hash.clone()).unwrap_or_else(|| GENESIS.into()))
    }

    /// A small named state file in the store (e.g. the receiver's inbox cursor).
    /// Written atomically: tmp, fsync, rename, fsync dir.
    pub fn write_state(&self, name: &str, value: &str) -> Result<()> {
        check_name(name)?;
        let tmp = self.dir.join("tmp").join(format!(
            "state-{}-{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut f = create0600(&tmp, true)?;
        f.write_all(value.as_bytes())?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, self.dir.join(format!("state.{name}")))?;
        fsync_dir(&self.dir)?;
        Ok(())
    }

    pub fn read_state(&self, name: &str) -> Result<Option<String>> {
        check_name(name)?;
        match fs::read_to_string(self.dir.join(format!("state.{name}"))) {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Store a message (steps 2 to 7 of 4.3; step 1, signature check, is the caller's).
    /// `Ok(Stored)` is returned only when record and chain line are both durable.
    pub fn put(&self, sender: &str, conv: &str, msg_id: &str, payload: &[u8]) -> Result<PutOutcome> {
        for n in [sender, conv, msg_id] {
            check_name(n)?;
        }
        let _g = self.guard();
        let digest = sha_hex(payload);
        let (lines, _) = self.read_chain(conv)?;
        if let Some(first) = lines
            .iter()
            .find(|l| l.event == "STORED" && l.sender == sender && l.msg_id == msg_id)
        {
            if first.digest == digest {
                if !self.rec_path(conv, first.seq, msg_id).exists() {
                    return Err(StoreError::MissingRecord { conv: conv.into(), seq: first.seq, msg_id: msg_id.into() });
                }
                return Ok(PutOutcome::Duplicate { seq: first.seq });
            }
            let first_seq = first.seq;
            let known = lines
                .iter()
                .find(|l| l.event == "CONFLICT" && l.sender == sender && l.msg_id == msg_id && l.digest == digest);
            let conflict_seq = match known {
                Some(l) => l.seq,
                None => self.append_line(conv, "CONFLICT", msg_id, sender, &digest)?,
            };
            return Ok(PutOutcome::Conflict { first_seq, conflict_seq });
        }
        let seq = lines.len() as u64;
        let convdir = self.dir.join("msgs").join(conv);
        let new_conv = !convdir.exists();
        mkdir0700(&convdir)?;
        let header = serde_json::to_string(&Header {
            v: 1,
            len: payload.len() as u64,
            sha256: digest.clone(),
            sender: sender.into(),
        })
        .expect("serialize header");
        let tmp = self.dir.join("tmp").join(format!(
            "{}-{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fp(Step::WriteTmp)?;
        // A real I/O error (e.g. ENOSPC) removes the partial temp file, so a
        // retried put does not leak one per attempt and keep the disk full. A
        // failpoint stands for a crash, where no cleanup runs: recovery sweeps
        // `tmp/` at the next start.
        let staged = (|| -> Result<()> {
            let mut f = create0600(&tmp, true)?;
            f.write_all(header.as_bytes())?;
            f.write_all(b"\n")?;
            f.write_all(payload)?;
            fp(Step::FsyncFile)?;
            f.sync_all()?;
            drop(f);
            fp(Step::Rename)?;
            fs::rename(&tmp, self.rec_path(conv, seq, msg_id))?;
            Ok(())
        })();
        if let Err(e) = staged {
            if !matches!(e, StoreError::Failpoint(_)) {
                let _ = fs::remove_file(&tmp);
            }
            return Err(e);
        }
        fp(Step::FsyncDir)?;
        fsync_dir(&convdir)?;
        if new_conv {
            fsync_dir(&self.dir.join("msgs"))?;
        }
        let got = self.append_line(conv, "STORED", msg_id, sender, &digest)?;
        debug_assert_eq!(got, seq);
        fp(Step::Report)?;
        Ok(PutOutcome::Stored { seq })
    }

    fn quarantine(&self, p: &Path, why: &str) -> Result<PathBuf> {
        let mut bad = PathBuf::from(format!("{}.bad", p.display()));
        let mut i = 1;
        while bad.exists() {
            bad = PathBuf::from(format!("{}.bad.{i}", p.display()));
            i += 1;
        }
        fs::rename(p, &bad)?;
        if let Some(parent) = p.parent() {
            fsync_dir(parent)?;
        }
        self.attention("quarantined", &format!("{}: {why}", bad.display()))?;
        Ok(bad)
    }

    fn parse_record(bytes: &[u8]) -> std::result::Result<(Header, Vec<u8>), String> {
        let nl = bytes.iter().position(|b| *b == b'\n').ok_or("no header line")?;
        let h: Header = serde_json::from_slice(&bytes[..nl]).map_err(|e| format!("header: {e}"))?;
        let body = &bytes[nl + 1..];
        if body.len() as u64 != h.len {
            return Err(format!("length {} != header {}", body.len(), h.len));
        }
        if sha_hex(body) != h.sha256 {
            return Err("checksum mismatch".into());
        }
        Ok((h, body.to_vec()))
    }

    /// Read a record, recomputing its checksum. A failing record is quarantined
    /// and never returned.
    pub fn read_record(&self, conv: &str, seq: u64, msg_id: &str) -> Result<(Header, Vec<u8>)> {
        for n in [conv, msg_id] {
            check_name(n)?;
        }
        let p = self.rec_path(conv, seq, msg_id);
        let bytes = match fs::read(&p) {
            Ok(b) => b,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(StoreError::MissingRecord { conv: conv.into(), seq, msg_id: msg_id.into() });
            }
            Err(e) => return Err(e.into()),
        };
        match Self::parse_record(&bytes) {
            Ok(r) => Ok(r),
            Err(why) => Err(StoreError::Quarantined(self.quarantine(&p, &why)?)),
        }
    }

    /// Delivery claim (SI-30): `claims/<msg_id>.<attempt>` created O_EXCL holding
    /// the nonce and time, durable before the caller lets any content leave.
    pub fn claim(&self, msg_id: &str, attempt: u32, nonce: &str, time_secs: u64) -> Result<ClaimOutcome> {
        check_name(msg_id)?;
        let p = self.dir.join("claims").join(format!("{msg_id}.{attempt}"));
        let mut f = match create0600(&p, true) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => return Ok(ClaimOutcome::Held),
            Err(e) => return Err(e.into()),
        };
        writeln!(f, "{nonce}\n{time_secs}")?;
        f.sync_all()?;
        fsync_dir(&self.dir.join("claims"))?;
        Ok(ClaimOutcome::Won)
    }

    /// Attempt numbers already claimed for `msg_id`, ascending.
    pub fn claims_for(&self, msg_id: &str) -> Result<Vec<u32>> {
        check_name(msg_id)?;
        let prefix = format!("{msg_id}.");
        let mut v: Vec<u32> = fs::read_dir(self.dir.join("claims"))?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().and_then(|n| n.strip_prefix(&prefix)?.parse().ok()))
            .collect();
        v.sort_unstable();
        Ok(v)
    }

    fn recover(&self) -> Result<RecoveryReport> {
        let mut r = RecoveryReport::default();
        for e in fs::read_dir(self.dir.join("tmp"))? {
            let e = e?;
            fs::remove_file(e.path())?;
            r.tmp_removed += 1;
        }
        let mut convs: Vec<String> = vec![];
        for e in fs::read_dir(self.dir.join("msgs"))? {
            convs.push(e?.file_name().to_string_lossy().into_owned());
        }
        for e in fs::read_dir(self.dir.join("log"))? {
            let n = e?.file_name().to_string_lossy().into_owned();
            if let Some(c) = n.strip_suffix(".jsonl") {
                convs.push(c.to_string());
            }
        }
        convs.sort();
        convs.dedup();
        for conv in convs {
            if check_name(&conv).is_err() {
                self.attention("recovery", &format!("ignored unsafe conversation name {conv:?}"))?;
                continue;
            }
            // A torn last line was never reported (it follows the record, precedes STORED).
            let (_, torn) = self.read_chain(&conv)?;
            if torn {
                let p = self.log_path(&conv);
                let bytes = fs::read(&p)?;
                let keep = bytes.iter().rposition(|b| *b == b'\n').map(|i| i + 1).unwrap_or(0);
                let f = OpenOptions::new().write(true).open(&p)?;
                f.set_len(keep as u64)?;
                f.sync_all()?;
                r.torn_lines_truncated.push(conv.clone());
            }
            let (lines, _) = self.read_chain(&conv)?;
            let convdir = self.dir.join("msgs").join(&conv);
            if convdir.is_dir() {
                let mut recs: Vec<(u64, String, PathBuf)> = vec![];
                for e in fs::read_dir(&convdir)? {
                    let e = e?;
                    let name = e.file_name().to_string_lossy().into_owned();
                    let Some(stem) = name.strip_suffix(".rec") else { continue };
                    let Some((s, m)) = stem.split_once('-') else { continue };
                    let Ok(seq) = s.parse::<u64>() else { continue };
                    recs.push((seq, m.to_string(), e.path()));
                }
                recs.sort();
                for (seq, msg_id, path) in recs {
                    let parsed = match fs::read(&path) {
                        Ok(b) => Self::parse_record(&b),
                        Err(e) => Err(e.to_string()),
                    };
                    let (h, _) = match parsed {
                        Ok(x) => x,
                        Err(why) => {
                            r.quarantined.push(self.quarantine(&path, &why)?);
                            continue;
                        }
                    };
                    let has_line = lines
                        .iter()
                        .any(|l| l.event == "STORED" && l.seq == seq && l.msg_id == msg_id);
                    if has_line {
                        continue;
                    }
                    let next = self.read_chain(&conv)?.0.len() as u64;
                    if seq == next {
                        self.append_line(&conv, "STORED", &msg_id, &h.sender, &h.sha256)?;
                        r.appended.push((conv.clone(), seq));
                    } else {
                        r.quarantined.push(self.quarantine(&path, "record out of sequence with chain")?);
                    }
                }
            }
            for l in &lines {
                if l.event != "STORED" {
                    continue;
                }
                let good = self.rec_path(&conv, l.seq, &l.msg_id);
                let bad = PathBuf::from(format!("{}.bad", good.display()));
                if !good.exists() && !bad.exists() {
                    self.attention(
                        "chain-line-without-record",
                        &format!("{conv}/{}-{} STORED in chain, record missing", l.seq, l.msg_id),
                    )?;
                    r.missing.push((conv.clone(), l.seq, l.msg_id.clone()));
                }
            }
        }
        Ok(r)
    }
}
