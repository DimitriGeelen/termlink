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
    _receiver: Receiver,
    pub api: Api,
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

/// Start every part. `store_root`, `runtime` and `home` come from the estate.
pub fn start(cfg: &Config, store_root: &Path, runtime: &Path, home: &Path) -> Result<Running, String> {
    let roster = roster_of(&cfg.roster)?;
    let identity = Identity::load_or_create_from_file(&cfg.key_file).map_err(|e| format!("key file: {e}"))?;
    let store = Arc::new(Store::open(store_root, &cfg.agent).map_err(|e| e.to_string())?);
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
    let api = Api::start(api_cfg, store.clone()).map_err(|e| format!("api: {e}"))?;
    // Decide attempts whose finalizer never ran or died (B13).
    let sweep_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let sweeper = {
        let (d, stop) = (deliverer.clone(), sweep_stop.clone());
        std::thread::spawn(move || {
            let mut waited = Duration::ZERO;
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(100));
                waited += Duration::from_millis(100);
                if waited >= SWEEP_EVERY {
                    waited = Duration::ZERO;
                    d.sweep(crate::store::now_ms(), SWEEP_WINDOW_MS);
                }
            }
        })
    };
    Ok(Running { store, sender, deliverer, _receiver: receiver, api, sweep_stop, sweeper: Some(sweeper) })
}

/// `termlink-sidecar start --config F`: run until SIGTERM or SIGINT.
pub fn main_start(config: &Path) -> i32 {
    // The guard returns the estate root; it has checked that HOME, the runtime
    // dir and the store root all lie inside it. The store itself is the
    // configured store root, not the estate root.
    if let Err(e) = crate::estate::guard_from_env() {
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
    rt.block_on(async {
        use tokio::signal::unix::{SignalKind, signal};
        let (Ok(mut term), Ok(mut int)) = (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) else {
            return;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = int.recv() => {}
        }
    });
    drop(running);
    println!("termlink-sidecar: {} stopped", cfg.agent);
    0
}
