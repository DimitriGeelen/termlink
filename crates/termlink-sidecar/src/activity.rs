//! Activity veto measured by the sidecar immediately before it types the doorbell (T-3387).
//! Activity veto before the doorbell and the derived busy bit (T-3387, OQ-16 D, CR-5).
//!
//! Before typing the doorbell, after the readiness re-check, the sidecar samples
//! the agent's own process tree twice, [`WINDOW_MS`] apart:
//!
//! | signal | source | role |
//! |---|---|---|
//! | bytes SENT per owned socket | `ss -tienH`, matched by inode to the tree's fds (055's method) | primary |
//! | CPU ticks | `/proc/<pid>/stat` utime + stime | secondary |
//! | disk bytes | `/proc/<pid>/io` read_bytes + write_bytes | secondary |
//!
//! Clear activity holds the doorbell (retried). Activity never means ready: a
//! quiet tree only lets an agent the readiness re-check already found READY be
//! rung. Bytes received and `lastrcv` are never read: keep-alive traffic fools
//! them (055). The tree is rooted at the process whose fd/0 is the
//! conversation's pty, never at a recorded claude pid, which for a background
//! session is the shared daemon of every project (055 T-491).
//!
//! Where the host cannot measure (no `/proc`) there is no veto and the gate falls
//! back to the readiness re-check alone; `status` says so. Without `ss` only the
//! secondary signals are used, also shown. The presence heartbeat gets only the
//! derived busy bit and its sample time ([`presence_fields`]); raw figures stay
//! on the host (charter non-goal #2).

use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Sampling window. OQ-16 D: "network send over the last 1-2 s".
pub const WINDOW_MS: u64 = 1500;
/// H-11 starting value. 055 measured megabytes sent per model request against at
/// most 2.8 KB of keep-alive per 30 s, so 16 KiB in 1.5 s cannot be keep-alive.
pub const NET_SENT_BUSY_BYTES: u64 = 16 * 1024;
/// H-11 starting value: CPU time of the whole tree, percent of one core.
pub const CPU_BUSY_PCT: u64 = 25;
/// H-11 starting value: disk bytes read + written by the tree within the window.
pub const DISK_BUSY_BYTES: u64 = 4 * 1024 * 1024;

/// One sample of a process tree. `net_sent` is per socket inode; `None` when
/// the network could not be read (no `ss`).
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub at_ms: u64,
    pub net_sent: Option<HashMap<u64, u64>>,
    pub cpu_ticks: u64,
    pub disk_bytes: u64,
}

/// What the veto found.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Clear activity on the named signal: do not type.
    Active(&'static str),
    /// No clear activity. Says nothing about readiness.
    Quiet,
    /// Could not measure: no veto (OQ-16 D 3).
    Unmeasurable(String),
}

/// What the doorbell may do.
#[derive(Debug, Clone, PartialEq)]
pub enum Gate {
    Ring,
    /// Do not type now; the doorbell is retried later.
    Hold(String),
}

/// OQ-16 D 1-2: ring only when READY at the re-check AND no clear activity.
/// An unmeasurable host has no veto; readiness alone decides.
pub fn gate(ready_at_recheck: bool, v: &Verdict) -> Gate {
    if !ready_at_recheck {
        return Gate::Hold("not ready at the re-check".into());
    }
    match v {
        Verdict::Active(sig) => Gate::Hold(format!("activity veto: {sig}")),
        Verdict::Quiet | Verdict::Unmeasurable(_) => Gate::Ring,
    }
}

/// The presence heartbeat's share: the busy bit and its sample time only.
/// `busy` is null when the host could not measure.
pub fn presence_fields(v: &Verdict, sample_ms: u64) -> Value {
    let busy = match v {
        Verdict::Active(_) => json!(true),
        Verdict::Quiet => json!(false),
        Verdict::Unmeasurable(_) => Value::Null,
    };
    json!({"busy": busy, "busy_sample_ms": sample_ms})
}

/// What `status` shows about the veto on this host.
pub fn availability(proc_root: &Path, ss_found: bool) -> &'static str {
    if !proc_root.join("self").exists() {
        "unavailable: no /proc (no veto; readiness re-check alone)"
    } else if !ss_found {
        "network unmeasured: no ss (CPU and disk only)"
    } else {
        "available"
    }
}

/// [`availability`] on this host.
pub fn host_availability() -> &'static str {
    availability(Path::new("/proc"), on_path("ss"))
}

fn on_path(name: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
}

/// Fields after the LAST ')' of `/proc/<pid>/stat` (comm may hold spaces and ')').
fn stat_rest(proc_root: &Path, pid: u32) -> Option<Vec<String>> {
    let s = std::fs::read_to_string(proc_root.join(pid.to_string()).join("stat")).ok()?;
    Some(s[s.rfind(')')? + 1..].split_whitespace().map(str::to_owned).collect())
}

fn pids(proc_root: &Path) -> Vec<u32> {
    let Ok(rd) = std::fs::read_dir(proc_root) else { return vec![] };
    rd.flatten().filter_map(|e| e.file_name().to_str()?.parse().ok()).collect()
}

fn ppid(proc_root: &Path, pid: u32) -> Option<u32> {
    stat_rest(proc_root, pid)?.get(1)?.parse().ok()
}

/// The top process whose fd/0 is `pty`: one holding it whose parent does not.
pub fn pty_root(proc_root: &Path, pty: &Path) -> Option<u32> {
    let on: HashSet<u32> = pids(proc_root)
        .into_iter()
        .filter(|p| std::fs::read_link(proc_root.join(p.to_string()).join("fd/0")).is_ok_and(|t| t == pty))
        .collect();
    let mut tops: Vec<u32> = on.iter().copied().filter(|p| ppid(proc_root, *p).is_none_or(|pp| !on.contains(&pp))).collect();
    tops.sort_unstable();
    tops.first().copied()
}

/// `root` and all its descendants.
pub fn tree(proc_root: &Path, root: u32) -> Vec<u32> {
    let mut kids: HashMap<u32, Vec<u32>> = HashMap::new();
    for p in pids(proc_root) {
        if let Some(pp) = ppid(proc_root, p) {
            kids.entry(pp).or_default().push(p);
        }
    }
    let mut out = vec![root];
    let mut i = 0;
    while i < out.len() {
        if let Some(k) = kids.get(&out[i]) {
            out.extend(k.iter().filter(|c| !out.contains(c)).copied().collect::<Vec<_>>());
        }
        i += 1;
    }
    out
}

/// Socket inodes held open by `pids` (`socket:[N]` fd links).
pub fn socket_inodes(proc_root: &Path, pids: &[u32]) -> HashSet<u64> {
    let mut out = HashSet::new();
    for p in pids {
        let Ok(rd) = std::fs::read_dir(proc_root.join(p.to_string()).join("fd")) else { continue };
        for e in rd.flatten() {
            if let Ok(t) = std::fs::read_link(e.path())
                && let Some(n) = t.to_str().and_then(|s| s.strip_prefix("socket:[")?.strip_suffix(']')?.parse().ok())
            {
                out.insert(n);
            }
        }
    }
    out
}

/// `ss -tienH` output to inode -> bytes_sent. The inode is on the socket line,
/// the counters on the indented line after it. Only `bytes_sent` is read.
pub fn parse_ss(text: &str) -> HashMap<u64, u64> {
    let field = |l: &str, k: &str| l.split_whitespace().find_map(|w| w.strip_prefix(k)?.parse::<u64>().ok());
    let mut out = HashMap::new();
    let mut ino = None;
    for l in text.lines() {
        if let Some(n) = field(l, "ino:") {
            ino = Some(n);
        }
        if let (Some(n), Some(b)) = (ino, field(l, "bytes_sent:")) {
            out.insert(n, b);
            ino = None;
        }
    }
    out
}

/// One sample of `pids`. `ss` is the raw `ss -tienH` text, `None` if unavailable.
pub fn sample(proc_root: &Path, pids: &[u32], ss: Option<&str>, at_ms: u64) -> Sample {
    let mut cpu = 0;
    let mut disk = 0;
    for p in pids {
        // utime, stime are stat fields 14 and 15: indexes 11 and 12 after ')'.
        if let Some(f) = stat_rest(proc_root, *p) {
            cpu += [11, 12].iter().filter_map(|i| f.get(*i)?.parse::<u64>().ok()).sum::<u64>();
        }
        if let Ok(io) = std::fs::read_to_string(proc_root.join(p.to_string()).join("io")) {
            disk += io
                .lines()
                .filter(|l| l.starts_with("read_bytes:") || l.starts_with("write_bytes:"))
                .filter_map(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
                .sum::<u64>();
        }
    }
    let net_sent = ss.map(|t| {
        let mine = socket_inodes(proc_root, pids);
        parse_ss(t).into_iter().filter(|(i, _)| mine.contains(i)).collect()
    });
    Sample { at_ms, net_sent, cpu_ticks: cpu, disk_bytes: disk }
}

/// Compare two samples. A socket new in `b` counts all it sent.
pub fn judge(a: &Sample, b: &Sample, clk_tck: u64) -> Verdict {
    if let (Some(na), Some(nb)) = (&a.net_sent, &b.net_sent) {
        let sent: u64 = nb.iter().map(|(i, v)| v.saturating_sub(*na.get(i).unwrap_or(&0))).sum();
        if sent >= NET_SENT_BUSY_BYTES {
            return Verdict::Active("network send");
        }
    }
    let window = b.at_ms.saturating_sub(a.at_ms).max(1);
    let cpu_ms = b.cpu_ticks.saturating_sub(a.cpu_ticks) * 1000 / clk_tck.max(1);
    if cpu_ms * 100 / window >= CPU_BUSY_PCT {
        return Verdict::Active("cpu");
    }
    if b.disk_bytes.saturating_sub(a.disk_bytes) >= DISK_BUSY_BYTES {
        return Verdict::Active("disk");
    }
    Verdict::Quiet
}

/// Measure the tree on `pty` over `window_ms`. `ss` is called once per sample.
pub fn measure_at(proc_root: &Path, pty: &Path, window_ms: u64, clk_tck: u64, ss: &dyn Fn() -> Option<String>, now: &dyn Fn() -> u64) -> Verdict {
    if !proc_root.join("self").exists() {
        return Verdict::Unmeasurable("no /proc".into());
    }
    let Some(root) = pty_root(proc_root, pty) else {
        return Verdict::Unmeasurable(format!("no process has {} as fd/0", pty.display()));
    };
    let pids = tree(proc_root, root);
    let a = sample(proc_root, &pids, ss().as_deref(), now());
    std::thread::sleep(std::time::Duration::from_millis(window_ms));
    let b = sample(proc_root, &pids, ss().as_deref(), now());
    judge(&a, &b, clk_tck)
}

/// [`measure_at`] on this host over [`WINDOW_MS`].
pub fn measure(pty: &Path) -> Verdict {
    // SAFETY: sysconf takes no pointers.
    let tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64;
    let ss = || {
        let o = std::process::Command::new("ss").arg("-tienH").stdin(std::process::Stdio::null()).output().ok()?;
        o.status.success().then(|| String::from_utf8_lossy(&o.stdout).into_owned())
    };
    measure_at(Path::new("/proc"), pty, WINDOW_MS, tck, &ss, &crate::store::now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    fn fake_proc(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("tl-activity-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("self")).unwrap();
        d
    }

    fn proc_entry(root: &Path, pid: u32, ppid: u32, fd0: &str, sockets: &[u64], ticks: (u64, u64), disk: (u64, u64)) {
        let d = root.join(pid.to_string());
        std::fs::create_dir_all(d.join("fd")).unwrap();
        std::fs::write(
            d.join("stat"),
            format!("{pid} (we ird) S {ppid} 1 1 0 -1 0 0 0 0 0 {} {} 0 0 20 0 1 0 1 1 1\n", ticks.0, ticks.1),
        )
        .unwrap();
        std::fs::write(d.join("io"), format!("rchar: 9\nwchar: 9\nread_bytes: {}\nwrite_bytes: {}\n", disk.0, disk.1)).unwrap();
        symlink(fd0, d.join("fd/0")).unwrap();
        for (i, s) in sockets.iter().enumerate() {
            symlink(format!("socket:[{s}]"), d.join(format!("fd/{}", i + 3))).unwrap();
        }
    }

    fn ss_text(rows: &[(u64, u64)]) -> String {
        rows.iter()
            .map(|(ino, sent)| {
                format!("ESTAB 0 0 10.0.0.1:1 1.2.3.4:443 ino:{ino} sk:1 <->\n\t ts sack bytes_sent:{sent} bytes_acked:1 bytes_received:999999 lastrcv:1\n")
            })
            .collect()
    }

    #[test]
    fn parse_ss_reads_sent_by_inode_never_received() {
        let m = parse_ss(&ss_text(&[(11, 500), (12, 70_000)]));
        assert_eq!(m.get(&11), Some(&500));
        assert_eq!(m.get(&12), Some(&70_000));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn root_is_the_top_process_on_the_pty_and_tree_follows_children() {
        let p = fake_proc("root");
        proc_entry(&p, 100, 1, "/dev/null", &[], (0, 0), (0, 0)); // daemon: not on the pty
        proc_entry(&p, 200, 100, "/dev/pts/7", &[], (0, 0), (0, 0)); // bg-spare
        proc_entry(&p, 201, 200, "/dev/pts/7", &[], (0, 0), (0, 0)); // shell child, same pty
        proc_entry(&p, 300, 201, "/dev/null", &[], (0, 0), (0, 0)); // grandchild
        proc_entry(&p, 400, 100, "/dev/pts/9", &[], (0, 0), (0, 0)); // another project's session
        assert_eq!(pty_root(&p, Path::new("/dev/pts/7")), Some(200));
        let mut t = tree(&p, 200);
        t.sort_unstable();
        assert_eq!(t, vec![200, 201, 300]);
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn network_send_of_own_socket_is_active_other_sockets_ignored() {
        let p = fake_proc("net");
        proc_entry(&p, 200, 1, "/dev/pts/7", &[11], (0, 0), (0, 0));
        let pids = tree(&p, 200);
        // Socket 99 is not ours: its megabytes must not count.
        let a = sample(&p, &pids, Some(&ss_text(&[(11, 1000), (99, 0)])), 0);
        let b = sample(&p, &pids, Some(&ss_text(&[(11, 1000 + 2_000_000), (99, 5_000_000)])), 1500);
        assert_eq!(judge(&a, &b, 100), Verdict::Active("network send"));
        let c = sample(&p, &pids, Some(&ss_text(&[(11, 1000 + 2_800), (99, 5_000_000)])), 1500);
        assert_eq!(judge(&a, &c, 100), Verdict::Quiet, "keep-alive-sized send is not activity");
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn cpu_and_disk_are_secondary_signals() {
        let a = Sample { at_ms: 0, net_sent: None, cpu_ticks: 0, disk_bytes: 0 };
        let cpu = Sample { at_ms: 1000, net_sent: None, cpu_ticks: 50, disk_bytes: 0 }; // 500 ms of 1000
        assert_eq!(judge(&a, &cpu, 100), Verdict::Active("cpu"));
        let disk = Sample { at_ms: 1000, net_sent: None, cpu_ticks: 1, disk_bytes: DISK_BUSY_BYTES };
        assert_eq!(judge(&a, &disk, 100), Verdict::Active("disk"));
        let idle = Sample { at_ms: 1000, net_sent: None, cpu_ticks: 1, disk_bytes: 4096 };
        assert_eq!(judge(&a, &idle, 100), Verdict::Quiet);
    }

    #[test]
    fn gate_rings_only_when_ready_and_not_active() {
        assert_eq!(gate(true, &Verdict::Quiet), Gate::Ring);
        assert!(matches!(gate(true, &Verdict::Active("network send")), Gate::Hold(r) if r.contains("activity veto")));
        // Activity never means ready, and quiet never makes a not-ready agent ready.
        assert!(matches!(gate(false, &Verdict::Quiet), Gate::Hold(r) if r.contains("not ready")));
        assert!(matches!(gate(false, &Verdict::Active("cpu")), Gate::Hold(_)));
        // No /proc: no veto, readiness alone decides.
        assert_eq!(gate(true, &Verdict::Unmeasurable("no /proc".into())), Gate::Ring);
        assert!(matches!(gate(false, &Verdict::Unmeasurable("no /proc".into())), Gate::Hold(_)));
    }

    #[test]
    fn no_proc_is_unmeasurable_and_shown() {
        let missing = std::env::temp_dir().join(format!("tl-activity-noproc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&missing);
        let v = measure_at(&missing, Path::new("/dev/pts/7"), 0, 100, &|| None, &|| 0);
        assert!(matches!(v, Verdict::Unmeasurable(ref w) if w.contains("no /proc")));
        assert!(availability(&missing, true).starts_with("unavailable: no /proc"));
        let p = fake_proc("avail");
        assert_eq!(availability(&p, true), "available");
        assert!(availability(&p, false).starts_with("network unmeasured"));
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn measure_at_samples_the_pty_tree_twice() {
        let p = fake_proc("measure");
        proc_entry(&p, 200, 1, "/dev/pts/7", &[11], (0, 0), (0, 0));
        let calls = std::cell::Cell::new(0u64);
        let ss = || {
            calls.set(calls.get() + 1);
            Some(ss_text(&[(11, calls.get() * 1_000_000)]))
        };
        let clock = std::cell::Cell::new(0u64);
        let now = || {
            clock.set(clock.get() + 1500);
            clock.get()
        };
        assert_eq!(measure_at(&p, Path::new("/dev/pts/7"), 0, 100, &ss, &now), Verdict::Active("network send"));
        assert_eq!(calls.get(), 2);
        let v = measure_at(&p, Path::new("/dev/pts/8"), 0, 100, &ss, &now);
        assert!(matches!(v, Verdict::Unmeasurable(_)), "nobody on that pty: no veto");
        let _ = std::fs::remove_dir_all(&p);
    }

    #[test]
    fn presence_carries_only_the_busy_bit_and_its_time() {
        let v = presence_fields(&Verdict::Active("network send"), 42);
        assert_eq!(v, json!({"busy": true, "busy_sample_ms": 42}));
        assert_eq!(presence_fields(&Verdict::Quiet, 7), json!({"busy": false, "busy_sample_ms": 7}));
        assert_eq!(presence_fields(&Verdict::Unmeasurable("no /proc".into()), 7), json!({"busy": null, "busy_sample_ms": 7}));
        assert_eq!(v.as_object().unwrap().len(), 2, "no raw telemetry leaves the host");
    }
}
