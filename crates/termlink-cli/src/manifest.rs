//! Dispatch manifest — tracks worktree branches created by `termlink dispatch --isolate`.
//!
//! The manifest is a JSON file at `.termlink/dispatch-manifest.json` that records
//! every dispatch branch's lifecycle: pending → merged | conflict | deferred | expired.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Manifest file location relative to project root.
const MANIFEST_DIR: &str = ".termlink";
const MANIFEST_FILE: &str = "dispatch-manifest.json";

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

fn now_rfc3339() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple RFC3339-ish format without chrono dependency
    let secs_per_day = 86400u64;
    let days_since_epoch = now / secs_per_day;
    let secs_today = now % secs_per_day;
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

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

// === Git worktree operations ===

/// Create a git worktree for a dispatch worker.
pub fn create_worktree(
    project_root: &Path,
    branch_name: &str,
) -> Result<PathBuf> {
    let worktree_dir = std::env::temp_dir().join(format!(
        "termlink-dispatch-{}",
        branch_name.replace('/', "-")
    ));

    let output = std::process::Command::new("git")
        .args(["worktree", "add", "-b", branch_name])
        .arg(&worktree_dir)
        .arg("HEAD")
        .current_dir(project_root)
        .output()
        .context("Failed to run git worktree add")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git worktree add failed: {stderr}");
    }

    Ok(worktree_dir)
}

/// Auto-commit any changes in a worktree, returning true if a commit was made.
pub fn auto_commit_worktree(worktree_path: &Path, worker_name: &str) -> Result<bool> {
    // Check for changes
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()
        .context("Failed to check git status in worktree")?;

    let changes = String::from_utf8_lossy(&status.stdout);
    if changes.trim().is_empty() {
        return Ok(false);
    }

    // Stage all changes
    let add = std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(worktree_path)
        .status()
        .context("Failed to git add in worktree")?;
    if !add.success() {
        anyhow::bail!("git add failed in worktree");
    }

    // Commit
    let commit_msg = format!("dispatch: auto-commit from worker {worker_name}");
    let commit = std::process::Command::new("git")
        .args(["commit", "-m", &commit_msg])
        .current_dir(worktree_path)
        .status()
        .context("Failed to git commit in worktree")?;
    if !commit.success() {
        anyhow::bail!("git commit failed in worktree");
    }

    Ok(true)
}

/// Remove a worktree and optionally delete the branch if no commits were made.
pub fn cleanup_worktree(
    project_root: &Path,
    worktree_path: &Path,
    branch_name: &str,
    has_commits: bool,
) -> Result<()> {
    // Remove worktree
    let _ = std::process::Command::new("git")
        .args(["worktree", "remove", "--force"])
        .arg(worktree_path)
        .current_dir(project_root)
        .status();

    // Delete branch only if no commits were made
    if !has_commits {
        let _ = std::process::Command::new("git")
            .args(["branch", "-D", branch_name])
            .current_dir(project_root)
            .status();
    }

    Ok(())
}

/// Get the current git branch name.
pub fn current_branch(project_root: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_root)
        .output()
        .context("Failed to get current branch")?;

    if !output.status.success() {
        anyhow::bail!("Not a git repository");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Merge a dispatch branch back into the base branch.
/// Returns Ok(true) if merged successfully, Ok(false) if there was a conflict.
pub fn merge_branch(
    project_root: &Path,
    branch_name: &str,
    base_branch: &str,
) -> Result<bool> {
    // Checkout base branch
    let checkout = std::process::Command::new("git")
        .args(["checkout", base_branch])
        .current_dir(project_root)
        .output()
        .context("Failed to checkout base branch")?;
    if !checkout.status.success() {
        let stderr = String::from_utf8_lossy(&checkout.stderr);
        anyhow::bail!("Failed to checkout {base_branch}: {stderr}");
    }

    // Attempt merge
    let merge = std::process::Command::new("git")
        .args(["merge", "--no-edit", branch_name])
        .current_dir(project_root)
        .output()
        .context("Failed to run git merge")?;

    if merge.status.success() {
        // Merge succeeded — delete the branch
        let _ = std::process::Command::new("git")
            .args(["branch", "-d", branch_name])
            .current_dir(project_root)
            .status();
        Ok(true)
    } else {
        // Merge conflict — abort and preserve branch
        let _ = std::process::Command::new("git")
            .args(["merge", "--abort"])
            .current_dir(project_root)
            .status();
        Ok(false)
    }
}

/// Check if the current directory is inside a git repository.
pub fn is_git_repo(path: &Path) -> bool {
    std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_manifest_empty_on_missing_file() {
        let tmp = std::env::temp_dir().join("termlink-test-manifest-missing");
        let manifest = DispatchManifest::load(&tmp).unwrap();
        assert_eq!(manifest.version, "1");
        assert!(manifest.dispatches.is_empty());
    }

    #[test]
    fn test_manifest_save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        let mut manifest = DispatchManifest::load(root).unwrap();
        manifest.add_dispatch(DispatchRecord {
            id: "D-123-456".to_string(),
            created_at: "2026-03-30T12:00:00Z".to_string(),
            status: DispatchStatus::Pending,
            worker_count: 3,
            topic: "task.completed".to_string(),
            prefix: "worker".to_string(),
            branches: vec![BranchEntry {
                worker_name: "worker-1".to_string(),
                branch_name: "tl-dispatch/D-123-456/worker-1".to_string(),
                base_branch: "main".to_string(),
                worktree_path: "/tmp/test".to_string(),
                has_commits: false,
            }],
        });
        manifest.save(root).unwrap();

        let loaded = DispatchManifest::load(root).unwrap();
        assert_eq!(loaded.dispatches.len(), 1);
        assert_eq!(loaded.dispatches[0].id, "D-123-456");
        assert_eq!(loaded.dispatches[0].status, DispatchStatus::Pending);
        assert_eq!(loaded.dispatches[0].branches.len(), 1);
        assert_eq!(loaded.dispatches[0].branches[0].worker_name, "worker-1");
    }

    #[test]
    fn test_manifest_add_multiple_dispatches() {
        let mut manifest = DispatchManifest::load(Path::new("/nonexistent")).unwrap();

        manifest.add_dispatch(DispatchRecord {
            id: "D-1-100".to_string(),
            created_at: "2026-03-30T10:00:00Z".to_string(),
            status: DispatchStatus::Pending,
            worker_count: 2,
            topic: "task.completed".to_string(),
            prefix: "a".to_string(),
            branches: vec![],
        });
        manifest.add_dispatch(DispatchRecord {
            id: "D-2-200".to_string(),
            created_at: "2026-03-30T11:00:00Z".to_string(),
            status: DispatchStatus::Merged,
            worker_count: 1,
            topic: "task.completed".to_string(),
            prefix: "b".to_string(),
            branches: vec![],
        });

        assert_eq!(manifest.dispatches.len(), 2);
        assert_eq!(manifest.count_by_status(&DispatchStatus::Pending), 1);
        assert_eq!(manifest.count_by_status(&DispatchStatus::Merged), 1);
        assert_eq!(manifest.pending_dispatches().len(), 1);
    }

    #[test]
    fn test_manifest_find_dispatch_mut() {
        let mut manifest = DispatchManifest::load(Path::new("/nonexistent")).unwrap();
        manifest.add_dispatch(DispatchRecord {
            id: "D-1-100".to_string(),
            created_at: "2026-03-30T10:00:00Z".to_string(),
            status: DispatchStatus::Pending,
            worker_count: 1,
            topic: "task.completed".to_string(),
            prefix: "w".to_string(),
            branches: vec![],
        });

        let record = manifest.find_dispatch_mut("D-1-100").unwrap();
        record.status = DispatchStatus::Merged;
        assert_eq!(manifest.dispatches[0].status, DispatchStatus::Merged);

        assert!(manifest.find_dispatch_mut("D-nonexistent").is_none());
    }

    #[test]
    fn test_manifest_handles_corrupt_json() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(MANIFEST_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(MANIFEST_FILE);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"{ this is not valid json }}}").unwrap();

        let result = DispatchManifest::load(root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse"));
    }

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = DispatchManifest {
            version: "1".to_string(),
            last_updated: "2026-03-30T12:00:00Z".to_string(),
            dispatches: vec![DispatchRecord {
                id: "D-99-999".to_string(),
                created_at: "2026-03-30T12:00:00Z".to_string(),
                status: DispatchStatus::Conflict,
                worker_count: 5,
                topic: "custom.topic".to_string(),
                prefix: "test".to_string(),
                branches: vec![
                    BranchEntry {
                        worker_name: "test-1".to_string(),
                        branch_name: "tl-dispatch/D-99-999/test-1".to_string(),
                        base_branch: "develop".to_string(),
                        worktree_path: "/tmp/wt1".to_string(),
                        has_commits: true,
                    },
                    BranchEntry {
                        worker_name: "test-2".to_string(),
                        branch_name: "tl-dispatch/D-99-999/test-2".to_string(),
                        base_branch: "develop".to_string(),
                        worktree_path: "/tmp/wt2".to_string(),
                        has_commits: false,
                    },
                ],
            }],
        };

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let deserialized: DispatchManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, "1");
        assert_eq!(deserialized.dispatches.len(), 1);
        assert_eq!(deserialized.dispatches[0].branches.len(), 2);
        assert_eq!(deserialized.dispatches[0].status, DispatchStatus::Conflict);
        assert!(deserialized.dispatches[0].branches[0].has_commits);
        assert!(!deserialized.dispatches[0].branches[1].has_commits);
    }

    #[test]
    fn test_dispatch_status_serde() {
        // Verify lowercase serialization
        let pending = serde_json::to_string(&DispatchStatus::Pending).unwrap();
        assert_eq!(pending, "\"pending\"");
        let merged = serde_json::to_string(&DispatchStatus::Merged).unwrap();
        assert_eq!(merged, "\"merged\"");
        let conflict = serde_json::to_string(&DispatchStatus::Conflict).unwrap();
        assert_eq!(conflict, "\"conflict\"");

        // Roundtrip
        let from_str: DispatchStatus = serde_json::from_str("\"deferred\"").unwrap();
        assert_eq!(from_str, DispatchStatus::Deferred);
        let from_str: DispatchStatus = serde_json::from_str("\"expired\"").unwrap();
        assert_eq!(from_str, DispatchStatus::Expired);
    }

    #[test]
    fn test_now_rfc3339_format() {
        let ts = now_rfc3339();
        // Should be YYYY-MM-DDTHH:MM:SSZ format
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
    }

    #[test]
    fn test_is_git_repo_on_temp_dir() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!is_git_repo(tmp.path()));
    }

    /// Helper: create a temporary git repo with an initial commit.
    fn create_test_git_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(path)
            .output()
            .unwrap();

        // Create initial commit (git worktree requires at least one commit)
        std::fs::write(path.join("README.md"), "# Test repo\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(path)
            .output()
            .unwrap();

        tmp
    }

    #[test]
    fn test_is_git_repo_on_real_repo() {
        let repo = create_test_git_repo();
        assert!(is_git_repo(repo.path()));
    }

    #[test]
    fn test_current_branch_on_real_repo() {
        let repo = create_test_git_repo();
        let branch = current_branch(repo.path()).unwrap();
        // Default branch is either "main" or "master"
        assert!(
            branch == "main" || branch == "master",
            "Expected main or master, got: {branch}"
        );
    }

    #[test]
    fn test_create_and_cleanup_worktree() {
        let repo = create_test_git_repo();
        let branch = "tl-dispatch/test-branch";

        // Create worktree
        let wt_path = create_worktree(repo.path(), branch).unwrap();
        assert!(wt_path.exists(), "Worktree directory should exist");
        assert!(
            wt_path.join("README.md").exists(),
            "Worktree should have initial files"
        );

        // Verify branch was created
        let output = std::process::Command::new("git")
            .args(["branch", "--list", branch])
            .current_dir(repo.path())
            .output()
            .unwrap();
        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(branches.contains(branch), "Branch should exist");

        // Cleanup without commits — branch should be deleted
        cleanup_worktree(repo.path(), &wt_path, branch, false).unwrap();
        assert!(!wt_path.exists(), "Worktree directory should be removed");

        let output = std::process::Command::new("git")
            .args(["branch", "--list", branch])
            .current_dir(repo.path())
            .output()
            .unwrap();
        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(
            !branches.contains(branch),
            "Branch should be deleted when no commits"
        );
    }

    #[test]
    fn test_auto_commit_worktree_with_changes() {
        let repo = create_test_git_repo();
        let branch = "tl-dispatch/worker-1";

        let wt_path = create_worktree(repo.path(), branch).unwrap();

        // Make changes in the worktree
        std::fs::write(wt_path.join("output.txt"), "worker result\n").unwrap();

        // Auto-commit should succeed and return true
        let committed = auto_commit_worktree(&wt_path, "worker-1").unwrap();
        assert!(committed, "Should have committed changes");

        // Cleanup with has_commits=true — branch should be preserved
        cleanup_worktree(repo.path(), &wt_path, branch, true).unwrap();

        let output = std::process::Command::new("git")
            .args(["branch", "--list", branch])
            .current_dir(repo.path())
            .output()
            .unwrap();
        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(
            branches.contains(branch),
            "Branch should be preserved when has commits"
        );
    }

    #[test]
    fn test_auto_commit_worktree_no_changes() {
        let repo = create_test_git_repo();
        let branch = "tl-dispatch/idle-worker";

        let wt_path = create_worktree(repo.path(), branch).unwrap();

        // No changes — auto-commit should return false
        let committed = auto_commit_worktree(&wt_path, "idle-worker").unwrap();
        assert!(!committed, "Should not commit when no changes");

        cleanup_worktree(repo.path(), &wt_path, branch, false).unwrap();
    }

    #[test]
    fn test_merge_branch_clean() {
        let repo = create_test_git_repo();
        let base = current_branch(repo.path()).unwrap();
        let branch = "tl-dispatch/merge-test";

        let wt_path = create_worktree(repo.path(), branch).unwrap();

        // Make a change and commit
        std::fs::write(wt_path.join("new-file.txt"), "from worker\n").unwrap();
        auto_commit_worktree(&wt_path, "merge-worker").unwrap();

        // Cleanup worktree but keep branch
        cleanup_worktree(repo.path(), &wt_path, branch, true).unwrap();

        // Merge should succeed
        let merged = merge_branch(repo.path(), branch, &base).unwrap();
        assert!(merged, "Merge should succeed for non-conflicting changes");

        // Verify merged file exists on base branch
        assert!(
            repo.path().join("new-file.txt").exists(),
            "Merged file should exist on base branch"
        );
    }

    #[test]
    fn test_merge_branch_conflict() {
        let repo = create_test_git_repo();
        let base = current_branch(repo.path()).unwrap();

        // Create two branches that modify the same file
        let branch1 = "tl-dispatch/conflict-1";
        let branch2 = "tl-dispatch/conflict-2";

        let wt1 = create_worktree(repo.path(), branch1).unwrap();
        std::fs::write(wt1.join("README.md"), "Worker 1 changes\n").unwrap();
        auto_commit_worktree(&wt1, "w1").unwrap();
        cleanup_worktree(repo.path(), &wt1, branch1, true).unwrap();

        let wt2 = create_worktree(repo.path(), branch2).unwrap();
        std::fs::write(wt2.join("README.md"), "Worker 2 changes\n").unwrap();
        auto_commit_worktree(&wt2, "w2").unwrap();
        cleanup_worktree(repo.path(), &wt2, branch2, true).unwrap();

        // First merge should succeed
        let merged1 = merge_branch(repo.path(), branch1, &base).unwrap();
        assert!(merged1, "First merge should succeed");

        // Second merge should conflict
        let merged2 = merge_branch(repo.path(), branch2, &base).unwrap();
        assert!(!merged2, "Second merge should conflict");

        // Conflicting branch should still exist
        let output = std::process::Command::new("git")
            .args(["branch", "--list", branch2])
            .current_dir(repo.path())
            .output()
            .unwrap();
        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(
            branches.contains(branch2),
            "Conflicting branch should be preserved"
        );
    }

    #[test]
    fn test_manifest_with_git_roundtrip() {
        let repo = create_test_git_repo();

        // Create manifest in git repo
        let mut manifest = DispatchManifest::load(repo.path()).unwrap();
        assert!(manifest.dispatches.is_empty());

        // Simulate a dispatch
        let branch = "tl-dispatch/roundtrip-test";
        let wt_path = create_worktree(repo.path(), branch).unwrap();

        manifest.add_dispatch(DispatchRecord {
            id: "D-test-1".to_string(),
            created_at: now_rfc3339(),
            status: DispatchStatus::Pending,
            worker_count: 1,
            topic: "task.completed".to_string(),
            prefix: "worker".to_string(),
            branches: vec![BranchEntry {
                worker_name: "worker-1".to_string(),
                branch_name: branch.to_string(),
                base_branch: "main".to_string(),
                worktree_path: wt_path.to_string_lossy().to_string(),
                has_commits: false,
            }],
        });
        manifest.save(repo.path()).unwrap();

        // Verify manifest file was created
        let manifest_path = repo.path().join(MANIFEST_DIR).join(MANIFEST_FILE);
        assert!(manifest_path.exists(), "Manifest file should exist");

        // Reload and verify
        let reloaded = DispatchManifest::load(repo.path()).unwrap();
        assert_eq!(reloaded.dispatches.len(), 1);
        assert_eq!(reloaded.dispatches[0].status, DispatchStatus::Pending);

        cleanup_worktree(repo.path(), &wt_path, branch, false).unwrap();
    }

    #[test]
    fn test_multiple_worktrees_unique_branches() {
        let repo = create_test_git_repo();

        let branch1 = "tl-dispatch/D-1/w1";
        let branch2 = "tl-dispatch/D-1/w2";
        let branch3 = "tl-dispatch/D-1/w3";

        let wt1 = create_worktree(repo.path(), branch1).unwrap();
        let wt2 = create_worktree(repo.path(), branch2).unwrap();
        let wt3 = create_worktree(repo.path(), branch3).unwrap();

        // All should be unique directories
        assert_ne!(wt1, wt2);
        assert_ne!(wt2, wt3);
        assert_ne!(wt1, wt3);

        // All should exist
        assert!(wt1.exists());
        assert!(wt2.exists());
        assert!(wt3.exists());

        // Cleanup
        cleanup_worktree(repo.path(), &wt1, branch1, false).unwrap();
        cleanup_worktree(repo.path(), &wt2, branch2, false).unwrap();
        cleanup_worktree(repo.path(), &wt3, branch3, false).unwrap();
    }
}
