//! `~/.termlink/hubs.toml` model + its read-only loader (T-3214).
//!
//! Moved here from the CLI's `config.rs` so the doctor's `secret_cache_profiles`
//! check (and any other shared consumer) reads hubs.toml through ONE parser.
//! The CLI re-exports these items; mutation paths (strict loader, save, profile
//! resolution) stay in the CLI.

/// Hub profiles config file (~/.termlink/hubs.toml)
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct HubsConfig {
    #[serde(default)]
    pub hubs: std::collections::HashMap<String, HubEntry>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct HubEntry {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// T-1291: declared out-of-band trust anchor for `fleet reauth --bootstrap-from auto`.
    /// Same scheme vocabulary as T-1055 (`file:<path>`, `ssh:<host>`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_from: Option<String>,
}

pub fn hubs_config_path() -> std::path::PathBuf {
    termlink_config_dir().join("hubs.toml")
}

/// Pure resolution core — no env reads, no filesystem side effects — so the
/// policy is deterministically unit-testable without touching process globals
/// (mirrors `termlink_session::identity_dir::resolve_identity_dir_from`, T-2607).
///
/// Returns the resolved config DIRECTORY (the one holding `hubs.toml`) plus
/// `last_resort = true` when `HOME` was unset/empty and it fell through to the
/// UID-namespaced private tmp path. An exported-but-empty `HOME=` is treated as
/// unset, never as the root directory `/`.
///
/// NOTE (T-2629): deliberately does NOT add `XDG_STATE_HOME` / `TERMLINK_IDENTITY_DIR`
/// precedence the way the identity plane does — that would silently relocate an
/// operator's existing `~/.termlink/hubs.toml` when either var is set (a "all my
/// hubs vanished" regression). This fix is scoped to closing ONLY the shared-/tmp
/// leak; HOME-set resolution is behavior-preserving.
pub fn resolve_config_dir_from(
    home: Option<&str>,
    temp_dir: &std::path::Path,
    uid: u32,
) -> (std::path::PathBuf, bool) {
    if let Some(h) = home.filter(|s| !s.is_empty()) {
        // Behavior-preserving: the pre-T-2629 `$HOME/.termlink` shape.
        return (std::path::PathBuf::from(h).join(".termlink"), false);
    }
    // HOME unset/empty: a UID-namespaced private dir under the temp base — never
    // the shared, world-writable `/tmp/.termlink` that would leak hub profiles +
    // bootstrap anchors to any local user. Mirrors identity_dir's last-resort.
    (temp_dir.join(format!("termlink-{uid}")), true)
}

pub fn termlink_config_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME").ok();
    let temp = std::env::temp_dir();
    let uid = unsafe { libc::getuid() };
    let (dir, last_resort) = resolve_config_dir_from(home.as_deref(), &temp, uid);
    if last_resort {
        warn_config_last_resort_once(&dir);
        harden_config_last_resort_dir(&dir);
    }
    dir
}

static CONFIG_LAST_RESORT_WARNED: std::sync::Once = std::sync::Once::new();

fn warn_config_last_resort_once(dir: &std::path::Path) {
    CONFIG_LAST_RESORT_WARNED.call_once(|| {
        tracing::error!(
            config_dir = %dir.display(),
            "HOME is unset — hub profile config (hubs.toml + bootstrap anchors) is \
             falling back to a UID-namespaced temp directory; it will NOT persist \
             across reboot and fleet discovery will see no configured hubs. Set HOME \
             to a persistent private directory."
        );
    });
}

/// Create the last-resort directory with mode `0700` so hub profiles + bootstrap
/// trust anchors are never left world-readable/writable. Best-effort: a creation
/// failure surfaces later as an ordinary file-open error (still loud) — the point
/// is only that we never silently produce a world-writable config store.
fn harden_config_last_resort_dir(dir: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    if std::fs::create_dir_all(dir).is_ok() {
        let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
    }
}

/// Lenient loader for READ-ONLY callers (fleet doctor/verify/status,
/// address resolution, ...). A missing file is a normal empty config;
/// a file that exists but fails to parse degrades to empty LOUDLY
/// (T-2437) — one stderr line naming the path and the toml error —
/// so "why did every hub vanish?" is answered on the spot instead of
/// silently returning zero hubs. Mutation paths (profile add/remove)
/// MUST use [`load_hubs_config_strict`] instead: saving a leniently
/// defaulted config would wipe the operator's file.
pub fn load_hubs_config() -> HubsConfig {
    let path = hubs_config_path();
    if let Ok(content) = std::fs::read_to_string(&path) {
        match toml::from_str(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "warning: {} is unparseable ({e}); treating hub config as EMPTY — fix the file to restore your hub profiles",
                    path.display()
                );
                HubsConfig::default()
            }
        }
    } else {
        HubsConfig::default()
    }
}
