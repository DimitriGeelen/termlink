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

/// Locate the `termlink` binary used for the throwaway hub.
pub fn termlink_bin() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("TERMLINK_BIN") {
        return Some(PathBuf::from(p));
    }
    let ws = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    ["debug", "release"]
        .iter()
        .map(|d| ws.join("target").join(d).join("termlink"))
        .filter(|p| p.is_file())
        .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
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
    /// asked again, for up to 30 s in all.
    pub fn hub_id(&self) -> std::io::Result<String> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
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
        let bin = termlink_bin()
            .ok_or_else(|| std::io::Error::other("termlink binary not found (set TERMLINK_BIN)"))?;
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
