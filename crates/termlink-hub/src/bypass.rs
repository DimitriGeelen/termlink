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

    /// Load from a specific path. Returns empty registry if file doesn't exist or is corrupt.
    pub fn load_from(path: &PathBuf) -> Self {
        match std::fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(reg) => reg,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Bypass registry corrupt — returning empty registry"
                    );
                    Self::default()
                }
            },
            Err(_) => Self::default(),
        }
    }

    /// Save registry to the default path.
    pub fn save(&self) -> std::io::Result<()> {
        let path = registry_path();
        self.save_to(&path)
    }

    /// Save to a specific path using atomic write (temp file + rename).
    pub fn save_to(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        // Write to temp file in same directory, then atomic rename
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &data)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
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

    /// Load the registry under an advisory file lock, apply a mutation, save atomically.
    /// This prevents concurrent load+modify+save races between hub request handlers.
    pub fn locked_update<F>(path: &PathBuf, f: F) -> std::io::Result<Self>
    where
        F: FnOnce(&mut Self),
    {
        use std::fs::OpenOptions;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let lock_path = path.with_extension("json.lock");
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&lock_path)?;

        // Acquire exclusive advisory lock
        flock_exclusive(&lock_file)?;

        // Load current state (under lock)
        let mut registry = Self::load_from(path);

        // Apply mutation
        f(&mut registry);

        // Save atomically
        registry.save_to(path)?;

        // Lock released on drop of lock_file
        drop(lock_file);

        Ok(registry)
    }
}

/// Acquire an exclusive advisory lock (blocking).
fn flock_exclusive(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Default registry file path.
pub fn registry_path() -> PathBuf {
    termlink_session::discovery::runtime_dir().join("bypass-registry.json")
}

fn now_iso() -> String {
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

    #[test]
    fn load_corrupt_json_returns_default() {
        let (_dir, path) = tmp_registry();

        // Write garbage to the registry file
        std::fs::write(&path, "not valid json {{{").unwrap();

        let reg = BypassRegistry::load_from(&path);
        assert!(reg.entries.is_empty());
        assert!(reg.candidates.is_empty());
    }

    #[test]
    fn load_empty_file_returns_default() {
        let (_dir, path) = tmp_registry();

        std::fs::write(&path, "").unwrap();

        let reg = BypassRegistry::load_from(&path);
        assert!(reg.entries.is_empty());
    }

    #[test]
    fn atomic_save_no_partial_file() {
        let (_dir, path) = tmp_registry();

        let mut reg = BypassRegistry::default();
        for i in 0..10 {
            reg.entries.insert(
                format!("cmd-{i}"),
                BypassEntry {
                    command: format!("cmd-{i}"),
                    tier: 3,
                    run_count: i as u64,
                    fail_count: 0,
                    promoted_at: "100".to_string(),
                    last_run: None,
                },
            );
        }

        reg.save_to(&path).unwrap();

        // Verify no temp file remains
        let tmp_path = path.with_extension("json.tmp");
        assert!(!tmp_path.exists());

        // Verify file is valid JSON
        let loaded = BypassRegistry::load_from(&path);
        assert_eq!(loaded.entries.len(), 10);
    }

    #[test]
    fn locked_update_serializes_mutations() {
        let (_dir, path) = tmp_registry();

        // Seed with 4 successes
        let mut reg = BypassRegistry::default();
        for _ in 0..4 {
            reg.record_orchestrated_run("test.cmd", true);
        }
        reg.save_to(&path).unwrap();

        // Apply locked update — 5th success triggers promotion
        let result = BypassRegistry::locked_update(&path, |r| {
            r.record_orchestrated_run("test.cmd", true);
        })
        .unwrap();

        assert!(result.check("test.cmd").is_some());
        assert_eq!(result.check("test.cmd").unwrap().tier, 3);

        // Verify persisted
        let loaded = BypassRegistry::load_from(&path);
        assert!(loaded.check("test.cmd").is_some());
    }

    #[tokio::test]
    async fn concurrent_locked_updates_no_data_loss() {
        let (_dir, path) = tmp_registry();

        // 10 parallel tasks each record a success for a unique command
        let mut handles = Vec::new();
        for i in 0..10 {
            let p = path.clone();
            handles.push(tokio::task::spawn_blocking(move || {
                let cmd = format!("concurrent-cmd-{i}");
                BypassRegistry::locked_update(&p, |r| {
                    r.record_orchestrated_run(&cmd, true);
                })
                .unwrap();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        // All 10 commands should be in candidates with success_count=1
        let reg = BypassRegistry::load_from(&path);
        for i in 0..10 {
            let cmd = format!("concurrent-cmd-{i}");
            let stats = reg
                .candidates
                .get(&cmd)
                .unwrap_or_else(|| panic!("Missing candidate: {cmd}"));
            assert_eq!(
                stats.success_count, 1,
                "Command {cmd} should have exactly 1 success"
            );
        }
        assert_eq!(reg.candidates.len(), 10);
    }
}
