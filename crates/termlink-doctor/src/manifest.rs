//! Dispatch manifest data model (T-3214) — `.termlink/dispatch-manifest.json`,
//! the lifecycle record of every `termlink dispatch --isolate` branch
//! (pending → merged | conflict | deferred | expired).
//!
//! Moved here from the CLI's `manifest.rs` so the doctor's `dispatch` check reads
//! the manifest through ONE parser. The git worktree operations stay in the CLI,
//! which re-exports these items.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Manifest file location relative to project root.
pub const MANIFEST_DIR: &str = ".termlink";
pub const MANIFEST_FILE: &str = "dispatch-manifest.json";

/// Root manifest structure persisted to JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchManifest {
    pub version: String,
    pub last_updated: String,
    pub dispatches: Vec<DispatchRecord>,
}

/// A single dispatch operation and its tracking state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRecord {
    pub id: String,
    pub created_at: String,
    pub status: DispatchStatus,
    pub worker_count: u32,
    pub topic: String,
    pub prefix: String,
    pub branches: Vec<BranchEntry>,
    /// LLM model used for this dispatch (None = default/unspecified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Individual branch created for a worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchEntry {
    pub worker_name: String,
    pub branch_name: String,
    pub base_branch: String,
    pub worktree_path: String,
    pub has_commits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DispatchStatus {
    Pending,
    Merged,
    Conflict,
    Deferred,
    Expired,
}

impl DispatchManifest {
    /// Load manifest from project root, or return empty manifest if file doesn't exist.
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = Self::manifest_path(project_root);
        if !path.exists() {
            return Ok(Self::empty());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse manifest: {}", path.display()))
    }

    /// Save manifest to project root.
    pub fn save(&mut self, project_root: &Path) -> Result<()> {
        let dir = project_root.join(MANIFEST_DIR);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
        }
        self.last_updated = now_rfc3339();
        let path = Self::manifest_path(project_root);
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize manifest")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write manifest: {}", path.display()))
    }

    /// Add a new dispatch record.
    pub fn add_dispatch(&mut self, record: DispatchRecord) {
        self.dispatches.push(record);
    }

    /// Find a mutable dispatch record by ID.
    pub fn find_dispatch_mut(&mut self, dispatch_id: &str) -> Option<&mut DispatchRecord> {
        self.dispatches.iter_mut().find(|d| d.id == dispatch_id)
    }

    /// Get all pending dispatch records. Used by gate check (T-793).
    pub fn pending_dispatches(&self) -> Vec<&DispatchRecord> {
        self.dispatches
            .iter()
            .filter(|d| d.status == DispatchStatus::Pending)
            .collect()
    }

    /// Count dispatches by status. Used by dispatch status command (T-794).
    pub fn count_by_status(&self, status: &DispatchStatus) -> usize {
        self.dispatches.iter().filter(|d| d.status == *status).count()
    }

    fn empty() -> Self {
        Self {
            version: "1".to_string(),
            last_updated: now_rfc3339(),
            dispatches: Vec::new(),
        }
    }

    fn manifest_path(project_root: &Path) -> PathBuf {
        project_root.join(MANIFEST_DIR).join(MANIFEST_FILE)
    }
}

/// Convert unix timestamp (seconds) to RFC3339 string.
pub fn secs_to_rfc3339(secs: u64) -> String {
    let secs_per_day = 86400u64;
    let days_since_epoch = secs / secs_per_day;
    let secs_today = secs % secs_per_day;
    let hours = secs_today / 3600;
    let minutes = (secs_today % 3600) / 60;
    let seconds = secs_today % 60;

    // Days to Y-M-D (simplified leap year calculation)
    let mut y = 1970i64;
    let mut remaining_days = days_since_epoch as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    let days_in_months: [i64; 12] = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 0usize;
    for (i, &dm) in days_in_months.iter().enumerate() {
        if remaining_days < dm {
            m = i;
            break;
        }
        remaining_days -= dm;
    }
    let d = remaining_days + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        d,
        hours,
        minutes,
        seconds
    )
}

pub fn now_rfc3339() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    secs_to_rfc3339(now)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
