//! T-3310 (arc-012 step 5): hub-side topic policy — the retention config file
//! (D3: limits are configurable) and the "forever needs an owner" enforcement
//! (D1, operator ruling D+).
//!
//! **Config file.** `<runtime_dir>/retention.yaml`, optional; an absent file
//! means built-in defaults. Every key is optional:
//!
//! ```yaml
//! max_records: 10000          # count ceiling for day/per-key topics; forever warns here
//! max_live_bytes: 67108864    # live-byte ceiling (64 MB); forever warns here
//! ceilings_on_post: true      # false = bound only by the background sweeper
//! forever_requires_owner: auto   # auto | on | never
//! forever_quiet_days: 14      # auto mode: days without a bare-forever create before enforcing
//! ```
//!
//! Environment variables override the file: `TERMLINK_FOREVER_REQUIRES_OWNER`,
//! `TERMLINK_FOREVER_QUIET_DAYS`, `TERMLINK_MAX_TOPIC_RECORDS`,
//! `TERMLINK_MAX_TOPIC_LIVE_BYTES`, `TERMLINK_CEILINGS_ON_POST`.
//!
//! **D1 enforcement.** Every client asked for `forever` explicitly by default and
//! old binaries cannot send an owner, so refusing immediately would break the
//! fleet. Instead a bare forever create (no owner, no reason) of a NEW topic is
//! accepted, labelled `unowned_forever`, and recorded. In `auto` mode the hub
//! starts refusing once it has watched for `quiet_days` AND no bare-forever
//! create has happened for `quiet_days` — i.e. the fleet's own behaviour shows
//! every topic-creating client has upgraded. `on` refuses now; `never` never
//! refuses but is reported (governor status, backstop canary) every day it is set.

use std::path::Path;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use termlink_bus::{BareForeverSummary, Ceilings};

pub const CONFIG_FILE: &str = "retention.yaml";
pub const DEFAULT_QUIET_DAYS: u64 = 14;
const DAY_MS: i64 = 86_400_000;

/// D1 mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ForeverMode {
    Auto,
    On,
    Never,
}

impl ForeverMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "on" | "true" | "1" => Some(Self::On),
            "never" | "off" | "false" | "0" => Some(Self::Never),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Never => "never",
        }
    }
}

/// The on-disk file shape. Unknown keys are rejected so a typo is loud.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionFile {
    pub max_records: Option<u64>,
    pub max_live_bytes: Option<u64>,
    pub ceilings_on_post: Option<bool>,
    pub forever_requires_owner: Option<String>,
    pub forever_quiet_days: Option<u64>,
}

/// The effective policy after file + environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopicPolicy {
    pub ceilings: Ceilings,
    pub forever_mode: ForeverMode,
    pub quiet_days: u64,
}

impl Default for TopicPolicy {
    fn default() -> Self {
        Self { ceilings: Ceilings::default(), forever_mode: ForeverMode::Auto, quiet_days: DEFAULT_QUIET_DAYS }
    }
}

/// Merge a parsed file and an environment lookup over the defaults. Pure, so
/// precedence is tested without touching the process environment.
pub fn resolve(file: &RetentionFile, env: &dyn Fn(&str) -> Option<String>) -> Result<TopicPolicy, String> {
    let mut p = TopicPolicy::default();
    if let Some(v) = file.max_records {
        p.ceilings.max_records = v;
    }
    if let Some(v) = file.max_live_bytes {
        p.ceilings.max_live_bytes = v;
    }
    if let Some(v) = file.ceilings_on_post {
        p.ceilings.on_post = v;
    }
    if let Some(v) = &file.forever_requires_owner {
        p.forever_mode = ForeverMode::parse(v)
            .ok_or_else(|| format!("forever_requires_owner: '{v}' is not auto|on|never"))?;
    }
    if let Some(v) = file.forever_quiet_days {
        p.quiet_days = v;
    }
    let num = |key: &str| -> Result<Option<u64>, String> {
        match env(key) {
            None => Ok(None),
            Some(s) => s.trim().parse().map(Some).map_err(|_| format!("{key}: '{s}' is not a number")),
        }
    };
    if let Some(v) = num("TERMLINK_MAX_TOPIC_RECORDS")? {
        p.ceilings.max_records = v;
    }
    if let Some(v) = num("TERMLINK_MAX_TOPIC_LIVE_BYTES")? {
        p.ceilings.max_live_bytes = v;
    }
    if let Some(v) = num("TERMLINK_FOREVER_QUIET_DAYS")? {
        p.quiet_days = v;
    }
    if let Some(s) = env("TERMLINK_CEILINGS_ON_POST") {
        p.ceilings.on_post = !matches!(s.trim().to_ascii_lowercase().as_str(), "0" | "false" | "off" | "no");
    }
    if let Some(s) = env("TERMLINK_FOREVER_REQUIRES_OWNER") {
        p.forever_mode = ForeverMode::parse(&s)
            .ok_or_else(|| format!("TERMLINK_FOREVER_REQUIRES_OWNER: '{s}' is not auto|on|never"))?;
    }
    if p.ceilings.max_records == 0 || p.ceilings.max_live_bytes == 0 {
        return Err("max_records and max_live_bytes must be >= 1".into());
    }
    Ok(p)
}

/// Load `<runtime_dir>/retention.yaml` plus the environment. A missing file is
/// the defaults; an unreadable or invalid one is an error the caller logs
/// loudly (the hub then runs on defaults rather than refusing to start).
pub fn load(runtime_dir: &Path) -> Result<TopicPolicy, String> {
    let path = runtime_dir.join(CONFIG_FILE);
    let file = match std::fs::read_to_string(&path) {
        Ok(s) if s.trim().is_empty() => RetentionFile::default(),
        Ok(s) => serde_yaml::from_str(&s).map_err(|e| format!("{}: {e}", path.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RetentionFile::default(),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    resolve(&file, &|k| std::env::var(k).ok())
}

static POLICY: OnceLock<TopicPolicy> = OnceLock::new();

/// Set once at hub start (`channel::init_bus`).
pub fn set_policy(p: TopicPolicy) {
    let _ = POLICY.set(p);
}

pub fn policy() -> TopicPolicy {
    POLICY.get().copied().unwrap_or_default()
}

/// D1 enforcement state at `now_ms`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Enforcement {
    pub mode: &'static str,
    /// True when a bare forever create of a new topic is refused right now.
    pub enforced: bool,
    pub quiet_days: u64,
    /// When the quiet window started: the later of the watch start and the
    /// last bare-forever create.
    pub quiet_since_ms: i64,
    /// Auto mode, not yet enforced: when enforcement will switch on if no
    /// bare-forever create happens before then.
    pub enforce_at_ms: Option<i64>,
}

/// Pure D1 clock.
pub fn enforcement(mode: ForeverMode, quiet_days: u64, s: &BareForeverSummary, now_ms: i64) -> Enforcement {
    let quiet_since_ms = s.last_create_ms.map_or(s.watch_since_ms, |l| l.max(s.watch_since_ms));
    let window = (quiet_days as i64).saturating_mul(DAY_MS);
    let (enforced, enforce_at_ms) = match mode {
        ForeverMode::On => (true, None),
        ForeverMode::Never => (false, None),
        ForeverMode::Auto => {
            let at = quiet_since_ms.saturating_add(window);
            if now_ms >= at { (true, None) } else { (false, Some(at)) }
        }
    };
    Enforcement { mode: mode.as_str(), enforced, quiet_days, quiet_since_ms, enforce_at_ms }
}

/// The refusal text a client gets once enforcement is on. Names the fix.
pub fn refusal_message(name: &str) -> String {
    format!(
        "channel.create: topic '{name}' asks for retention 'forever' without an owner and a reason \
         (T-3310). Either pass a bounded retention (new topics default to 14 days; mail topics to the \
         newest 1000), or name who keeps it and why: CLI `channel create {name} --retention forever \
         --owner <who> --reason <why>`, RPC params {{\"owner\": .., \"reason\": ..}}. Upgrade old \
         termlink binaries: they send 'forever' by default."
    )
}

/// T-3310: the fields `hub.governor_status` reports — ceilings, post-time
/// trim counters, and the D1 enforcement state including who still sends bare
/// forever. Absent bus = policy only. The backstop canary reads these.
pub fn status_fields(bus: Option<&termlink_bus::Bus>, now_ms: i64) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::json;
    let pol = policy();
    let mut m = serde_json::Map::new();
    m.insert("retention_max_records".into(), json!(pol.ceilings.max_records));
    m.insert("retention_max_live_bytes".into(), json!(pol.ceilings.max_live_bytes));
    m.insert("retention_ceilings_on_post".into(), json!(pol.ceilings.on_post));
    m.insert("forever_requires_owner".into(), json!(pol.forever_mode.as_str()));
    if let Some(bus) = bus {
        let c = bus.ceiling_counters();
        m.insert("post_trims_total".into(), json!(c.post_trims_total));
        m.insert("post_trimmed_records_total".into(), json!(c.post_trimmed_records_total));
        m.insert("forever_warnings_total".into(), json!(c.forever_warnings_total));
        if let Ok(s) = bus.bare_forever_summary() {
            let e = enforcement(pol.forever_mode, pol.quiet_days, &s, now_ms);
            m.insert("forever_enforced".into(), json!(e.enforced));
            m.insert("forever_quiet_days".into(), json!(e.quiet_days));
            m.insert("forever_quiet_since_ms".into(), json!(e.quiet_since_ms));
            m.insert("forever_enforce_at_ms".into(), json!(e.enforce_at_ms));
            m.insert("bare_forever_senders".into(), json!(s.senders.iter().take(20).collect::<Vec<_>>()));
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(watch: i64, last: Option<i64>) -> BareForeverSummary {
        BareForeverSummary { watch_since_ms: watch, last_create_ms: last, senders: vec![] }
    }

    #[test]
    fn auto_flips_after_14_quiet_days_not_13() {
        let s = summary(0, None);
        assert!(!enforcement(ForeverMode::Auto, 14, &s, 13 * DAY_MS).enforced);
        let e = enforcement(ForeverMode::Auto, 14, &s, 14 * DAY_MS);
        assert!(e.enforced, "14 quiet days since watching began");
        assert_eq!(e.enforce_at_ms, None);
    }

    #[test]
    fn a_bare_create_restarts_the_clock() {
        // Watching for 30 days, but a bare forever create 5 days ago.
        let s = summary(0, Some(25 * DAY_MS));
        let e = enforcement(ForeverMode::Auto, 14, &s, 30 * DAY_MS);
        assert!(!e.enforced);
        assert_eq!(e.quiet_since_ms, 25 * DAY_MS);
        assert_eq!(e.enforce_at_ms, Some(39 * DAY_MS));
        assert!(enforcement(ForeverMode::Auto, 14, &s, 39 * DAY_MS).enforced);
    }

    #[test]
    fn on_and_never_ignore_the_clock() {
        let s = summary(0, Some(0));
        assert!(enforcement(ForeverMode::On, 14, &s, 0).enforced);
        assert!(!enforcement(ForeverMode::Never, 14, &s, 1_000 * DAY_MS).enforced);
    }

    #[test]
    fn env_overrides_file_and_bad_values_are_loud() {
        let file = RetentionFile {
            max_records: Some(500),
            forever_requires_owner: Some("never".into()),
            ..Default::default()
        };
        let none = |_: &str| None;
        let p = resolve(&file, &none).unwrap();
        assert_eq!((p.ceilings.max_records, p.forever_mode), (500, ForeverMode::Never));
        let env = |k: &str| match k {
            "TERMLINK_FOREVER_REQUIRES_OWNER" => Some("auto".to_string()),
            "TERMLINK_CEILINGS_ON_POST" => Some("off".to_string()),
            _ => None,
        };
        let p = resolve(&file, &env).unwrap();
        assert_eq!(p.forever_mode, ForeverMode::Auto);
        assert!(!p.ceilings.on_post);
        let bad = RetentionFile { forever_requires_owner: Some("sometimes".into()), ..Default::default() };
        assert!(resolve(&bad, &none).is_err());
        let zero = RetentionFile { max_records: Some(0), ..Default::default() };
        assert!(resolve(&zero, &none).is_err());
    }

    #[test]
    fn load_reads_yaml_and_rejects_unknown_keys() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()).unwrap().quiet_days, DEFAULT_QUIET_DAYS, "absent file = defaults");
        std::fs::write(dir.path().join(CONFIG_FILE), "max_live_bytes: 1024\nforever_quiet_days: 3\n").unwrap();
        let p = load(dir.path()).unwrap();
        assert_eq!((p.ceilings.max_live_bytes, p.quiet_days), (1024, 3));
        std::fs::write(dir.path().join(CONFIG_FILE), "max_recrods: 5\n").unwrap();
        assert!(load(dir.path()).is_err(), "a typo must be loud");
    }
}
