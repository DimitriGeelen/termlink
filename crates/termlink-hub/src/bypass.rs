use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Bypass registry — tracks commands that have earned Tier 3 (autonomous execution).
///
/// Commands promoted after `PROMOTION_THRESHOLD` successful orchestrated runs with
/// zero failures. Failed bypass executions de-promote the command.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BypassRegistry {
    pub entries: HashMap<String, BypassEntry>,
    /// Tracks orchestrated run counts for commands not yet promoted.
    #[serde(default)]
    pub candidates: HashMap<String, RunStats>,
}

/// A promoted command in the bypass registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassEntry {
    pub command: String,
    pub tier: u8,
    pub run_count: u64,
    pub fail_count: u64,
    pub promoted_at: String,
    pub last_run: Option<String>,
}

/// Pre-promotion tracking for commands still being observed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunStats {
    pub success_count: u64,
    pub fail_count: u64,
}

/// Number of successful orchestrated runs required for promotion.
pub const PROMOTION_THRESHOLD: u64 = 5;

impl BypassRegistry {
    /// Load registry from the default path (`{runtime_dir}/bypass-registry.json`).
    pub fn load() -> Self {
        let path = registry_path();
        Self::load_from(&path)
    }

    /// Load from a specific path. Returns empty registry if file doesn't exist.
    pub fn load_from(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save registry to the default path.
    pub fn save(&self) -> std::io::Result<()> {
        let path = registry_path();
        self.save_to(&path)
    }

    /// Save to a specific path.
    pub fn save_to(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)
    }

    /// Check if a command is in the bypass registry (Tier 3 promoted).
    pub fn check(&self, command: &str) -> Option<&BypassEntry> {
        self.entries.get(command)
    }

    /// Record an orchestrated run for a command (pre-promotion tracking).
    /// Returns `true` if the command was just promoted to bypass.
    pub fn record_orchestrated_run(&mut self, command: &str, success: bool) -> bool {
        // If already promoted, just update stats
        if let Some(entry) = self.entries.get_mut(command) {
            entry.run_count += 1;
            if !success {
                entry.fail_count += 1;
            }
            entry.last_run = Some(now_iso());
            return false;
        }

        let stats = self.candidates.entry(command.to_string()).or_default();
        if success {
            stats.success_count += 1;
        } else {
            stats.fail_count += 1;
        }

        // Check promotion threshold
        if stats.success_count >= PROMOTION_THRESHOLD && stats.fail_count == 0 {
            self.candidates.remove(command);
            self.entries.insert(
                command.to_string(),
                BypassEntry {
                    command: command.to_string(),
                    tier: 3,
                    run_count: 0,
                    fail_count: 0,
                    promoted_at: now_iso(),
                    last_run: None,
                },
            );
            return true;
        }

        false
    }

    /// Record a bypass execution result. De-promotes on failure.
    /// Returns `true` if the command was de-promoted.
    pub fn record_bypass_run(&mut self, command: &str, success: bool) -> bool {
        if let Some(entry) = self.entries.get_mut(command) {
            entry.run_count += 1;
            entry.last_run = Some(now_iso());
            if !success {
                entry.fail_count += 1;
                // De-promote: remove from registry
                self.entries.remove(command);
                return true;
            }
        }
        false
    }
}

/// Default registry file path.
fn registry_path() -> PathBuf {
    termlink_session::discovery::runtime_dir().join("bypass-registry.json")
}

fn now_iso() -> String {
    // Simple timestamp — uses system time formatted as ISO 8601
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{now}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn tmp_registry() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("bypass-registry.json");
        (dir, path)
    }

    #[test]
    fn load_save_round_trip() {
        let (_dir, path) = tmp_registry();

        let mut reg = BypassRegistry::default();
        reg.entries.insert(
            "fw doctor".to_string(),
            BypassEntry {
                command: "fw doctor".to_string(),
                tier: 3,
                run_count: 10,
                fail_count: 0,
                promoted_at: "12345".to_string(),
                last_run: Some("12346".to_string()),
            },
        );

        reg.save_to(&path).unwrap();
        let loaded = BypassRegistry::load_from(&path);

        assert_eq!(loaded.entries.len(), 1);
        let entry = loaded.entries.get("fw doctor").unwrap();
        assert_eq!(entry.tier, 3);
        assert_eq!(entry.run_count, 10);
    }

    #[test]
    fn check_hit_and_miss() {
        let mut reg = BypassRegistry::default();
        reg.entries.insert(
            "git status".to_string(),
            BypassEntry {
                command: "git status".to_string(),
                tier: 3,
                run_count: 5,
                fail_count: 0,
                promoted_at: "100".to_string(),
                last_run: None,
            },
        );

        assert!(reg.check("git status").is_some());
        assert!(reg.check("rm -rf /").is_none());
    }

    #[test]
    fn promotion_after_threshold() {
        let mut reg = BypassRegistry::default();

        // 4 runs — not yet promoted
        for _ in 0..4 {
            assert!(!reg.record_orchestrated_run("fw metrics", true));
        }
        assert!(reg.check("fw metrics").is_none());

        // 5th run — promoted
        assert!(reg.record_orchestrated_run("fw metrics", true));
        assert!(reg.check("fw metrics").is_some());
        assert_eq!(reg.check("fw metrics").unwrap().tier, 3);
    }

    #[test]
    fn promotion_blocked_by_failure() {
        let mut reg = BypassRegistry::default();

        for _ in 0..4 {
            reg.record_orchestrated_run("flaky_cmd", true);
        }
        // One failure resets the zero-failure requirement
        reg.record_orchestrated_run("flaky_cmd", false);

        // 5th success — but fail_count > 0, so no promotion
        reg.record_orchestrated_run("flaky_cmd", true);
        assert!(reg.check("flaky_cmd").is_none());
    }

    #[test]
    fn demotion_on_bypass_failure() {
        let mut reg = BypassRegistry::default();
        reg.entries.insert(
            "fw doctor".to_string(),
            BypassEntry {
                command: "fw doctor".to_string(),
                tier: 3,
                run_count: 10,
                fail_count: 0,
                promoted_at: "100".to_string(),
                last_run: None,
            },
        );

        // Successful bypass run — stays promoted
        assert!(!reg.record_bypass_run("fw doctor", true));
        assert!(reg.check("fw doctor").is_some());

        // Failed bypass run — de-promoted
        assert!(reg.record_bypass_run("fw doctor", false));
        assert!(reg.check("fw doctor").is_none());
    }
}
