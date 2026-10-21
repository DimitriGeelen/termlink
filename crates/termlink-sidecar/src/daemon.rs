//! The sidecar process started by `termlink-sidecar start`, which opens the store, send pump, receive loop, deliverer and API sockets.
//! The sidecar process: `termlink-sidecar start --config <file>` (B10).
//!
//! One process per agent, inside the estate guard (C-1): it opens the store
//! (holding its lock, SI-16), the send pump (C-5), the receive loop (C-6), the
//! deliverer (C-9) and the three API sockets (C-4), then waits for SIGTERM or
//! SIGINT. The config names files, never secrets: the signing key is read from
//! `key_file`, the roster is a list of public keys.

use crate::api::{Api, ApiConfig};
use crate::deliver::Deliverer;
use crate::policy::BuiltIn;
use crate::pull::{ReceiveConfig, Receiver};
use crate::send::{SendConfig, Sender};
use crate::store::Store;
use crate::wire::Roster;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use termlink_protocol::TransportAddr;
use termlink_session::agent_identity::Identity;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub agent: String,
    pub instance: String,
    pub hub: TransportAddr,
    pub hub_id: String,
    /// This agent's signing key file.
    pub key_file: PathBuf,
    /// Public keys (64 hex) of the senders this agent accepts: the PH-1 roster
    /// and allow-list.
    pub roster: Vec<String>,
    pub queue_path: PathBuf,
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    /// Where the agent's Claude Code writes its transcripts. Set when the agent
    /// runs under another HOME than the sidecar (B14: the agent is `testdev` with
    /// its own login); default `$HOME/.claude/projects` of the sidecar.
    #[serde(default)]
    pub claude_projects: Option<PathBuf>,
    /// The operator's key register entry for this agent: a file holding its
    /// public key (64 hex) (P7-2, PR-1a). Required in production, where a
    /// missing key file or a different key is a refused start.
    #[serde(default)]
    pub key_register: Option<PathBuf>,
    /// The project's address book and pinned main (P7; SI-20, SI-21, R-62).
    /// Required in production.
    #[serde(default)]
    pub book: crate::book::Book,
}

fn default_poll_ms() -> u64 {
    500
}
fn default_timeout_ms() -> u64 {
    10_000
}

/// A running sidecar; dropping it stops every part.
pub struct Running {
    pub store: Arc<Store>,
    pub sender: Arc<Sender>,
    pub deliverer: Arc<Deliverer>,
    _receiver: Arc<Receiver>,
    pub api: Api,
    receiver_beat: Box<dyn Fn() -> u64 + Send + Sync>,
    tick_beat: Arc<std::sync::atomic::AtomicU64>,
    sweep_stop: Arc<std::sync::atomic::AtomicBool>,
    sweeper: Option<std::thread::JoinHandle<()>>,
}

/// How often open attempts are swept, and how old one must be for the final check.
pub const SWEEP_EVERY: Duration = Duration::from_secs(2);
pub const SWEEP_WINDOW_MS: u64 = 60_000;

impl Drop for Running {
    fn drop(&mut self) {
        self.sweep_stop.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(t) = self.sweeper.take() {
            let _ = t.join();
        }
    }
}

fn roster_of(keys: &[String]) -> Result<Roster, String> {
    let mut r = Roster::new();
    for k in keys {
        let bytes: Vec<u8> = (0..k.len())
            .step_by(2)
            .map(|i| k.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("roster key is not hex: {k}"))?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| format!("roster key is not 32 bytes: {k}"))?;
        r.add(VerifyingKey::from_bytes(&arr).map_err(|e| format!("roster key: {e}"))?);
    }
    Ok(r)
}

/// The agent's signing identity (P7-2). In production the key file must exist
/// (a missing file is never silently replaced by a fresh key) and its public key
/// must equal the operator's register entry; in a test estate a register entry,
/// when given, is checked the same way.
fn load_identity(cfg: &Config, production: bool) -> Result<Identity, String> {
    if production && cfg.key_register.is_none() {
        return Err("production needs key_register (the operator's record of this agent's public key)".into());
    }
    let identity = if production {
        if !cfg.key_file.is_file() {
            return Err(format!(
                "key file {} is missing; production never mints a key (re-provision through runme)",
                cfg.key_file.display()
            ));
        }
        Identity::load_or_create_from_file(&cfg.key_file).map_err(|e| format!("key file: {e}"))?
    } else {
        Identity::load_or_create_from_file(&cfg.key_file).map_err(|e| format!("key file: {e}"))?
    };
    if let Some(reg) = &cfg.key_register {
        let want = std::fs::read_to_string(reg).map_err(|e| format!("key register {}: {e}", reg.display()))?;
        let want = want.trim();
        if want != identity.public_key_hex() {
            return Err(format!(
                "key file {} does not match the key register {} (registered {}, file has {}); a key changes only through a runme action that names the change",
                cfg.key_file.display(),
                reg.display(),
                &want.get(..16).unwrap_or(want),
                &identity.public_key_hex()[..16]
            ));
        }
    }
    Ok(identity)
}

/// Start every part. `store_root`, `runtime` and `home` come from the estate.
pub fn start(cfg: &Config, store_root: &Path, runtime: &Path, home: &Path) -> Result<Running, String> {
    let roster = roster_of(&cfg.roster)?;
    let store = Arc::new(Store::open(store_root, &cfg.agent).map_err(|e| e.to_string())?);
    let production = matches!(store.mode(), crate::estate::Mode::Production(_));
    // SI-27: hash the running binary in the background; status says "verifying" until it lands.
    crate::tools::start_binary_hash();
    let identity = load_identity(cfg, production)?;
    if production && cfg.book.is_empty() {
        return Err("production needs an address book (book.peers) naming this project's agents".into());
    }
    cfg.book.validate(&cfg.agent).map_err(|e| format!("address book: {e}"))?;
    let timeout = Duration::from_millis(cfg.timeout_ms);
    let poll = Some(Duration::from_millis(cfg.poll_ms));
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: cfg.agent.clone(),
                instance: cfg.instance.clone(),
                identity,
                hub: cfg.hub.clone(),
                declared_hub_id: cfg.hub_id.clone(),
                queue_path: cfg.queue_path.clone(),
                timeout,
                pump_interval: poll,
            },
            store.clone(),
        )
        .map_err(|e| e.to_string())?,
    );
    let receiver = Receiver::new(
        ReceiveConfig {
            agent: cfg.agent.clone(),
            instance: cfg.instance.clone(),
            hub: cfg.hub.clone(),
            declared_hub_id: cfg.hub_id.clone(),
            roster: roster.clone(),
            timeout,
            page: 100,
            poll_interval: poll,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster.clone()).with_operator_lists(store.clone())),
    )
    .map_err(|e| e.to_string())?;
    let deliverer = Arc::new(
        Deliverer::new(&cfg.agent, store.clone(), Box::new(BuiltIn::new(roster).with_operator_lists(store.clone())), Some(sender.clone()))
            .with_projects_root(cfg.claude_projects.clone().unwrap_or_else(|| home.join(".claude").join("projects"))),
    );
    let mut api_cfg = ApiConfig::new(&cfg.agent, runtime);
    api_cfg.hub_id = Some(cfg.hub_id.clone());
    api_cfg.sender = Some(sender.clone());
    api_cfg.deliverer = Some(deliverer.clone());
    api_cfg.book = (!cfg.book.is_empty()).then(|| cfg.book.clone());
    let api = Api::start(api_cfg, store.clone()).map_err(|e| format!("api: {e}"))?;
    // Decide attempts whose finalizer never ran or died (B13).
    let sweep_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let tick_beat = Arc::new(std::sync::atomic::AtomicU64::new(crate::store::now_ms()));
    let sweeper = {
        let (d, stop, beat) = (deliverer.clone(), sweep_stop.clone(), tick_beat.clone());
        std::thread::spawn(move || {
            let mut waited = Duration::ZERO;
            let mut since_tick = Duration::ZERO;
            let tick = Duration::from_millis(crate::deadline::TICK_MS);
            let deadlines = crate::deadline::Deadlines::R44E;
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                beat.store(crate::store::now_ms(), std::sync::atomic::Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(100));
                waited += Duration::from_millis(100);
                since_tick += Duration::from_millis(100);
                if waited >= SWEEP_EVERY {
                    waited = Duration::ZERO;
                    d.sweep(crate::store::now_ms(), SWEEP_WINDOW_MS);
                }
                // Once per tick (R-17): the receiver's overdue notices (R-44.e).
                if since_tick >= tick {
                    since_tick = Duration::ZERO;
                    d.check_overdue(crate::store::now_ms(), &deadlines);
                }
            }
        })
    };
    let receiver = Arc::new(receiver);
    let receiver_beat: Box<dyn Fn() -> u64 + Send + Sync> = {
        let r = receiver.clone();
        Box::new(move || r.last_beat_ms())
    };
    Ok(Running { store, sender, deliverer, _receiver: receiver, api, receiver_beat, tick_beat, sweep_stop, sweeper: Some(sweeper) })
}

impl Running {
    /// Last beat of every component (K-3): send pump, receive loop, tick, API scopes.
    pub fn beats(&self) -> Vec<(String, u64)> {
        let mut v = vec![
            ("send".to_string(), self.sender.last_beat_ms()),
            ("receive".to_string(), (self.receiver_beat)()),
            ("tick".to_string(), self.tick_beat.load(std::sync::atomic::Ordering::SeqCst)),
        ];
        v.extend(self.api.beats());
        v
    }

    /// Write `heartbeat.json` for `now`; returns the components silent past `after`.
    pub fn heartbeat(&self, now: u64, after: u64) -> std::io::Result<Vec<String>> {
        self.heartbeat_with(now, after, None)
    }

    /// As [`Running::heartbeat`], reporting `frozen` (a test estate's fault seam)
    /// as never having beaten.
    pub fn heartbeat_with(&self, now: u64, after: u64, frozen: Option<&str>) -> std::io::Result<Vec<String>> {
        let mut beats = self.beats();
        if let Some(f) = frozen {
            for (n, t) in beats.iter_mut() {
                if n == f {
                    *t = 0;
                }
            }
        }
        crate::beat::write(self.store.dir(), now, &beats, after)
    }
}

/// `termlink-sidecar start --config F`: run until SIGTERM or SIGINT.
pub fn main_start(config: &Path) -> i32 {
    // The guard returns the estate root; it has checked that HOME, the runtime
    // dir and the store root all lie inside it. The store itself is the
    // configured store root, not the estate root.
    if let Err(e) = crate::estate::guard_any_from_env() {
        eprintln!("termlink-sidecar: refusing to start: {e}");
        return 2;
    }
    let env_path = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let (Some(store_root), Some(runtime), Some(home)) =
        (env_path(crate::estate::STORE_ENV), env_path("TERMLINK_RUNTIME_DIR"), env_path("HOME"))
    else {
        eprintln!("termlink-sidecar: {}, TERMLINK_RUNTIME_DIR and HOME must be set", crate::estate::STORE_ENV);
        return 2;
    };
    let cfg: Config = match std::fs::read_to_string(config).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("termlink-sidecar: config {}: {e}", config.display());
            return 2;
        }
    };
    let running = match start(&cfg, &store_root, &runtime, &home) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("termlink-sidecar: cannot start: {e}");
            return 1;
        }
    };
    println!("termlink-sidecar: {} running; send socket {}", cfg.agent, running.api.path(crate::api::Scope::Send).display());
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("termlink-sidecar: runtime: {e}");
            return 1;
        }
    };
    // Once per tick: the heartbeat file, and exit when a component went silent
    // (FP-1 failure 4) so the supervisor restarts the whole sidecar (R-8).
    let dead: Option<Vec<String>> = rt.block_on(async {
        use tokio::signal::unix::{SignalKind, signal};
        let (Ok(mut term), Ok(mut int)) = (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) else {
            return None;
        };
        let (every, after, frozen) = crate::beat::test_overrides(running.store.mode());
        let mut tick = tokio::time::interval(Duration::from_millis(every));
        loop {
            tokio::select! {
                _ = term.recv() => return None,
                _ = int.recv() => return None,
                _ = tick.tick() => match running.heartbeat_with(crate::store::now_ms(), after, frozen.as_deref()) {
                    Ok(dead) if !dead.is_empty() => return Some(dead),
                    Ok(_) => {}
                    Err(e) => eprintln!("termlink-sidecar: heartbeat not written: {e}"),
                },
            }
        }
    });
    if let Some(dead) = dead {
        let why = format!("component(s) silent past {} ms: {}", crate::beat::test_overrides(running.store.mode()).1, dead.join(", "));
        let _ = running.store.attention("component-dead", &why);
        eprintln!("termlink-sidecar: {} exiting for restart: {why}", cfg.agent);
        // A hung loop may never join; leave without dropping the parts.
        std::process::exit(3);
    }
    drop(running);
    println!("termlink-sidecar: {} stopped", cfg.agent);
    0
}
