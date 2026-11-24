//! Disposable estate test harness with its own HOME, runtime dir, keys and throwaway hub.
//! C-12 core: the disposable estate harness (E-1, E-2, E-3, E-5).
//!
//! `Estate::create()` builds a marked throwaway tree with its own HOME, runtime
//! dir and keys, exports a secret canary, and starts a throwaway hub whose only
//! environment is the estate's. Nothing outside the tree is read or written.

use crate::estate::{MARKER, MARKER_MAGIC, STORE_ENV};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime};

pub const CANARY_ENV: &str = "TERMLINK_ESTATE_CANARY";

pub struct Estate {
    root: PathBuf,
    hub: Option<Child>,
    canary: String,
    prod_before: Vec<(PathBuf, Snapshot)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Snapshot {
    exists: bool,
    newest: Option<SystemTime>,
    entries: usize,
}

fn snapshot(p: &Path) -> Snapshot {
    fn walk(p: &Path, newest: &mut Option<SystemTime>, n: &mut usize, depth: u8) {
        if let Ok(md) = fs::symlink_metadata(p) {
            *n += 1;
            if let Ok(m) = md.modified() {
                *newest = Some(newest.map_or(m, |c| c.max(m)));
            }
            if md.is_dir()
                && depth < 4
                && let Ok(rd) = fs::read_dir(p)
            {
                for e in rd.flatten() {
                    walk(&e.path(), newest, n, depth + 1);
                }
            }
        }
    }
    let (mut newest, mut entries) = (None, 0);
    walk(p, &mut newest, &mut entries, 0);
    Snapshot {
        exists: p.exists(),
        newest,
        entries,
    }
}

/// The production locations the estate must never touch.
fn production_paths() -> Vec<PathBuf> {
    let uid = unsafe { libc::getuid() };
    let mut v = vec![
        PathBuf::from("/var/lib/termlink"),
        PathBuf::from(format!("/tmp/termlink-{uid}")),
    ];
    if let Some(h) = std::env::var_os("HOME") {
        v.push(PathBuf::from(h).join(".termlink"));
    }
    v
}

fn random_hex(bytes: usize) -> std::io::Result<String> {
    let mut buf = vec![0u8; bytes];
    fs::File::open("/dev/urandom")?.read_exact(&mut buf)?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

/// A load-sensitive test bound scaled by `TERMLINK_TEST_TIME_FACTOR` (default 1, values
/// below 1 or unparseable read as 1). Same dial and parsing as the hub crate's
/// `test_budget` (T-3479), so one variable widens both crates on a contended host (T-3535).
pub fn test_budget(base: Duration) -> Duration {
    base * time_factor(std::env::var("TERMLINK_TEST_TIME_FACTOR").ok().as_deref())
}

fn time_factor(raw: Option<&str>) -> u32 {
    raw.and_then(|v| v.trim().parse::<u32>().ok()).filter(|f| *f >= 1).unwrap_or(1)
}

/// Locate the `termlink` binary used for the throwaway hub.
///
/// Order: `TERMLINK_BIN`; then (T-3677) the `termlink` beside the running test
/// binary (`<target-dir>/<profile>/deps/x` → `<target-dir>/<profile>/termlink`),
/// so a build with `--target-dir` elsewhere (node1 runs from a read-only source
/// tree with `--target-dir ../target`) finds the binary of the same build; then
/// the workspace's own `target/{debug,release}`, newest first.
///
/// T-3855: `cargo test -p termlink-sidecar` does not build the `termlink`
/// package, so every fallback can be built from older source (a 2026-11-15
/// release binary failed t3366 on correct HEAD). A fallback older than the
/// newest hub source file ([`hub_source_roots`]) is skipped; when none is
/// current the error says which binary, which newer file, and how to fix it.
/// An explicit `TERMLINK_BIN` is used as given.
pub fn termlink_bin() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("TERMLINK_BIN") {
        return Ok(PathBuf::from(p));
    }
    let mut candidates = Vec::new();
    if let Some(profile_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .filter(|deps| deps.file_name().is_some_and(|n| n == "deps"))
        .and_then(|deps| deps.parent().map(Path::to_path_buf))
    {
        candidates.push(profile_dir.join("termlink"));
    }
    let ws = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ws = ws.canonicalize().unwrap_or(ws);
    let mut ws_bins: Vec<PathBuf> = ["debug", "release"]
        .iter()
        .map(|d| ws.join("target").join(d).join("termlink"))
        .filter(|p| p.is_file())
        .collect();
    ws_bins.sort_by_key(|p| std::cmp::Reverse(mtime(p)));
    candidates.extend(ws_bins);
    // The binary beside the test is usually one of the workspace ones: list each file once.
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.canonicalize().unwrap_or_else(|_| c.clone())));
    pick_fresh(&candidates, newest_file(&hub_source_roots(&ws)))
}

/// What the estate's hub binary is built from (T-3855). The sidecar crate is
/// left out on purpose: the CLI links it, but counting it would force a CLI
/// rebuild after every sidecar edit while the hub under test is unchanged.
pub fn hub_source_roots(ws: &Path) -> Vec<PathBuf> {
    let mut roots = vec![ws.join("Cargo.lock"), ws.join("crates/version-derive.rs")];
    for c in ["termlink-hub", "termlink-session", "termlink-protocol", "termlink-bus", "termlink-cli"] {
        roots.push(ws.join("crates").join(c).join("src"));
        roots.push(ws.join("crates").join(c).join("Cargo.toml"));
    }
    roots
}

fn mtime(p: &Path) -> Option<std::time::SystemTime> {
    fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// The newest file under `roots` (files or directories, walked recursively)
/// and its mtime; None when none can be read.
pub fn newest_file(roots: &[PathBuf]) -> Option<(PathBuf, std::time::SystemTime)> {
    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    let mut stack: Vec<PathBuf> = roots.to_vec();
    while let Some(p) = stack.pop() {
        let Ok(meta) = fs::metadata(&p) else { continue };
        if meta.is_dir() {
            if let Ok(rd) = fs::read_dir(&p) {
                stack.extend(rd.filter_map(|e| e.ok().map(|e| e.path())));
            }
        } else if let Ok(t) = meta.modified()
            && best.as_ref().is_none_or(|(_, b)| t > *b)
        {
            best = Some((p, t));
        }
    }
    best
}

/// The first existing candidate not older than `newest_source`. With no
/// readable source there is nothing to compare, so the first existing one wins.
pub fn pick_fresh(
    candidates: &[PathBuf],
    newest_source: Option<(PathBuf, std::time::SystemTime)>,
) -> Result<PathBuf, String> {
    let mut stale = Vec::new();
    for c in candidates.iter().filter(|c| c.is_file()) {
        match (&newest_source, mtime(c)) {
            (Some((_, src_t)), Some(bin_t)) if bin_t < *src_t => stale.push(c.display().to_string()),
            _ => return Ok(c.clone()),
        }
    }
    let fix = "run `cargo build -p termlink` (same profile and --target-dir as this test run), \
               or set TERMLINK_BIN to a current termlink binary";
    match (stale.is_empty(), newest_source) {
        (true, _) => Err(format!("termlink binary not found; {fix}")),
        (false, Some((src, _))) => Err(format!(
            "termlink binary is older than its source (T-3855): {} predate(s) {}; {fix}",
            stale.join(", "),
            src.display()
        )),
        (false, None) => unreachable!("a candidate is only stale against a source file"),
    }
}

impl Estate {
    /// Create the tree, keys, projects and canary only (no hub).
    pub fn create_without_hub() -> std::io::Result<Self> {
        let prod_before = production_paths()
            .into_iter()
            .map(|p| {
                let s = snapshot(&p);
                (p, s)
            })
            .collect();
        let root = PathBuf::from(format!("/tmp/tl-estate-{}", random_hex(6)?));
        for d in ["home", "run", "xdg", "store", "keys", "proj-a", "proj-b"] {
            fs::create_dir_all(root.join(d))?;
        }
        let canary = format!("CANARY-{}", random_hex(16)?);
        for p in ["proj-a", "proj-b"] {
            fs::write(
                root.join(p).join("TASK.md"),
                format!("scratch task for {p}\n"),
            )?;
        }
        let est = Estate {
            root,
            hub: None,
            canary,
            prod_before,
        };
        // E-5: per-run keys, deleted with the tree.
        for who in ["a", "b"] {
            termlink_session::agent_identity::Identity::load_or_create_from_file(
                &est.root.join("keys").join(format!("{who}.key")),
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        }
        // Marker last: a tree without it is never trusted.
        fs::write(est.root.join(MARKER), format!("{MARKER_MAGIC}\n"))?;
        Ok(est)
    }

    /// Create the estate and start its throwaway hub (E-1).
    pub fn create() -> std::io::Result<Self> {
        let mut est = Self::create_without_hub()?;
        est.start_hub()?;
        Ok(est)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn home(&self) -> PathBuf {
        self.root.join("home")
    }
    pub fn runtime_dir(&self) -> PathBuf {
        self.root.join("run")
    }
    pub fn store_root(&self) -> PathBuf {
        self.root.join("store")
    }
    pub fn canary(&self) -> &str {
        &self.canary
    }
    pub fn hub_pid(&self) -> Option<u32> {
        self.hub.as_ref().map(Child::id)
    }

    /// The complete environment for any child (E-2): nothing is inherited.
    pub fn env(&self) -> BTreeMap<String, String> {
        let s = |p: PathBuf| p.display().to_string();
        BTreeMap::from([
            ("HOME".into(), s(self.home())),
            ("TERMLINK_RUNTIME_DIR".into(), s(self.runtime_dir())),
            ("XDG_RUNTIME_DIR".into(), s(self.root.join("xdg"))),
            (STORE_ENV.into(), s(self.store_root())),
            (CANARY_ENV.into(), self.canary.clone()),
            ("PATH".into(), "/usr/local/bin:/usr/bin:/bin".into()),
        ])
    }

    /// A command with `env -i` semantics plus the estate environment.
    pub fn command(&self, program: impl AsRef<std::ffi::OsStr>) -> Command {
        let mut c = Command::new(program);
        c.env_clear().envs(self.env()).current_dir(&self.root);
        c
    }

    /// Where the throwaway hub listens.
    pub fn hub_addr(&self) -> termlink_protocol::TransportAddr {
        termlink_protocol::TransportAddr::unix(self.runtime_dir().join("hub.sock"))
    }

    /// The throwaway hub's own id, as `hub.version` states it. A hub that is up
    /// (it is only called while one runs) but slow to answer on a loaded host is
    /// asked again, for up to 90 s in all (a fresh debug-build hub on a host at
    /// load average 40+ has been measured to take over 30 s to answer). T-3535: at
    /// load ~60 even 90 s ran out, so the bound scales by `TERMLINK_TEST_TIME_FACTOR`,
    /// the same dial the hub crate's load-sensitive tests read (T-3479).
    pub fn hub_id(&self) -> std::io::Result<String> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let deadline = std::time::Instant::now() + test_budget(Duration::from_secs(90));
        let resp = loop {
            match rt.block_on(termlink_session::client::rpc_call_addr_with_timeout(
                &self.hub_addr(),
                "hub.version",
                serde_json::json!({}),
                Duration::from_secs(5),
            )) {
                Ok(r) => break r,
                Err(e) if std::time::Instant::now() >= deadline => return Err(std::io::Error::other(e.to_string())),
                Err(_) => std::thread::sleep(Duration::from_millis(200)),
            }
        };
        let v = termlink_session::client::unwrap_result(resp).map_err(std::io::Error::other)?;
        v["hub_id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| std::io::Error::other("hub states no hub_id"))
    }

    /// Stop the hub (a hub outage for FC-8); the tree and keys stay.
    pub fn stop_hub(&mut self) {
        if let Some(mut c) = self.hub.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        let _ = fs::remove_file(self.runtime_dir().join("hub.sock"));
    }

    /// Start the hub again on the same runtime dir (same certificate, same id).
    pub fn restart_hub(&mut self) -> std::io::Result<()> {
        self.stop_hub();
        self.start_hub()
    }

    /// The estate's key for `who` ("a" or "b").
    pub fn identity(&self, who: &str) -> std::io::Result<termlink_session::agent_identity::Identity> {
        termlink_session::agent_identity::Identity::load_or_create_from_file(
            &self.root.join("keys").join(format!("{who}.key")),
        )
        .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn start_hub(&mut self) -> std::io::Result<()> {
        let bin = termlink_bin().map_err(std::io::Error::other)?;
        let log = fs::File::create(self.root.join("hub.log"))?;
        // A loopback TCP listener makes the hub create its certificate, and with it
        // the hub id (FX-2): a hub with no TCP listener states no id. Random port.
        let port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        let tcp = format!("127.0.0.1:{port}");
        let child = self
            .command(bin)
            .args(["hub", "start", "--tcp", &tcp])
            .stdin(Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log)
            .spawn()?;
        self.hub = Some(child);
        let sock = self.runtime_dir().join("hub.sock");
        for _ in 0..100 {
            if sock.exists() {
                return Ok(());
            }
            if let Some(c) = self.hub.as_mut()
                && let Some(st) = c.try_wait()?
            {
                return Err(std::io::Error::other(format!(
                    "hub exited early ({st}): {}",
                    fs::read_to_string(self.root.join("hub.log")).unwrap_or_default()
                )));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(std::io::Error::other("hub.sock did not appear"))
    }

    /// Paths held open by the hub that fall under a production location.
    pub fn open_production_paths(&self) -> Vec<PathBuf> {
        let Some(pid) = self.hub_pid() else {
            return vec![];
        };
        let prod = production_paths();
        let mut hits = vec![];
        if let Ok(rd) = fs::read_dir(format!("/proc/{pid}/fd")) {
            for e in rd.flatten() {
                if let Ok(t) = fs::read_link(e.path())
                    && prod.iter().any(|p| t.starts_with(p))
                {
                    hits.push(t);
                }
            }
        }
        hits
    }

    /// Production paths whose contents changed since `create` (mtime/entries).
    pub fn changed_production_paths(&self) -> Vec<PathBuf> {
        self.prod_before
            .iter()
            .filter(|(p, before)| snapshot(p) != *before)
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Path-open audit: empty means the hub holds no production path open.
    /// mtime drift ([`Self::changed_production_paths`]) is reported separately
    /// because other processes on a shared host legitimately touch those dirs.
    pub fn path_open_audit(&self) -> Vec<PathBuf> {
        self.open_production_paths()
    }

    /// SI-25 secret scan: files under `dir` (default: estate) containing the canary.
    pub fn secret_scan(&self, dir: &Path) -> Vec<PathBuf> {
        fn walk(d: &Path, needle: &[u8], out: &mut Vec<PathBuf>) {
            let Ok(rd) = fs::read_dir(d) else { return };
            for e in rd.flatten() {
                let p = e.path();
                let Ok(md) = fs::symlink_metadata(&p) else {
                    continue;
                };
                if md.is_dir() {
                    walk(&p, needle, out);
                } else if md.is_file()
                    && let Ok(b) = fs::read(&p)
                    && b.windows(needle.len()).any(|w| w == needle)
                {
                    out.push(p);
                }
            }
        }
        let mut out = vec![];
        walk(dir, self.canary.as_bytes(), &mut out);
        out
    }

    /// Live processes whose environment names this estate's runtime dir.
    pub fn surviving_processes(&self) -> Vec<u32> {
        let needle = format!("TERMLINK_RUNTIME_DIR={}", self.runtime_dir().display());
        let me = std::process::id();
        let mut v = vec![];
        if let Ok(rd) = fs::read_dir("/proc") {
            for e in rd.flatten() {
                let Some(pid) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else {
                    continue;
                };
                if pid == me {
                    continue;
                }
                if let Ok(env) = fs::read(format!("/proc/{pid}/environ"))
                    && env.split(|b| *b == 0).any(|kv| kv == needle.as_bytes())
                    && !is_zombie(pid)
                {
                    v.push(pid);
                }
            }
        }
        v
    }

    /// Kill the hub, delete the tree; returns processes that survived.
    pub fn teardown(&mut self) -> Vec<u32> {
        if let Some(mut c) = self.hub.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        for _ in 0..20 {
            let s = self.surviving_processes();
            if s.is_empty() {
                break;
            }
            for pid in s {
                unsafe { libc::kill(pid as i32, libc::SIGKILL) };
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let survivors = self.surviving_processes();
        // Only delete a tree that still carries the marker.
        if self.root.join(MARKER).exists() {
            let _ = fs::remove_dir_all(&self.root);
        }
        survivors
    }
}

fn is_zombie(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat"))
        .map(|s| {
            s.rsplit(')')
                .next()
                .is_some_and(|r| r.trim_start().starts_with('Z'))
        })
        .unwrap_or(false)
}

impl Drop for Estate {
    fn drop(&mut self) {
        let survivors = self.teardown();
        if !survivors.is_empty() && !std::thread::panicking() {
            panic!("estate teardown left processes alive: {survivors:?}");
        }
    }
}

/// T-3772: the harness identity of a freshly spawned PTY child, read only once it has
/// exec'd. Before `exec` its exe is the test binary itself, and a registry built from that
/// read never matches the shell the doorbell later sees (a race that shows on a loaded host).
/// Linux only: it reads `/proc`.
#[cfg(target_os = "linux")]
pub fn harness_after_exec(pid: u32) -> crate::doorbell::Harness {
    let me = std::env::current_exe().ok().and_then(|p| p.canonicalize().ok()).map(|p| p.to_string_lossy().into_owned());
    for _ in 0..200 {
        if let Some(h) = crate::doorbell::harness_of(Path::new("/proc"), pid)
            && h.exe.is_some()
            && h.exe != me
        {
            return h;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("pid {pid} never exec'd");
}

#[cfg(test)]
mod tests {
    use super::{newest_file, pick_fresh, time_factor};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    /// T-3855 fixture: a scratch dir with a `src/` tree and two "binaries".
    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("t3855-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src/nested")).unwrap();
        d
    }

    fn touch(p: &std::path::Path, age_secs: u64) {
        std::fs::write(p, b"x").unwrap();
        let t = SystemTime::now() - Duration::from_secs(age_secs);
        std::fs::File::options().write(true).open(p).unwrap().set_modified(t).unwrap();
    }

    /// The newest file is found in a nested directory, not just at the top.
    #[test]
    fn newest_file_walks_directories() {
        let d = scratch("walk");
        touch(&d.join("src/a.rs"), 500);
        touch(&d.join("src/nested/b.rs"), 10);
        let (p, _) = newest_file(&[d.join("src"), d.join("missing")]).unwrap();
        assert_eq!(p, d.join("src/nested/b.rs"));
        assert!(newest_file(&[d.join("missing")]).is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// A binary newer than its source is used.
    #[test]
    fn fresh_binary_is_used() {
        let d = scratch("fresh");
        touch(&d.join("src/a.rs"), 500);
        touch(&d.join("termlink"), 10);
        let got = pick_fresh(&[d.join("termlink")], newest_file(&[d.join("src")])).unwrap();
        assert_eq!(got, d.join("termlink"));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The 2026-11-23 case: a stale first candidate is skipped for a fresh second
    /// one; with only stale ones the error names the binary, the newer source
    /// file and both fixes.
    #[test]
    fn stale_binary_is_skipped_and_named() {
        let d = scratch("stale");
        touch(&d.join("old-termlink"), 900);
        touch(&d.join("src/nested/b.rs"), 500);
        touch(&d.join("new-termlink"), 10);
        let src = newest_file(&[d.join("src")]);
        let got = pick_fresh(&[d.join("old-termlink"), d.join("new-termlink")], src.clone()).unwrap();
        assert_eq!(got, d.join("new-termlink"));
        let err = pick_fresh(&[d.join("old-termlink"), d.join("absent")], src).unwrap_err();
        assert!(err.contains("old-termlink"), "{err}");
        assert!(err.contains("src/nested/b.rs"), "{err}");
        assert!(err.contains("cargo build -p termlink") && err.contains("TERMLINK_BIN"), "{err}");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// No candidate at all, and no readable source: say not found, and with no
    /// source to compare against take the first existing binary.
    #[test]
    fn missing_binary_and_missing_source() {
        let d = scratch("missing");
        let err = pick_fresh(&[d.join("absent")], None).unwrap_err();
        assert!(err.contains("not found") && err.contains("TERMLINK_BIN"), "{err}");
        touch(&d.join("termlink"), 900);
        assert_eq!(pick_fresh(&[d.join("termlink")], None).unwrap(), d.join("termlink"));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// T-3535: the factor parses like the hub's `test_budget` — unset, empty, zero and
    /// garbage all read as 1, so a stray value never shortens a bound.
    #[test]
    fn time_factor_defaults_to_one_and_never_shrinks() {
        assert_eq!(time_factor(None), 1);
        assert_eq!(time_factor(Some("")), 1);
        assert_eq!(time_factor(Some("0")), 1);
        assert_eq!(time_factor(Some("abc")), 1);
        assert_eq!(time_factor(Some(" 3 ")), 3);
    }
}
