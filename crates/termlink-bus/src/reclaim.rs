//! Reclaim dead space in topic logs: rewrite a log keeping live records only, offsets unchanged.
//! T-3322 (arc-012, T-3310 D3 operator ruling C).
//!
//! Every trim (sweep, ceiling, per-key compaction) deletes index rows only, so a
//! log file never shrinks: agent-presence measured 31.6 MB on disk against
//! 0.55 MB live. A reclaim rewrites a log that is over
//! [`Ceilings::reclaim_min_file_bytes`] and more than
//! [`Ceilings::reclaim_min_dead_pct`] percent dead, keeping only the records the
//! index still holds. Offsets never change; only `byte_pos` does.
//!
//! Crash safety. The rewrite runs under the topic's append lock (the same lock a
//! post holds from its append through its index commit) and goes:
//!
//! 1. copy the live records into `<hex>.g<N+1>.log.tmp`, fsync it;
//! 2. rename it to `<hex>.g<N+1>.log`, fsync the directory;
//! 3. in ONE SQLite commit (WAL + synchronous=FULL), move every `byte_pos` and
//!    set the topic's generation to N+1;
//! 4. point the appender at the new file and remove the old one.
//!
//! The commit in step 3 is the switch. A crash before it leaves the index wholly
//! on generation N, whose file is untouched; a crash after it leaves the index
//! wholly on N+1, whose file was fsynced and renamed before the commit. Leftover
//! files of the other generation are removed when the bus next opens and before
//! the next reclaim of that topic. Readers take positions and generation from
//! one statement and open that generation's file; a reader that already has the
//! old file open keeps reading it (an unlinked file stays readable while open).

use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::limits::Ceilings;
use crate::log;
use crate::meta::Meta;
use crate::{BusError, Result};

/// Default trigger: the log file is larger than this...
pub const DEFAULT_RECLAIM_MIN_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// ...and more than this percent of it is dead.
pub const DEFAULT_RECLAIM_MIN_DEAD_PCT: u8 = 50;

/// What one reclaim did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct ReclaimOutcome {
    /// The topic's log generation after the rewrite.
    pub generation: u64,
    /// Live records copied.
    pub records: u64,
    /// Log size before and after.
    pub old_bytes: u64,
    pub new_bytes: u64,
}

/// Pure trigger: `file_bytes` on disk, `live_bytes` of it still indexed
/// (payloads plus their 8-byte length prefixes). A percentage of 100 or more
/// never fires, which switches reclaim off.
pub fn should_reclaim(file_bytes: u64, live_bytes: u64, c: &Ceilings) -> bool {
    if file_bytes <= c.reclaim_min_file_bytes {
        return false;
    }
    let dead = file_bytes.saturating_sub(live_bytes);
    u128::from(dead) * 100 > u128::from(file_bytes) * u128::from(c.reclaim_min_dead_pct)
}

/// Test seam: points at which a simulated crash abandons the rewrite exactly as
/// a killed process would (no cleanup runs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CrashPoint {
    AfterTempWrite,
    AfterRename,
    AfterCommit,
}

#[cfg(test)]
thread_local! {
    pub(crate) static CRASH_AT: std::cell::Cell<Option<CrashPoint>> = const { std::cell::Cell::new(None) };
}

/// True while a test is simulating a crash: a killed process runs no cleanup,
/// so the error paths below must not either.
fn simulating_crash() -> bool {
    #[cfg(test)]
    if CRASH_AT.with(|c| c.get()).is_some() {
        return true;
    }
    false
}

fn crash_here(_p: CrashPoint) -> Result<()> {
    #[cfg(test)]
    if CRASH_AT.with(|c| c.get()) == Some(_p) {
        return Err(BusError::Io(std::io::Error::other(format!("T-3322 test crash at {_p:?}"))));
    }
    Ok(())
}

/// Remove every log file of `topic` other than generation `keep` (leftovers of
/// a crash mid-rewrite, or the old file a crash after the commit did not get to
/// remove). Files of other topics are never touched.
pub(crate) fn remove_stale_generations(root: &Path, topic: &str, keep: u64) -> Result<()> {
    let hex = log::topic_hex(topic);
    let keep_name = log::log_file_name(&hex, keep);
    let prefix = format!("{hex}.");
    let dir = root.join("topics");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        let name = entry?.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with(&prefix) && name != keep_name {
            remove_reporting(&dir.join(name));
        }
    }
    Ok(())
}

/// The same cleanup for every registered topic, from one directory listing
/// (run when the bus opens). Files whose stem matches no registered topic are
/// left alone.
pub(crate) fn remove_stale_generations_all(root: &Path, current: &[(String, u64)]) -> Result<()> {
    let keep: std::collections::HashMap<String, String> = current
        .iter()
        .map(|(t, g)| {
            let hex = log::topic_hex(t);
            let name = log::log_file_name(&hex, *g);
            (hex, name)
        })
        .collect();
    let dir = root.join("topics");
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        let name = entry?.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(stem) = name.get(..64) else { continue };
        if name.as_bytes().get(64) != Some(&b'.') {
            continue;
        }
        if let Some(keep_name) = keep.get(stem)
            && name != keep_name
        {
            remove_reporting(&dir.join(name));
        }
    }
    Ok(())
}

/// Remove every log file of `topic`, all generations (topic deleted). Quiet:
/// a missing file is the normal case for a topic never posted to.
pub(crate) fn remove_topic_files(root: &Path, topic: &str) {
    let prefix = format!("{}.", log::topic_hex(topic));
    let dir = root.join("topics");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_str().is_some_and(|n| n.starts_with(&prefix)) {
            let _ = std::fs::remove_file(dir.join(&name));
        }
    }
}

fn remove_reporting(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => eprintln!("termlink-bus: T-3322 removed stale log file {}", path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => eprintln!("warning: T-3322: could not remove stale log file {}: {e}", path.display()),
    }
}

fn sync_dir(dir: &Path) -> Result<()> {
    File::open(dir)?.sync_all()?;
    Ok(())
}

/// Rewrite `topic`'s log if it meets the trigger (or always, with `force`).
/// The caller holds the topic's append lock; `appender` is the file behind it
/// and is replaced by the new generation's file on success.
pub(crate) fn rewrite_locked(
    root: &Path,
    meta: &Meta,
    topic: &str,
    appender: &mut File,
    c: &Ceilings,
    force: bool,
) -> Result<Option<ReclaimOutcome>> {
    // The generation changes only under the lock we hold, so these two reads agree.
    let generation = meta.log_generation(topic)?;
    let old_path = log::topic_log_path_gen(root, topic, generation);
    let old_bytes = match std::fs::metadata(&old_path) {
        Ok(m) => m.len(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    // Records swept after this read are fine (see `Meta::commit_rewrite`); no
    // record can be added, a post commits its index entry under our lock.
    let locs = meta.records_from(topic, 0)?;
    let live: u64 = locs.iter().map(|l| 8 + l.length).sum();
    if !force && !should_reclaim(old_bytes, live, c) {
        return Ok(None);
    }
    let new_gen = generation + 1;
    let new_path = log::topic_log_path_gen(root, topic, new_gen);
    let mut tmp_name = new_path.clone().into_os_string();
    tmp_name.push(".tmp");
    let tmp_path = std::path::PathBuf::from(tmp_name);
    let dir = root.join("topics");

    // Steps 1 and 2: the new file, durable and in place, before the index moves.
    let (moves, new_bytes, new_append) = crate::run_blocking(|| -> Result<_> {
        remove_stale_generations(root, topic, generation)?;
        let mut src = File::open(&old_path)?;
        let mut out = BufWriter::new(File::create(&tmp_path)?);
        let mut moves = Vec::with_capacity(locs.len());
        let mut pos = 0u64;
        let mut buf = Vec::new();
        for loc in &locs {
            let len = usize::try_from(loc.length).map_err(|_| {
                BusError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "record length overflows usize",
                ))
            })?;
            buf.resize(len, 0);
            src.seek(SeekFrom::Start(loc.byte_pos + 8))?;
            src.read_exact(&mut buf).map_err(|e| {
                // The index points at bytes the file does not hold (a pre-T-3461
                // record or disk damage). Readers skip it as a gap; a rewrite has
                // nowhere honest to point it, so it leaves the topic alone.
                BusError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "T-3322: reclaim of {topic} aborted, record at offset {} (byte_pos {}, {} bytes) \
                         cannot be read: {e}",
                        loc.offset, loc.byte_pos, loc.length
                    ),
                ))
            })?;
            out.write_all(&loc.length.to_be_bytes())?;
            out.write_all(&buf)?;
            moves.push((loc.offset, pos));
            pos += 8 + loc.length;
        }
        let f = out.into_inner().map_err(|e| BusError::Io(e.into_error()))?;
        f.sync_all()?;
        drop(f);
        crash_here(CrashPoint::AfterTempWrite)?;
        std::fs::rename(&tmp_path, &new_path)?;
        sync_dir(&dir)?;
        // Opened before the switch, so nothing can fail between the commit and
        // the appender moving to the new file.
        let new_append = log::open_append(&new_path)?;
        Ok((moves, pos, new_append))
    })
    .inspect_err(|_| {
        // Nothing is switched yet: drop whatever this attempt wrote.
        if !simulating_crash() {
            let _ = std::fs::remove_file(&tmp_path);
            let _ = std::fs::remove_file(&new_path);
        }
    })?;
    crash_here(CrashPoint::AfterRename)?;

    // Step 3: the switch.
    if let Err(e) = meta.commit_rewrite(topic, new_gen, &moves) {
        drop(new_append);
        let _ = std::fs::remove_file(&new_path);
        return Err(e);
    }
    crash_here(CrashPoint::AfterCommit)?;

    // Step 4.
    *appender = new_append;
    if let Err(e) = std::fs::remove_file(&old_path) {
        eprintln!(
            "warning: T-3322: reclaimed {topic} but could not remove the old log {}: {e} \
             (removed on the next open or reclaim)",
            old_path.display()
        );
    }
    Ok(Some(ReclaimOutcome { generation: new_gen, records: moves.len() as u64, old_bytes, new_bytes }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bus, Envelope, Retention};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn env(i: u64) -> Envelope {
        Envelope {
            topic: "t".into(),
            sender_id: "test".into(),
            msg_type: "note".into(),
            // ~1 KB, and the payload names its offset so every read can be checked.
            payload: format!("p{i}:{}", "x".repeat(1000)).into_bytes(),
            artifact_ref: None,
            ts_unix_ms: 0,
            metadata: std::collections::BTreeMap::new(),
            signature: None,
        }
    }

    fn payload_offset(e: &Envelope) -> u64 {
        let s = std::str::from_utf8(&e.payload).unwrap();
        s[1..s.find(':').unwrap()].parse().unwrap()
    }

    /// Ceilings off (so only the test trims), reclaim at any size past half dead,
    /// or never when `auto` is false.
    fn open(dir: &Path, auto: bool) -> Bus {
        let bus = Bus::open(dir).unwrap();
        bus.set_ceilings(Ceilings {
            on_post: false,
            reclaim_min_file_bytes: 0,
            reclaim_min_dead_pct: if auto { 50 } else { 100 },
            ..Ceilings::default()
        });
        bus
    }

    /// Every record readable, each with the payload of its own offset.
    fn read_all(bus: &Bus) -> Vec<u64> {
        bus.subscribe("t", 0)
            .unwrap()
            .map(|r| {
                let (o, e) = r.unwrap();
                assert_eq!(payload_offset(&e), o, "record at offset {o} carries another record's bytes");
                o
            })
            .collect()
    }

    fn log_files(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir.join("topics"))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        v.sort();
        v
    }

    fn current_log(bus: &Bus) -> std::path::PathBuf {
        log::topic_log_path_gen(&bus.root, "t", bus.meta.log_generation("t").unwrap())
    }

    /// 20 records, the oldest 15 trimmed (no reclaim yet).
    async fn seeded(dir: &Path) -> Bus {
        let bus = open(dir, false);
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..20 {
            assert_eq!(bus.post("t", &env(i)).await.unwrap(), i);
        }
        bus.trim_topic("t", Some(15)).unwrap();
        bus
    }

    #[tokio::test]
    async fn reclaim_shrinks_the_log_and_keeps_offsets_and_payloads() {
        let dir = TempDir::new().unwrap();
        let bus = seeded(dir.path()).await;
        let before = std::fs::metadata(current_log(&bus)).unwrap().len();
        assert_eq!(read_all(&bus), (15..20).collect::<Vec<_>>());
        assert!(bus.reclaim("t", false).await.unwrap().is_none(), "reclaim is off (100 %)");

        bus.set_ceilings(Ceilings { reclaim_min_dead_pct: 50, ..bus.ceilings() });
        let out = bus.reclaim("t", false).await.unwrap().expect("15 of 20 dead: reclaims");
        assert_eq!((out.generation, out.records, out.old_bytes), (1, 5, before));
        let after = std::fs::metadata(current_log(&bus)).unwrap().len();
        assert_eq!(after, out.new_bytes);
        assert!(after * 3 < before, "log shrank: {before} -> {after}");
        assert_eq!(log_files(dir.path()).len(), 1, "old generation removed: {:?}", log_files(dir.path()));
        assert_eq!(read_all(&bus), (15..20).collect::<Vec<_>>(), "offsets and payloads unchanged");
        assert_eq!(bus.envelope_at("t", 17).unwrap().map(|e| payload_offset(&e)), Some(17));
        let c = bus.ceiling_counters();
        assert_eq!((c.reclaims_total, c.reclaimed_bytes_total), (1, before - after));

        // Posts after the rewrite append to the new file at the next offset.
        assert_eq!(bus.post("t", &env(20)).await.unwrap(), 20);
        assert_eq!(read_all(&bus), (15..21).collect::<Vec<_>>());
        // And all of it survives a reopen.
        drop(bus);
        let bus = open(dir.path(), false);
        assert_eq!(read_all(&bus), (15..21).collect::<Vec<_>>());
        assert_eq!(bus.post("t", &env(21)).await.unwrap(), 21);
        assert_eq!(read_all(&bus), (15..22).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn below_either_threshold_nothing_is_rewritten() {
        let dir = TempDir::new().unwrap();
        let bus = open(dir.path(), true);
        bus.create_topic("t", Retention::Forever).unwrap();
        for i in 0..10 {
            bus.post("t", &env(i)).await.unwrap();
        }
        bus.trim_topic("t", Some(5)).unwrap(); // exactly half dead
        assert!(bus.reclaim("t", false).await.unwrap().is_none(), "half is not more than half");
        bus.set_ceilings(Ceilings { reclaim_min_file_bytes: 1 << 30, ..bus.ceilings() });
        bus.trim_topic("t", Some(9)).unwrap();
        assert!(bus.reclaim("t", false).await.unwrap().is_none(), "file under the size floor");
        assert_eq!(bus.meta.log_generation("t").unwrap(), 0);
        assert_eq!(bus.ceiling_counters().reclaims_total, 0);
    }

    #[tokio::test]
    async fn a_sweep_that_leaves_the_log_mostly_dead_reclaims_it() {
        let dir = TempDir::new().unwrap();
        let bus = open(dir.path(), true);
        bus.create_topic("t", Retention::Messages(4)).unwrap();
        for i in 0..20 {
            bus.post("t", &env(i)).await.unwrap();
        }
        assert_eq!(bus.sweep("t", 0).unwrap(), 16);
        assert_eq!(bus.ceiling_counters().reclaims_total, 1, "the sweep triggered the rewrite");
        assert_eq!(bus.meta.log_generation("t").unwrap(), 1);
        assert_eq!(read_all(&bus), (16..20).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn crash_at_each_step_then_reopen_loses_nothing() {
        for point in [CrashPoint::AfterTempWrite, CrashPoint::AfterRename, CrashPoint::AfterCommit] {
            let dir = TempDir::new().unwrap();
            let bus = seeded(dir.path()).await;
            CRASH_AT.with(|c| c.set(Some(point)));
            let r = bus.reclaim("t", true).await;
            CRASH_AT.with(|c| c.set(None));
            assert!(r.is_err(), "{point:?}: the simulated crash aborts the rewrite");
            let files = log_files(dir.path());
            assert_eq!(files.len(), 2, "{point:?}: a crash leaves both generations: {files:?}");
            // The process dies here; nothing else runs on this bus.
            drop(bus);

            let bus = open(dir.path(), false);
            let expect_gen = if point == CrashPoint::AfterCommit { 1 } else { 0 };
            assert_eq!(bus.meta.log_generation("t").unwrap(), expect_gen, "{point:?}");
            assert_eq!(log_files(dir.path()).len(), 1, "{point:?}: reopen removed the leftover");
            assert_eq!(read_all(&bus), (15..20).collect::<Vec<_>>(), "{point:?}: every live record intact");
            assert_eq!(bus.post("t", &env(20)).await.unwrap(), 20, "{point:?}");
            assert_eq!(read_all(&bus), (15..21).collect::<Vec<_>>(), "{point:?}");
            // A later reclaim still works and still keeps everything.
            let out = bus.reclaim("t", true).await.unwrap().expect("reclaim after recovery");
            assert_eq!(out.generation, expect_gen + 1, "{point:?}");
            assert_eq!(read_all(&bus), (15..21).collect::<Vec<_>>(), "{point:?}");
            assert_eq!(log_files(dir.path()).len(), 1, "{point:?}");
        }
    }

    #[tokio::test]
    async fn a_leftover_from_a_crash_is_cleared_before_the_next_rewrite() {
        // Crash without a reopen (e.g. a failed rename); the next reclaim on the
        // same bus must not trip over the leftover temp or new-generation file.
        let dir = TempDir::new().unwrap();
        let bus = seeded(dir.path()).await;
        CRASH_AT.with(|c| c.set(Some(CrashPoint::AfterRename)));
        assert!(bus.reclaim("t", true).await.is_err());
        CRASH_AT.with(|c| c.set(None));
        std::fs::write(dir.path().join("topics").join(format!("{}.g1.log.tmp", log::topic_hex("t"))), b"junk")
            .unwrap();
        assert_eq!(bus.post("t", &env(20)).await.unwrap(), 20, "the old file is still current");
        let out = bus.reclaim("t", true).await.unwrap().unwrap();
        assert_eq!((out.generation, out.records), (1, 6));
        assert_eq!(log_files(dir.path()).len(), 1, "{:?}", log_files(dir.path()));
        assert_eq!(read_all(&bus), (15..21).collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn an_unreadable_record_aborts_the_rewrite_and_leaves_the_log_alone() {
        let dir = TempDir::new().unwrap();
        let bus = seeded(dir.path()).await;
        // Cut the file mid-record: the index now points past its end (T-2487 shape).
        let path = current_log(&bus);
        let len = std::fs::metadata(&path).unwrap().len();
        std::fs::OpenOptions::new().write(true).open(&path).unwrap().set_len(len - 10).unwrap();
        let err = bus.reclaim("t", true).await.unwrap_err();
        assert!(err.to_string().contains("offset 19"), "names the record: {err}");
        assert_eq!(bus.meta.log_generation("t").unwrap(), 0);
        assert_eq!(log_files(dir.path()).len(), 1, "the aborted attempt removed its temp file");
        assert_eq!(read_all(&bus), (15..19).collect::<Vec<_>>(), "readers still skip the gap");
    }

    #[tokio::test]
    async fn a_reader_opened_before_the_rewrite_keeps_reading_the_old_file() {
        let dir = TempDir::new().unwrap();
        let bus = seeded(dir.path()).await;
        let mut it = bus.subscribe("t", 0).unwrap();
        let first = it.next().unwrap().unwrap();
        bus.reclaim("t", true).await.unwrap().unwrap();
        bus.post("t", &env(20)).await.unwrap();
        let mut got = vec![first.0];
        for r in it {
            let (o, e) = r.unwrap();
            assert_eq!(payload_offset(&e), o);
            got.push(o);
        }
        assert_eq!(got, (15..20).collect::<Vec<_>>(), "old snapshot, old file, right bytes");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn readers_never_see_a_broken_log_during_posts_trims_and_rewrites() {
        let dir = TempDir::new().unwrap();
        let bus = Arc::new(open(dir.path(), true));
        bus.create_topic("t", Retention::Forever).unwrap();
        const N: u64 = 300;
        let done = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let poster = {
            let bus = bus.clone();
            tokio::spawn(async move {
                for i in 0..N {
                    assert_eq!(bus.post("t", &env(i)).await.unwrap(), i, "offsets stay dense");
                }
            })
        };
        let rewriter = {
            let (bus, done) = (bus.clone(), done.clone());
            tokio::spawn(async move {
                let mut rewrites = 0u64;
                while !done.load(std::sync::atomic::Ordering::SeqCst) {
                    if let Some(latest) = bus.latest_offset("t").unwrap() {
                        // trim_topic reclaims on its own when the lock is free...
                        bus.trim_topic("t", Some(latest.saturating_sub(5))).unwrap();
                    }
                    // ...and this one waits for the lock, racing the poster.
                    if bus.reclaim("t", true).await.unwrap().is_some() {
                        rewrites += 1;
                    }
                    tokio::task::yield_now().await;
                }
                rewrites
            })
        };
        let mut readers = Vec::new();
        for _ in 0..3 {
            let (bus, done) = (bus.clone(), done.clone());
            readers.push(tokio::spawn(async move {
                let mut reads = 0u64;
                while !done.load(std::sync::atomic::Ordering::SeqCst) {
                    for r in bus.subscribe("t", 0).unwrap() {
                        let (o, e) = r.expect("no read error, ever");
                        assert_eq!(payload_offset(&e), o, "offset {o} read another record's bytes");
                        reads += 1;
                    }
                    tokio::task::yield_now().await;
                }
                reads
            }));
        }
        poster.await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        done.store(true, std::sync::atomic::Ordering::SeqCst);
        let rewrites = rewriter.await.unwrap();
        let mut reads = 0;
        for r in readers {
            reads += r.await.unwrap();
        }
        assert!(rewrites > 10, "the race was exercised: {rewrites} rewrites");
        assert!(reads > 0);
        // Whatever survived the trims is intact and contiguous up to the last post.
        let live = read_all(&bus);
        assert_eq!(live.last(), Some(&(N - 1)));
        assert!(live.windows(2).all(|w| w[1] == w[0] + 1), "{live:?}");
        assert_eq!(bus.meta.synchronous_level(), 2, "meta still synchronous=FULL");
    }

    fn c(min: u64, pct: u8) -> Ceilings {
        Ceilings { reclaim_min_file_bytes: min, reclaim_min_dead_pct: pct, ..Ceilings::default() }
    }

    #[test]
    fn trigger_needs_both_size_and_dead_share() {
        let d = Ceilings::default();
        assert_eq!(d.reclaim_min_file_bytes, 8 * 1024 * 1024);
        assert_eq!(d.reclaim_min_dead_pct, 50);
        let mb = 1024 * 1024;
        assert!(!should_reclaim(8 * mb, 0, &d), "exactly 8 MB is not over 8 MB");
        assert!(should_reclaim(9 * mb, 4 * mb, &d), "9 MB, 5 MB dead");
        assert!(!should_reclaim(9 * mb, 5 * mb, &d), "under half dead");
        assert!(!should_reclaim(10 * mb, 5 * mb, &d), "exactly half is not more than half");
        assert!(!should_reclaim(100, 0, &c(10, 100)), "100 % switches it off");
        assert!(should_reclaim(100, 0, &c(10, 0)));
        assert!(!should_reclaim(u64::MAX, 0, &c(0, 100)), "no overflow");
    }
}
