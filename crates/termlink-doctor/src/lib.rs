//! TermLink doctor — the ONE environment health-check list (T-3214).
//!
//! Before T-3214 the CLI `termlink doctor` and the MCP `termlink_doctor` each
//! hand-maintained their own check list, and they drifted: MCP lacked
//! `ufw_listener`, `inbox`, `secret_cache` and `secret_cache_profiles`, and the
//! two disagreed on `ok` under strict. Operator ruling SQ-8 (option 1b): the
//! checks live in this neutral crate, which both surfaces depend on, so parity
//! is a property of the code rather than of discipline.
//!
//! [`run_doctor`] collects the checks (and performs `--fix` side effects when
//! asked). Callers own presentation only: the CLI renders text or JSON and maps
//! `ok` to its exit code; MCP returns [`DoctorReport::to_json`].

use std::path::PathBuf;

use serde_json::json;

pub mod hubs_config;
pub mod manifest;

/// Outcome of one check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            CheckStatus::Pass => "pass",
            CheckStatus::Warn => "warn",
            CheckStatus::Fail => "fail",
        }
    }
}

/// One emitted check line.
#[derive(Debug, Clone)]
pub struct Check {
    pub check: &'static str,
    pub status: CheckStatus,
    pub message: String,
    /// Follow-up lines for the CLI's text mode only (stale session names, `--fix`
    /// outcomes, hints). Never part of the JSON envelope.
    pub notes: Vec<String>,
}

/// Caller-supplied inputs. Everything else is read from the environment.
#[derive(Debug, Clone)]
pub struct DoctorOptions {
    /// Apply the safe auto-remediations (`termlink doctor --fix`). MCP never sets it.
    pub fix: bool,
    /// Warnings count against `ok` (CLI `--strict`, MCP `strict: true`).
    pub strict: bool,
    /// Project root whose `.termlink/dispatch-manifest.json` is inspected.
    pub project_root: PathBuf,
    /// Message for the trailing `version` check. Each surface derives its own
    /// version in its build script, so the caller supplies the line.
    pub version_message: String,
}

/// The full doctor result.
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub checks: Vec<Check>,
    pub pass: u32,
    pub warn: u32,
    pub fail: u32,
    pub strict: bool,
}

impl DoctorReport {
    /// T-1712 rollup: no failures, and under strict no warnings either. The CLI's
    /// exit code is `!ok`, so the envelope and the exit status always agree.
    pub fn ok(&self) -> bool {
        self.fail == 0 && !(self.strict && self.warn > 0)
    }

    /// The `{ok, strict, checks, summary}` envelope both surfaces emit.
    pub fn to_json(&self) -> serde_json::Value {
        let checks: Vec<serde_json::Value> = self
            .checks
            .iter()
            .map(|c| json!({"check": c.check, "status": c.status.as_str(), "message": c.message}))
            .collect();
        json!({
            "ok": self.ok(),
            "strict": self.strict,
            "checks": checks,
            "summary": {
                "pass": self.pass,
                "warn": self.warn,
                "fail": self.fail,
            }
        })
    }

    fn push(&mut self, check: &'static str, status: CheckStatus, message: impl Into<String>) {
        match status {
            CheckStatus::Pass => self.pass += 1,
            CheckStatus::Warn => self.warn += 1,
            CheckStatus::Fail => self.fail += 1,
        }
        self.checks.push(Check { check, status, message: message.into(), notes: Vec::new() });
    }

    fn note(&mut self, line: impl Into<String>) {
        if let Some(last) = self.checks.last_mut() {
            last.notes.push(line.into());
        }
    }
}

/// Run every doctor check in the canonical order. With `opts.fix` the safe
/// remediations run too (stale sessions, stale hub pidfile, orphaned sockets,
/// >24h pending dispatches, secret-cache chmod 600).
pub async fn run_doctor(opts: &DoctorOptions) -> DoctorReport {
    use termlink_session::{client, discovery, liveness, manager};
    use CheckStatus::{Fail, Pass, Warn};

    let fix = opts.fix;
    let mut r = DoctorReport { checks: Vec::new(), pass: 0, warn: 0, fail: 0, strict: opts.strict };

    // 1. Runtime directory
    let runtime_dir = discovery::runtime_dir();
    if runtime_dir.exists() {
        r.push("runtime_dir", Pass, format!("{}", runtime_dir.display()));
    } else {
        r.push("runtime_dir", Fail, format!("{} does not exist", runtime_dir.display()));
    }

    // 2. Sessions directory
    let sessions_dir = discovery::sessions_dir();
    if sessions_dir.exists() {
        r.push("sessions_dir", Pass, format!("{}", sessions_dir.display()));
    } else {
        r.push("sessions_dir", Warn, format!("{} does not exist (no sessions registered yet)", sessions_dir.display()));
    }

    // 3. Session health
    let sessions = manager::list_sessions(true).unwrap_or_default();
    let total = sessions.len();
    let mut alive = 0u32;
    let mut dead = 0u32;
    let mut stale_sockets: Vec<String> = Vec::new();

    let ping_timeout = std::time::Duration::from_secs(3);
    for s in &sessions {
        if liveness::process_exists(s.pid) {
            // Try actual ping with timeout to avoid hanging on dead sockets
            let rpc_future = client::rpc_call(s.socket_path(), "termlink.ping", json!({}));
            match tokio::time::timeout(ping_timeout, rpc_future).await {
                Ok(Ok(_)) => alive += 1,
                Ok(Err(_)) | Err(_) => {
                    dead += 1;
                    stale_sockets.push(s.display_name.clone());
                }
            }
        } else {
            dead += 1;
            stale_sockets.push(s.display_name.clone());
        }
    }

    if total == 0 {
        r.push("sessions", Pass, "no sessions registered");
    } else if dead == 0 {
        r.push("sessions", Pass, format!("{total} registered, all responding"));
    } else {
        r.push("sessions", Warn, format!("{total} registered, {alive} alive, {dead} dead/stale"));
        for name in &stale_sockets {
            r.note(format!("stale: {name}"));
        }
        if fix {
            let cleaned = manager::clean_stale_sessions(&sessions_dir, true).unwrap_or_default();
            if !cleaned.is_empty() {
                r.note(format!("\x1b[32mfixed:\x1b[0m removed {} stale session(s)", cleaned.len()));
            }
        } else if !stale_sockets.is_empty() {
            r.note("Run 'termlink doctor --fix' to auto-clean");
        }
    }

    // 4. Hub status — T-1030/T-1031: use resolve_hub_paths() to find the hub
    //    regardless of whether it's in default runtime_dir or /var/lib/termlink.
    let (pidfile_path, hub_socket) = resolve_hub_paths();
    let alt_dir = pidfile_path.parent() != Some(discovery::runtime_dir().as_path());
    let suffix = if alt_dir {
        format!(" (via {})", pidfile_path.parent().map(|p| p.display().to_string()).unwrap_or_default())
    } else {
        String::new()
    };
    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            let hub_rpc = client::rpc_call(&hub_socket, "termlink.ping", json!({}));
            match tokio::time::timeout(ping_timeout, hub_rpc).await {
                Ok(Ok(_)) => r.push("hub", Pass, format!("running (PID {pid}), responding{suffix}")),
                Ok(Err(_)) | Err(_) => r.push("hub", Warn, format!("running (PID {pid}), but not responding on socket{suffix}")),
            }
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            if fix {
                termlink_hub::pidfile::remove(&pidfile_path);
                let _ = std::fs::remove_file(&hub_socket);
                r.push("hub", Warn, format!("stale pidfile (PID {pid}) — fixed: removed pidfile and socket{suffix}"));
            } else {
                r.push("hub", Warn, format!("stale pidfile (PID {pid} is dead). Run 'termlink doctor --fix' to clean up"));
            }
        }
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            r.push("hub", Pass, "not running (optional — needed for multi-session routing)");
        }
    }

    // 4b. UFW rule vs. TCP listener consistency (T-934)
    //
    // Catches the exact state that triggered T-930: a firewall rule on
    // the hub's TCP port exists but nothing is bound to it, so cross-host
    // callers get connection-refused with no actionable signal. Parse
    // `ufw status` for any rule whose comment mentions "termlink" (case-
    // insensitive) and extract the port. If no corresponding listener is
    // found via `ss -tln`, emit a warn check.
    //
    // The whole block is best-effort: ufw unavailable or unreadable is a
    // skipped check, not a warning. That keeps the signal high for
    // environments actually using ufw + termlink together.
    if let Some((status, message)) = ufw_listener_check() {
        r.push("ufw_listener", status, message);
    }

    // 5. Orphaned sockets (sockets without matching registration JSON)
    if sessions_dir.exists() {
        let mut orphan_count = 0u32;
        let mut orphan_paths: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && ext == "sock"
                {
                    let json_path = path.with_extension("json");
                    if !json_path.exists() {
                        orphan_count += 1;
                        orphan_paths.push(path);
                    }
                }
            }
        }
        if orphan_count > 0 {
            if fix {
                for p in &orphan_paths {
                    let _ = std::fs::remove_file(p);
                    // Also remove .sock.data if it exists
                    let data_path = p.with_extension("sock.data");
                    let _ = std::fs::remove_file(&data_path);
                }
                r.push("sockets", Warn, format!("{orphan_count} orphaned socket(s) — fixed: removed"));
            } else {
                r.push("sockets", Warn, format!("{orphan_count} orphaned socket(s) without registration"));
            }
        } else {
            r.push("sockets", Pass, "no orphaned sockets");
        }
    }

    // 6. Dispatch manifest
    {
        let project_root = &opts.project_root;
        match manifest::DispatchManifest::load(project_root) {
            Ok(m) => {
                let pending = m.pending_dispatches();
                if pending.is_empty() {
                    if m.dispatches.is_empty() {
                        r.push("dispatch", Pass, "no dispatch manifest");
                    } else {
                        r.push("dispatch", Pass, format!("{} dispatch(es), none pending", m.dispatches.len()));
                    }
                } else {
                    let ids: Vec<&str> = pending.iter().map(|d| d.id.as_str()).collect();
                    r.push("dispatch", Warn, format!("{} pending dispatch(es): {}", pending.len(), ids.join(", ")));
                    if fix {
                        // Expire dispatches older than 24 hours
                        let mut m = m;
                        let cutoff_secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .saturating_sub(86400);
                        let cutoff = manifest::secs_to_rfc3339(cutoff_secs);
                        let mut expired_count = 0u32;
                        for d in &mut m.dispatches {
                            if d.status == manifest::DispatchStatus::Pending && d.created_at < cutoff {
                                d.status = manifest::DispatchStatus::Expired;
                                expired_count += 1;
                            }
                        }
                        if expired_count > 0 {
                            if let Err(e) = m.save(project_root) {
                                r.note(format!("\x1b[31merror:\x1b[0m failed to save manifest: {e}"));
                            } else {
                                r.note(format!("\x1b[32mfixed:\x1b[0m expired {expired_count} dispatch(es) older than 24h"));
                            }
                        } else {
                            r.note("no dispatches old enough to auto-expire (< 24h)");
                        }
                    } else {
                        r.note("Run 'termlink doctor --fix' to expire stale dispatches");
                        r.note("Or  'termlink dispatch-status' for details");
                    }
                }
            }
            Err(e) => {
                r.push("dispatch", Warn, format!("failed to read manifest: {e}"));
            }
        }
    }

    // 7. Inbox status (T-1001 / T-1400 / T-1415).
    //
    // Probes inbox state via `channel.list(prefix="inbox:")`. The legacy
    // `inbox.status` RPC was retired in T-1166 / T-1415 — its hub-side
    // handler no longer exists. channel.list is the only load-bearing path.
    if hub_socket.exists() {
        let outcome: Result<(u64, usize, Vec<String>), String> = async {
            let resp = client::rpc_call(&hub_socket, "channel.list", json!({"prefix": "inbox:"}))
                .await
                .map_err(|e| format!("channel.list transport: {e}"))?;
            let result = client::unwrap_result(resp)?;
            let topics = result["topics"].as_array().cloned().unwrap_or_default();
            let target_count = topics.len();
            // T-2623: helper tracks unreadable counts instead of dropping them,
            // so the caller can downgrade to `warn` rather than silent-pass.
            let (total, dropped) = sum_inbox_counts(&topics);
            Ok::<(u64, usize, Vec<String>), String>((total, target_count, dropped))
        }
        .await;

        match outcome {
            // T-2623: an unreadable count is NOT a clean inbox — surface it
            // loudly and name the offending topics rather than reporting green.
            Ok((total, targets, dropped)) if !dropped.is_empty() => r.push(
                "inbox",
                Warn,
                format!(
                    "{n} inbox topic(s) with unreadable count(s): {names} \
                     ({total} readable pending across {targets} target(s))",
                    n = dropped.len(),
                    names = dropped.join(", "),
                ),
            ),
            Ok((0, _, _)) => r.push("inbox", Pass, "no records on any inbox: topic"),
            // T-3197: record count, not transfers — inbox: topics also carry
            // ordinary messages. Informational, so it PASSES (T-2818 alarm fatigue).
            Ok((total, targets, _)) => r.push(
                "inbox",
                Pass,
                format!(
                    "{total} record(s) across {targets} inbox topic(s) — not a transfer count; \
                     see `termlink inbox list <target>`"
                ),
            ),
            Err(e) => r.push("inbox", Warn, format!("inbox query failed: {e}")),
        }
    }

    // 7b. T-1171 / G-011: Client-side secret cache audit.
    //
    //     ~/.termlink/secrets/<host>.hex is a cache of the shared hub.secret.
    //     When the hub restarts with a new secret (or the runtime_dir migrates)
    //     the cache silently diverges and the next auth fails. Two signals:
    //       (a) perms must be 0600;
    //       (b) if this host runs a local hub, any cache file older than the
    //           live hub.secret is a drift candidate (surfaced, never auto-healed).
    {
        // T-2630: resolve via the single hardened resolver (T-2629).
        let secrets_dir = secret_cache_dir();
        if !secrets_dir.exists() {
            r.push("secret_cache", Pass, "no cached secrets");
        } else {
            let local_hub_secret: Option<(PathBuf, std::time::SystemTime)> =
                pidfile_path.parent().and_then(|dir| {
                    let p = dir.join("hub.secret");
                    std::fs::metadata(&p).ok().and_then(|m| m.modified().ok()).map(|t| (p, t))
                });
            let issues = audit_secret_cache(
                &secrets_dir,
                local_hub_secret.as_ref().map(|(p, t)| (p.as_path(), *t)),
                fix,
            );
            if issues.is_empty() {
                r.push("secret_cache", Pass, "all cached secrets look healthy");
            } else {
                for msg in issues {
                    // T-1654: "fixed:" messages are successful auto-remediation
                    // (--fix chmod 600) and render pass-class.
                    let status = if msg.starts_with("fixed:") { Pass } else { Warn };
                    r.push("secret_cache", status, msg);
                }
            }
        }

        // T-1284 / G-011: profile audit — purely lexical, runs even when
        // secrets_dir doesn't exist. Flags self-hub profiles using the
        // IP-keyed cache as secret_file.
        let hubs_config = hubs_config::load_hubs_config();
        let profile_hints = audit_hubs_for_self_hub_cache(&hubs_config, &secrets_dir);
        if profile_hints.is_empty() {
            r.push("secret_cache_profiles", Pass, "no self-hub profiles using IP-keyed cache");
        } else {
            for hint in profile_hints {
                r.push("secret_cache_profiles", Warn, hint);
            }
        }
    }

    // 7c. Identity attribution — T-1705 (G-056 follow-up). Groups live sessions
    // by identity_fingerprint and surfaces the shared-host case (PL-166).
    // Pre-T-1436 sessions (no identity_fingerprint) are excluded from grouping.
    {
        let groups = group_sessions_by_identity(&sessions);
        let shared: Vec<_> = groups.iter().filter(|(_, names)| names.len() >= 2).collect();
        if shared.is_empty() {
            let with_fp: usize = groups.iter().map(|(_, n)| n.len()).sum();
            r.push("identity", Pass, format!("no shared identities ({with_fp} session(s) with FP)"));
        } else {
            let total_shared: usize = shared.iter().map(|(_, n)| n.len()).sum();
            let groups_desc: Vec<_> = shared
                .iter()
                .map(|(fp, names)| {
                    let short_fp = &fp[..8.min(fp.len())];
                    format!("{}×{}", short_fp, names.len())
                })
                .collect();
            r.push(
                "identity",
                Warn,
                format!(
                    "{total_shared} sessions share {} identity FP [{}] — pass --identity-key at register for per-agent identity (T-1700)",
                    shared.len(),
                    groups_desc.join(", ")
                ),
            );
        }
    }

    // 8. Version
    r.push("version", Pass, opts.version_message.clone());

    r
}

/// T-934: `ufw status` rules naming termlink vs. `ss -tln` listeners. `None`
/// when ufw is unavailable, fails, or carries no termlink rule (check skipped).
fn ufw_listener_check() -> Option<(CheckStatus, String)> {
    let out = std::process::Command::new("ufw").arg("status").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut termlink_ports: Vec<u16> = Vec::new();
    for line in stdout.lines() {
        if !line.to_lowercase().contains("termlink") {
            continue;
        }
        // Format: "9100/tcp  ALLOW  192.168.10.0/24  # TermLink..."
        if let Some(first) = line.split_whitespace().next()
            && let Some(port_str) = first.strip_suffix("/tcp")
            && let Ok(port) = port_str.parse::<u16>()
        {
            termlink_ports.push(port);
        }
    }
    if termlink_ports.is_empty() {
        return None;
    }
    let listening = std::process::Command::new("ss")
        .args(["-tln"])
        .output()
        .ok()
        .and_then(|o| o.status.success().then(|| String::from_utf8_lossy(&o.stdout).to_string()))
        .unwrap_or_default();
    let missing: Vec<u16> = termlink_ports
        .iter()
        .copied()
        .filter(|port| !listening.contains(&format!(":{port} ")))
        .collect();
    let fmt = |ports: &[u16]| ports.iter().map(|p| format!("{p}/tcp")).collect::<Vec<_>>().join(", ");
    if missing.is_empty() {
        Some((CheckStatus::Pass, format!("ufw allows {} — listener present", fmt(&termlink_ports))))
    } else {
        Some((
            CheckStatus::Warn,
            format!(
                "ufw allows {} but nothing is listening — run 'termlink hub start --tcp 0.0.0.0:{}' or start termlink-hub.service",
                fmt(&missing),
                missing[0]
            ),
        ))
    }
}

/// T-1031: Resolve the hub pidfile path, checking the default runtime dir first
/// and falling back to /var/lib/termlink/ (systemd-managed hubs).
/// Returns (pidfile_path, socket_path) from whichever dir has a running hub.
pub fn resolve_hub_paths() -> (PathBuf, PathBuf) {
    let default_pidfile = termlink_hub::pidfile::hub_pidfile_path();
    let default_socket = termlink_hub::server::hub_socket_path();
    // Fallback: /var/lib/termlink/ (systemd-managed hubs). Only consulted if
    // TERMLINK_RUNTIME_DIR is not explicitly set (tests set it to isolated temp
    // dirs and should never discover the real hub).
    let alt_dir = if std::env::var("TERMLINK_RUNTIME_DIR").is_err() {
        Some(PathBuf::from("/var/lib/termlink"))
    } else {
        None
    };
    resolve_hub_paths_in(default_pidfile, default_socket, alt_dir.as_deref())
}

/// T-3340: the pure core of `resolve_hub_paths`, with the alternate dir injectable.
/// Order: a RUNNING default hub; then a RUNNING alternate hub; then a stale default
/// pidfile (so `hub stop`/`restart` still find and clean it); then an alternate
/// pidfile that merely exists (pre-T-3340 behaviour); else the defaults.
/// Before T-3340 a STALE default pidfile won over a live /var/lib/termlink hub, so
/// once a /tmp/termlink-$UID hub had ever existed, every client without the env
/// var latched onto the dead or second hub.
pub fn resolve_hub_paths_in(
    default_pidfile: PathBuf,
    default_socket: PathBuf,
    alt_dir: Option<&std::path::Path>,
) -> (PathBuf, PathBuf) {
    use termlink_hub::pidfile::{check, PidfileStatus};
    let default_status = check(&default_pidfile);
    if matches!(default_status, PidfileStatus::Running(_)) {
        return (default_pidfile, default_socket);
    }
    if let Some(alt_dir) = alt_dir {
        let alt_pidfile = alt_dir.join("hub.pid");
        if matches!(check(&alt_pidfile), PidfileStatus::Running(_)) {
            return (alt_pidfile, alt_dir.join("hub.sock"));
        }
        if matches!(default_status, PidfileStatus::Stale(_)) {
            return (default_pidfile, default_socket);
        }
        if alt_pidfile.exists() {
            return (alt_pidfile, alt_dir.join("hub.sock"));
        }
    }
    (default_pidfile, default_socket)
}

/// T-2630: The IP-keyed secret cache directory (`<config-dir>/secrets`), resolved
/// through the single hardened resolver [`crate::hubs_config::termlink_config_dir`]
/// (T-2629) rather than an ad-hoc `HOME.unwrap_or_default()`. Centralizing the
/// derivation keeps the doctor's secret_cache check pointed at the SAME directory
/// the cache is actually written to, even when HOME is unset (where the old ad-hoc
/// read fell back to a CWD-relative path and reported a false-clean).
pub fn secret_cache_dir() -> PathBuf {
    crate::hubs_config::termlink_config_dir().join("secrets")
}

/// T-1171 / G-011: Audit `~/.termlink/secrets/*.hex` for perm smells and
/// staleness relative to a local hub secret. Returns a warning message per
/// issue; empty vec means all caches look healthy.
///
/// - Skips non-regular files, entries whose name doesn't end in `.hex`, and
///   `.bak` siblings.
/// - Perm check: any mode != 0o600 (low 9 bits) is flagged.
/// - Freshness check: only runs when `local_hub` is provided. A cache with
///   `mtime < hub_mtime` is flagged — the operator decides whether the
///   cache actually points at the local hub.
pub fn audit_secret_cache(
    secrets_dir: &std::path::Path,
    local_hub: Option<(&std::path::Path, std::time::SystemTime)>,
    fix: bool,
) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;
    let mut issues = Vec::new();
    let entries = match std::fs::read_dir(secrets_dir) {
        Ok(e) => e,
        Err(_) => return issues,
    };
    // T-1284: read the local hub.secret value once so we can compare each
    // cache by VALUE, not just by mtime. Value comparison eliminates the
    // mtime-based false-positive ("cache for remote hub X is older than
    // local hub.secret"); a cache that matches the local secret is
    // healthy regardless of mtime, and a cache that diverges + is older
    // is a real drift candidate (not a heuristic guess).
    let hub_value: Option<String> = local_hub.and_then(|(p, _)| {
        std::fs::read_to_string(p)
            .ok()
            .map(|s| s.trim().to_lowercase())
    });
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.ends_with(".hex") || name.ends_with(".bak") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_file() {
            continue;
        }
        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            // T-1654: when --fix is on, chmod 600 in place — the canonical
            // T-1055 write mode is the only correct state for a cached HMAC
            // secret, so auto-remediation is safe. Drift/divergence issues
            // below are NOT auto-fixed; the operator must decide what's
            // authoritative.
            if fix {
                match std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)) {
                    Ok(()) => {
                        issues.push(format!(
                            "fixed: {} mode 0o{:o} → 0o600",
                            path.display(),
                            mode
                        ));
                    }
                    Err(e) => {
                        issues.push(format!(
                            "{} has mode {:o} (expected 600) — chmod failed: {}",
                            path.display(),
                            mode,
                            e
                        ));
                    }
                }
            } else {
                issues.push(format!(
                    "{} has mode {:o} (expected 600) — world/group-readable cache",
                    path.display(),
                    mode
                ));
            }
        }
        if let Some((hub_path, hub_mtime)) = local_hub {
            // Try value comparison first — definitive when readable.
            let cache_value = std::fs::read_to_string(&path)
                .ok()
                .map(|s| s.trim().to_lowercase());
            match (cache_value.as_deref(), hub_value.as_deref()) {
                (Some(c), Some(h)) if c == h => {
                    // Match → cache IS the local hub's secret. Healthy
                    // regardless of mtime ordering. Skip the mtime check.
                    continue;
                }
                (Some(_), Some(_)) => {
                    // Diverges. Only flag when mtime suggests this cache
                    // PREDATES the current hub.secret — that's the real
                    // "stale cache for the local hub" signal. A diverging
                    // cache that's NEWER than hub.secret is almost
                    // certainly a cache of a remote hub, not drift.
                    if let Ok(cache_mtime) = meta.modified()
                        && cache_mtime < hub_mtime
                    {
                        issues.push(format!(
                            "{} cache value diverges from local hub.secret AND is older than {} — drift candidate; if this cache points at the local hub, refresh it",
                            path.display(),
                            hub_path.display()
                        ));
                    }
                    continue;
                }
                _ => {
                    // Value compare unavailable — fall back to mtime-only.
                }
            }
            if let Ok(cache_mtime) = meta.modified()
                && cache_mtime < hub_mtime
            {
                issues.push(format!(
                    "{} is older than local {} — may be stale if this cache points at the local hub",
                    path.display(),
                    hub_path.display()
                ));
            }
        }
    }
    issues
}

/// T-1284 / G-011: Audit `hubs.toml` for profiles that use an IP-keyed
/// cache file (under `secrets_dir`) for what is structurally a local-hub
/// address. Such profiles are the giving-end of cache drift: the cache
/// is written once and silently goes stale on hub restart. Per
/// CLAUDE.md R3 the fix is to point `secret_file` directly at the live
/// `<runtime_dir>/hub.secret`.
///
/// Returns one migration hint per offending profile. Empty vec means
/// no profiles need migration. The check is purely lexical/structural —
/// it does not contact any hub. Address forms recognised as "self":
/// `127.x.x.x`, `localhost`, `::1`, `0.0.0.0`. Anything else is treated
/// as remote (where IP-keyed cache is appropriate).
pub fn audit_hubs_for_self_hub_cache(
    config: &crate::hubs_config::HubsConfig,
    secrets_dir: &std::path::Path,
) -> Vec<String> {
    let mut hints = Vec::new();
    // Sort for deterministic output across runs.
    let mut names: Vec<&String> = config.hubs.keys().collect();
    names.sort();
    for name in names {
        let entry = &config.hubs[name];
        let Some(secret_file) = entry.secret_file.as_deref() else {
            continue;
        };
        let secret_path = std::path::PathBuf::from(secret_file);
        if !secret_path.starts_with(secrets_dir) {
            continue;
        }
        if !is_self_hub_address(&entry.address) {
            continue;
        }
        hints.push(format!(
            "Profile '{name}' uses IP-keyed cache ({sf}) for the local hub at {addr}. Per G-011, point secret_file directly at <runtime_dir>/hub.secret to avoid drift.",
            name = name,
            sf = secret_file,
            addr = entry.address,
        ));
    }
    hints
}

/// True if `address` (as found in hubs.toml — typically `host:port`)
/// names a loopback / wildcard interface. Strips the port if present;
/// any parse failure yields `false` (treat as remote, fail-safe).
pub fn is_self_hub_address(address: &str) -> bool {
    let host = match address.rsplit_once(':') {
        Some((h, _)) => h,
        None => address,
    }
    .trim_start_matches('[')
    .trim_end_matches(']');
    if host == "localhost" || host == "::1" || host == "0.0.0.0" {
        return true;
    }
    if let Some(rest) = host.strip_prefix("127.") {
        // Any 127.x.y.z is loopback. Coarse check is sufficient — even
        // garbage-after-127. would only false-positive a profile the
        // operator clearly meant as local.
        return rest.split('.').count() == 3;
    }
    false
}

/// T-1705: group live sessions by their `metadata.identity_fingerprint`,
/// excluding sessions without an FP (pre-T-1436). Returns a Vec of
/// `(fp, names)` so iteration order is stable for the doctor message.
/// Pure function — testable without spinning up a real hub.
pub fn group_sessions_by_identity(
    sessions: &[termlink_session::registration::Registration],
) -> Vec<(String, Vec<String>)> {
    use std::collections::BTreeMap;
    let mut by_fp: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for s in sessions {
        if let Some(fp) = s.metadata.identity_fingerprint.as_deref() {
            by_fp.entry(fp.to_string()).or_default().push(s.display_name.clone());
        }
    }
    by_fp.into_iter().collect()
}

/// T-2623: sum the inbox topics' pending `count` fields, tracking the NAMES of
/// any topic whose `count` is missing or non-u64. The prior inline fold used
/// `filter_map(|t| t["count"].as_u64()).sum()`, which silently DROPPED an
/// unreadable count — laundering it into a `0` that the doctor's `Ok((0, _))`
/// arm reported as a green "no pending transfers" PASS while inbox targets
/// existed whose counts could not be read (Directive #2 "no silent failures",
/// in the worst place for one — a health check). Returning the dropped names
/// lets the caller downgrade to `warn` and name what it could not read.
/// Pure function — testable without a live hub. Same 0/None-laundering family
/// as T-2619 / T-2621.
pub fn sum_inbox_counts(topics: &[serde_json::Value]) -> (u64, Vec<String>) {
    let mut total: u64 = 0;
    let mut dropped: Vec<String> = Vec::new();
    for t in topics {
        match t["count"].as_u64() {
            Some(c) => total += c,
            None => {
                let name = t["name"].as_str().unwrap_or("<unnamed>").to_string();
                dropped.push(name);
            }
        }
    }
    (total, dropped)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn report(fail: u32, warn: u32, strict: bool) -> DoctorReport {
        DoctorReport { checks: Vec::new(), pass: 0, warn, fail, strict }
    }

    // T-1712 rollup (moved from the MCP crate's `doctor_ok_rollup` tests).
    #[test]
    fn ok_clean_environment_regardless_of_strict() {
        assert!(report(0, 0, false).ok());
        assert!(report(0, 0, true).ok());
    }

    #[test]
    fn ok_fails_dominate_regardless_of_strict() {
        assert!(!report(1, 0, false).ok());
        assert!(!report(1, 0, true).ok());
        assert!(!report(1, 5, false).ok());
        assert!(!report(1, 5, true).ok());
    }

    #[test]
    fn ok_strict_turns_warnings_into_not_ok() {
        assert!(report(0, 1, false).ok());
        assert!(report(0, 10, false).ok());
        assert!(!report(0, 1, true).ok());
        assert!(!report(0, 10, true).ok());
    }

    // T-3214: the envelope carries `strict` and an `ok` that matches `ok()`, so
    // the CLI's JSON verdict agrees with its own exit code.
    #[test]
    fn envelope_echoes_strict_and_ok() {
        let mut r = report(0, 0, true);
        r.push("sessions_dir", CheckStatus::Warn, "missing");
        r.note("text-mode only");
        let v = r.to_json();
        assert_eq!(v["strict"], true);
        assert_eq!(v["ok"], false);
        assert_eq!(v["summary"]["warn"], 1);
        assert_eq!(v["checks"][0]["check"], "sessions_dir");
        assert_eq!(v["checks"][0]["status"], "warn");
        assert!(v["checks"][0].get("notes").is_none(), "notes must stay out of JSON");
    }
}
