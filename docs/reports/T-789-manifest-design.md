# TermLink Dispatch Manifest Design (T-789)

## Overview

The dispatch manifest is a persistent YAML file located at `~/.termlink/dispatch-manifest.yaml` that tracks git worktree branches created by `termlink dispatch --isolate`. It serves as a single source of truth for dispatch lifecycle management across sessions, enabling:

- **Session isolation**: Tracking worktree branches spawned by individual dispatch operations
- **Merge coordination**: Recording when branches are merged back to main
- **Conflict resolution**: Tracking branches with merge conflicts for manual intervention
- **Cleanup automation**: Pruning stale or completed entries
- **Recovery**: Rebuilding dispatch state if worktree is deleted outside TermLink

---

## 1. YAML Schema

### Complete Schema Definition

```yaml
version: "1"  # Schema version for forward compatibility
last_updated: "2026-03-30T14:30:00Z"  # RFC3339 timestamp of last manifest update
dispatches:
  - id: "D-12345-1704067200000"  # Unique dispatch ID (PID-timestamp)
    created_at: "2026-03-30T10:00:00Z"  # When dispatch was created
    status: "pending"  # pending | merged | conflict | deferred | expired
    
    # Dispatch metadata
    worker_count: 3  # Number of workers spawned
    topic: "task.completed"  # Event topic being collected
    prefix: "worker"  # Worker name prefix
    
    # Worktree and branch tracking
    worktree_path: "/path/to/project/.git/worktrees/tl-dispatch-D-12345-1704067200000"
    branch_name: "tl-dispatch/D-12345-1704067200000"  # Git branch name
    base_branch: "main"  # Branch from which worktree was created
    
    # Status-specific fields
    merged_at: "2026-03-30T11:30:00Z"  # When status changed to merged
    conflict_detected_at: "2026-03-30T11:35:00Z"  # When merge conflict occurred
    conflict_reason: "Branch has unpushed changes"  # Human-readable conflict description
    deferred_reason: "Waiting for CI to pass"  # Why merge was deferred
    deferred_until: "2026-03-31T00:00:00Z"  # When to retry merge
    
    # Notes and tracking
    notes: "Manual intervention required: conflict in src/main.rs"
    last_status_check: "2026-03-30T14:00:00Z"  # Last time status was verified
    
    # Health indicators
    branch_exists: true  # Whether git branch still exists
    worktree_exists: true  # Whether worktree directory still exists
```

### Field Types and Valid Values

| Field | Type | Required | Valid Values | Description |
|-------|------|----------|--------------|-------------|
| `version` | string | yes | `"1"` | Schema version |
| `last_updated` | RFC3339 timestamp | yes | ISO 8601 datetime | Last manifest write time |
| `dispatches` | array | yes | Array of DispatchRecord | All dispatch records |
| `id` | string | yes | `D-[PID]-[MILLIS]` | Globally unique dispatch ID |
| `created_at` | RFC3339 timestamp | yes | ISO 8601 datetime | Creation time (Unix time) |
| `status` | enum | yes | `pending`, `merged`, `conflict`, `deferred`, `expired` | Current lifecycle status |
| `worker_count` | u32 | yes | ≥1 | Count of spawned workers |
| `topic` | string | yes | Event topic | Collection topic (e.g., `task.completed`) |
| `prefix` | string | yes | Alphanumeric + dash | Worker name prefix |
| `worktree_path` | path string | yes | Absolute path | Filesystem path to worktree |
| `branch_name` | string | yes | Valid git branch | Full branch name (e.g., `tl-dispatch/D-...`) |
| `base_branch` | string | yes | Valid git branch | Parent branch (usually `main`) |
| `merged_at` | RFC3339 timestamp | no | ISO 8601 datetime | Only set when status=`merged` |
| `conflict_detected_at` | RFC3339 timestamp | no | ISO 8601 datetime | Only set when status=`conflict` |
| `conflict_reason` | string | no | Free text | Reason for merge conflict |
| `deferred_reason` | string | no | Free text | Why merge was deferred |
| `deferred_until` | RFC3339 timestamp | no | ISO 8601 datetime | Retry time for deferred merges |
| `notes` | string | no | Free text | Human-readable notes |
| `last_status_check` | RFC3339 timestamp | no | ISO 8601 datetime | Last verification timestamp |
| `branch_exists` | bool | yes | `true`, `false` | Cached status of git branch |
| `worktree_exists` | bool | yes | `true`, `false` | Cached status of worktree directory |

### Status Lifecycle

```
┌─────────────┐
│   pending   │  ← Initial state when dispatch --isolate creates worktree
└──────┬──────┘
       │ (merge succeeds)
       ├──────────────────► merged ✓
       │
       │ (merge conflicts)
       ├──────────────────► conflict
       │                        │
       │                        └──► (manual fix + retry)
       │                             └──────► merged ✓
       │
       │ (CI/checks not ready)
       ├──────────────────► deferred
       │                        │
       │                        └──► (retry after deferred_until)
       │                             └──────► merged or conflict
       │
       │ (not touched for 7+ days)
       └──────────────────► expired
```

---

## 2. Rust Module Design

### Suggested Module Location

**Crate**: `termlink-cli`  
**Module path**: `crates/termlink-cli/src/manifest/mod.rs`  
**Sub-modules**:
- `crates/termlink-cli/src/manifest/schema.rs` — Data structures (serde)
- `crates/termlink-cli/src/manifest/manager.rs` — Load/save/update operations
- `crates/termlink-cli/src/manifest/error.rs` — Error types

### Module Hierarchy

```rust
// crates/termlink-cli/src/manifest/mod.rs
pub mod schema;
pub mod manager;
pub mod error;

pub use manager::DispatchManifest;
pub use schema::{DispatchRecord, BranchEntry, Status};
pub use error::ManifestError;
```

### Core Data Structures

```rust
// crates/termlink-cli/src/manifest/schema.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Root manifest structure persisted to YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchManifestData {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub dispatches: Vec<DispatchRecord>,
}

/// Represents a single dispatch operation and its tracking state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchRecord {
    /// Globally unique identifier: D-{PID}-{MILLIS}
    pub id: String,
    
    /// When the dispatch was created (Unix timestamp)
    pub created_at: DateTime<Utc>,
    
    /// Current lifecycle status
    pub status: Status,
    
    /// Metadata about the dispatch
    pub metadata: DispatchMetadata,
    
    /// Git and worktree tracking
    pub branch_entry: BranchEntry,
    
    /// Status-specific details
    #[serde(default)]
    pub status_details: StatusDetails,
    
    /// Notes and annotations
    #[serde(default)]
    pub notes: Option<String>,
    
    /// Last time status was checked from filesystem
    #[serde(default)]
    pub last_status_check: Option<DateTime<Utc>>,
}

/// Status enum with all valid lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Dispatch created, worktree active, awaiting merge
    Pending,
    
    /// Successfully merged back to base branch
    Merged,
    
    /// Merge conflict detected; manual intervention needed
    Conflict,
    
    /// Merge deferred pending CI/checks; retry scheduled
    Deferred,
    
    /// Not touched for 7+ days; marked for cleanup
    Expired,
}

/// Metadata about the dispatch spawning operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchMetadata {
    /// Number of workers spawned
    pub worker_count: u32,
    
    /// Event topic being collected
    pub topic: String,
    
    /// Worker name prefix (e.g., "worker")
    pub prefix: String,
}

/// Git worktree and branch tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchEntry {
    /// Absolute path to the worktree directory
    pub worktree_path: PathBuf,
    
    /// Full branch name (e.g., "tl-dispatch/D-...")
    pub branch_name: String,
    
    /// Parent branch (usually "main")
    pub base_branch: String,
    
    /// Cached: does git branch still exist?
    pub branch_exists: bool,
    
    /// Cached: does worktree directory still exist?
    pub worktree_exists: bool,
}

/// Status-specific details (only relevant fields are populated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusDetails {
    /// When merged (set only if status == Merged)
    pub merged_at: Option<DateTime<Utc>>,
    
    /// When conflict was detected (set only if status == Conflict)
    pub conflict_detected_at: Option<DateTime<Utc>>,
    
    /// Human-readable reason for conflict
    pub conflict_reason: Option<String>,
    
    /// Reason merge was deferred
    pub deferred_reason: Option<String>,
    
    /// When to retry a deferred merge
    pub deferred_until: Option<DateTime<Utc>>,
}

impl StatusDetails {
    pub fn new_conflict(reason: String) -> Self {
        Self {
            conflict_detected_at: Some(Utc::now()),
            conflict_reason: Some(reason),
            ..Default::default()
        }
    }
    
    pub fn new_deferred(reason: String, retry_at: DateTime<Utc>) -> Self {
        Self {
            deferred_reason: Some(reason),
            deferred_until: Some(retry_at),
            ..Default::default()
        }
    }
}
```

### Manager Interface

```rust
// crates/termlink-cli/src/manifest/manager.rs

use crate::manifest::schema::*;
use crate::manifest::error::ManifestError;
use std::path::PathBuf;
use chrono::{DateTime, Utc, Duration};

/// High-level manifest manager for lifecycle operations.
pub struct DispatchManifest {
    path: PathBuf,
    data: DispatchManifestData,
}

impl DispatchManifest {
    /// Load manifest from disk, creating empty if not present.
    ///
    /// # Errors
    ///
    /// Returns `ManifestError::Corrupt` if YAML is unparseable.
    pub fn load() -> Result<Self, ManifestError> {
        let path = Self::manifest_path();
        
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| ManifestError::Io(e))?;
            
            serde_yaml::from_str(&content)
                .map_err(|e| ManifestError::Corrupt(e.to_string()))?
        } else {
            // Initialize new manifest
            DispatchManifestData {
                version: "1".to_string(),
                last_updated: Utc::now(),
                dispatches: Vec::new(),
            }
        };
        
        Ok(DispatchManifest { path, data })
    }
    
    /// Save manifest to disk atomically (write-to-temp, then rename).
    ///
    /// # Errors
    ///
    /// Returns `ManifestError::Io` if write fails.
    pub fn save(&mut self) -> Result<(), ManifestError> {
        self.data.last_updated = Utc::now();
        
        let content = serde_yaml::to_string(&self.data)
            .map_err(|e| ManifestError::Serialize(e.to_string()))?;
        
        // Atomic write: create temp file, then rename
        let temp_path = self.path.with_extension("yaml.tmp");
        std::fs::write(&temp_path, &content)
            .map_err(|e| ManifestError::Io(e))?;
        
        std::fs::rename(&temp_path, &self.path)
            .map_err(|e| ManifestError::Io(e))?;
        
        Ok(())
    }
    
    /// Add a new dispatch record (called on dispatch --isolate create).
    pub fn add_dispatch(
        &mut self,
        dispatch_id: String,
        worker_count: u32,
        topic: String,
        prefix: String,
        worktree_path: PathBuf,
        branch_name: String,
        base_branch: String,
    ) -> &mut DispatchRecord {
        let record = DispatchRecord {
            id: dispatch_id,
            created_at: Utc::now(),
            status: Status::Pending,
            metadata: DispatchMetadata {
                worker_count,
                topic,
                prefix,
            },
            branch_entry: BranchEntry {
                worktree_path,
                branch_name,
                base_branch,
                branch_exists: true,
                worktree_exists: true,
            },
            status_details: StatusDetails::default(),
            notes: None,
            last_status_check: None,
        };
        
        self.data.dispatches.push(record);
        self.data.dispatches.last_mut().unwrap()
    }
    
    /// Find dispatch record by ID.
    pub fn find_dispatch(&mut self, id: &str) -> Option<&mut DispatchRecord> {
        self.data.dispatches.iter_mut().find(|d| d.id == id)
    }
    
    /// Update branch status (called by merge integration).
    pub fn update_branch_status(
        &mut self,
        dispatch_id: &str,
        new_status: Status,
        details: StatusDetails,
    ) -> Result<(), ManifestError> {
        let record = self.find_dispatch(dispatch_id)
            .ok_or_else(|| ManifestError::NotFound(dispatch_id.to_string()))?;
        
        record.status = new_status;
        record.status_details = details;
        record.last_status_check = Some(Utc::now());
        
        Ok(())
    }
    
    /// Get all pending dispatches (not yet merged/expired).
    pub fn pending_dispatches(&self) -> Vec<&DispatchRecord> {
        self.data.dispatches
            .iter()
            .filter(|d| d.status == Status::Pending)
            .collect()
    }
    
    /// Get all deferred dispatches ready for retry.
    pub fn deferred_ready_for_retry(&self) -> Vec<&DispatchRecord> {
        let now = Utc::now();
        self.data.dispatches
            .iter()
            .filter(|d| {
                d.status == Status::Deferred
                    && d.status_details.deferred_until
                        .map(|until| until <= now)
                        .unwrap_or(false)
            })
            .collect()
    }
    
    /// Find stale branches (not touched for N days).
    ///
    /// Returns records that should be marked as expired or cleanup scheduled.
    pub fn stale_dispatches(&self, days: i64) -> Vec<&DispatchRecord> {
        let cutoff = Utc::now() - Duration::days(days);
        self.data.dispatches
            .iter()
            .filter(|d| {
                let check_time = d.last_status_check.unwrap_or(d.created_at);
                check_time < cutoff && d.status != Status::Merged
            })
            .collect()
    }
    
    /// Verify branch/worktree existence and update cached flags.
    ///
    /// Shells out to `git` to check if branch exists. Called periodically.
    pub fn refresh_branch_status(&mut self, dispatch_id: &str) -> Result<(), ManifestError> {
        let record = self.find_dispatch(dispatch_id)
            .ok_or_else(|| ManifestError::NotFound(dispatch_id.to_string()))?;
        
        // Check if git branch still exists
        let branch_exists = std::process::Command::new("git")
            .args(["rev-parse", "--verify", &record.branch_entry.branch_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        
        // Check if worktree directory exists
        let worktree_exists = record.branch_entry.worktree_path.exists();
        
        record.branch_entry.branch_exists = branch_exists;
        record.branch_entry.worktree_exists = worktree_exists;
        record.last_status_check = Some(Utc::now());
        
        Ok(())
    }
    
    /// Remove a dispatch record (called on cleanup).
    pub fn remove_dispatch(&mut self, dispatch_id: &str) -> Result<(), ManifestError> {
        self.data.dispatches.retain(|d| d.id != dispatch_id);
        Ok(())
    }
    
    /// Prune completed/expired entries older than N days, keeping latest M entries.
    ///
    /// Strategy: Remove entries where:
    /// - status == Merged AND merged_at < (now - prune_days)
    /// - status == Expired AND created_at < (now - prune_days)
    ///
    /// Always keep at least min_keep most recent entries.
    pub fn cleanup_merged(
        &mut self,
        prune_days: i64,
        min_keep: usize,
    ) -> Result<(), ManifestError> {
        let cutoff = Utc::now() - Duration::days(prune_days);
        let initial_count = self.data.dispatches.len();
        
        self.data.dispatches.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // Mark for deletion
        let to_remove: Vec<String> = self.data.dispatches
            .iter()
            .skip(min_keep)
            .filter(|d| {
                (d.status == Status::Merged
                    && d.status_details.merged_at
                        .map(|t| t < cutoff)
                        .unwrap_or(false))
                    || (d.status == Status::Expired
                        && d.created_at < cutoff)
            })
            .map(|d| d.id.clone())
            .collect();
        
        for id in &to_remove {
            self.data.dispatches.retain(|d| &d.id != id);
        }
        
        if to_remove.len() > 0 {
            tracing::info!("Manifest cleanup: {} -> {} entries", initial_count, self.data.dispatches.len());
        }
        
        Ok(())
    }
    
    /// Return the path to the manifest file.
    fn manifest_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        std::path::PathBuf::from(home)
            .join(".termlink")
            .join("dispatch-manifest.yaml")
    }
}
```

### Error Types

```rust
// crates/termlink-cli/src/manifest/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("manifest file corrupt or unparseable: {0}")]
    Corrupt(String),
    
    #[error("failed to serialize manifest: {0}")]
    Serialize(String),
    
    #[error("dispatch record not found: {0}")]
    NotFound(String),
    
    #[error("manifest lock timeout (concurrent writes?)")]
    LockTimeout,
    
    #[error("dispatch metadata error: {0}")]
    Metadata(String),
}
```

---

## 3. Integration Points

### 3.1 Dispatch Create (`dispatch.rs`)

**Where**: `termlink dispatch --isolate` command handler  
**When**: After worktree is successfully created  
**What**: Record new dispatch in manifest

```rust
// In crates/termlink-cli/src/commands/dispatch.rs

pub(crate) async fn cmd_dispatch(
    // ... existing params ...
    isolate: bool,  // NEW: if true, create worktree
) -> Result<()> {
    // ... existing dispatch setup ...
    
    let dispatch_id = format!("D-{}-{}", ...);
    
    let (worktree_path, branch_name) = if isolate {
        // Create git worktree with isolated branch
        let wt_path = create_worktree_for_dispatch(&dispatch_id)?;
        let branch = format!("tl-dispatch/{dispatch_id}");
        (wt_path, branch)
    } else {
        // Original behavior: run in current directory
        (std::env::current_dir()?, "".to_string())
    };
    
    // Record in manifest
    if isolate {
        let mut manifest = DispatchManifest::load()?;
        manifest.add_dispatch(
            dispatch_id.clone(),
            count,
            topic.to_string(),
            prefix.clone(),
            worktree_path,
            branch_name,
            "main".to_string(),  // TODO: detect base branch
        );
        manifest.save()?;
    }
    
    // ... rest of dispatch flow ...
}
```

### 3.2 Merge Handler Integration

**Where**: New command `termlink dispatch merge <dispatch-id>` or hook in git post-merge  
**When**: When user initiates merge or merge succeeds automatically  
**What**: Update manifest with merge result

```rust
// In crates/termlink-cli/src/commands/dispatch.rs or new merge.rs

pub(crate) async fn cmd_dispatch_merge(
    dispatch_id: String,
    auto_commit: bool,
) -> Result<()> {
    let mut manifest = DispatchManifest::load()?;
    let record = manifest.find_dispatch(&dispatch_id)
        .ok_or_else(|| anyhow::anyhow!("Dispatch not found"))?
        .clone();
    
    // Attempt merge: dispatch branch -> base branch
    let branch = &record.branch_entry.branch_name;
    let base = &record.branch_entry.base_branch;
    
    let merge_result = std::process::Command::new("git")
        .args(["merge", branch])
        .status()?;
    
    if merge_result.success() {
        // Success: update to Merged
        manifest.update_branch_status(
            &dispatch_id,
            Status::Merged,
            StatusDetails {
                merged_at: Some(Utc::now()),
                ..Default::default()
            },
        )?;
    } else {
        // Conflict detected
        manifest.update_branch_status(
            &dispatch_id,
            Status::Conflict,
            StatusDetails::new_conflict(
                "Merge conflict: manual resolution required".to_string()
            ),
        )?;
    }
    
    manifest.save()?;
    Ok(())
}
```

### 3.3 Pre-commit Hook Integration

**Where**: Git pre-commit hook (`.git/hooks/pre-commit`)  
**When**: Before each commit  
**What**: Check if current worktree matches a dispatch; warn if status is conflict

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if we're in a dispatch worktree
DISPATCH_ID=$(basename "$(git rev-parse --git-dir)" | sed 's/^worktrees\///')

if [[ "$DISPATCH_ID" == tl-dispatch-* ]]; then
    # Look up dispatch in manifest
    manifest="$HOME/.termlink/dispatch-manifest.yaml"
    if [ -f "$manifest" ]; then
        status=$(grep -A 2 "id: $DISPATCH_ID" "$manifest" | grep "status:" | awk '{print $2}')
        
        if [ "$status" = "conflict" ]; then
            echo "WARNING: This dispatch has a merge conflict. Check manifest and resolve conflicts before committing."
            # Optionally exit 1 to prevent commit:
            # exit 1
        fi
    fi
fi
```

---

## 4. Edge Cases and Handling

### 4.1 Manifest File Doesn't Exist (First Dispatch)

**Scenario**: User runs `termlink dispatch --isolate` for the first time.

**Handling**:
- `DispatchManifest::load()` checks if manifest path exists
- If not, initializes empty `DispatchManifestData` with version="1" and empty dispatches array
- Creates `~/.termlink/` directory if needed (with `0o700` perms for security)
- On first `add_dispatch()`, manifest is saved with initial entry

**Code**:
```rust
pub fn load() -> Result<Self, ManifestError> {
    let path = Self::manifest_path();
    
    if path.exists() {
        // Load from disk
    } else {
        // Initialize empty manifest
        std::fs::create_dir_all(path.parent().unwrap())?;
        let data = DispatchManifestData {
            version: "1".to_string(),
            last_updated: Utc::now(),
            dispatches: Vec::new(),
        };
        Ok(DispatchManifest { path, data })
    }
}
```

### 4.2 Manifest Is Corrupt/Unparseable

**Scenario**: Manifest file is corrupted (truncated, invalid YAML, old schema).

**Handling**:
- `serde_yaml::from_str()` fails → `ManifestError::Corrupt`
- Log error with file path for debugging
- **Option A** (strict): Exit with error, ask user to manually fix or delete file
- **Option B** (lenient): Back up corrupted file, initialize fresh manifest, log warning

**Recommended**: Option B with backup

```rust
pub fn load() -> Result<Self, ManifestError> {
    let path = Self::manifest_path();
    
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        match serde_yaml::from_str::<DispatchManifestData>(&content) {
            Ok(data) => Ok(DispatchManifest { path, data }),
            Err(e) => {
                // Back up corrupted file
                let backup = path.with_extension("yaml.corrupt");
                std::fs::rename(&path, &backup)
                    .ok();
                
                tracing::warn!(
                    "Manifest corrupted (backed up to {}): {}. Starting fresh.",
                    backup.display(),
                    e
                );
                
                // Initialize fresh
                let data = DispatchManifestData {
                    version: "1".to_string(),
                    last_updated: Utc::now(),
                    dispatches: Vec::new(),
                };
                Ok(DispatchManifest { path, data })
            }
        }
    } else {
        // ... initialize empty ...
    }
}
```

### 4.3 Branch Was Deleted Outside TermLink

**Scenario**: User runs `git branch -D tl-dispatch/D-xxx` directly, not via TermLink.

**Handling**:
- Manifest still references the branch in the record
- `refresh_branch_status()` detects `branch_exists = false`
- Options:
  1. **Mark as orphaned**: Keep record but set `branch_exists = false`, status → `Expired`
  2. **Prompt user**: If trying to merge, detect missing branch and offer cleanup
  3. **Auto-cleanup**: On periodic cleanup runs, remove records with `branch_exists = false` and status ≠ `Pending`

**Recommended approach**:
```rust
pub fn refresh_branch_status(&mut self, dispatch_id: &str) -> Result<(), ManifestError> {
    let record = self.find_dispatch(dispatch_id)...?;
    
    let branch_exists = git_branch_exists(&record.branch_entry.branch_name)?;
    record.branch_entry.branch_exists = branch_exists;
    
    // If branch was deleted externally, mark as expired
    if !branch_exists && record.status == Status::Pending {
        record.status = Status::Expired;
        tracing::warn!(
            "Dispatch {} branch was deleted externally",
            dispatch_id
        );
    }
    
    record.last_status_check = Some(Utc::now());
    Ok(())
}
```

### 4.4 Concurrent Dispatches Writing to Manifest

**Scenario**: Two `termlink dispatch --isolate` commands run simultaneously, both trying to write to manifest.

**Handling**:
1. **Read-Modify-Write Pattern**: Load manifest, add record, write back
2. **Atomic writes**: Use temp file + rename to avoid partial writes
3. **Lock file (optional)**: For stricter serialization, use `~/.termlink/dispatch-manifest.lock` with timeout

**Basic implementation (temp+rename, good enough for POSIX)**:
```rust
pub fn save(&mut self) -> Result<(), ManifestError> {
    self.data.last_updated = Utc::now();
    let content = serde_yaml::to_string(&self.data)?;
    
    // Atomic write: write to temp, then rename
    let temp_path = self.path.with_extension("yaml.tmp");
    std::fs::write(&temp_path, &content)?;
    std::fs::rename(&temp_path, &self.path)?;
    
    Ok(())
}
```

**For stronger guarantees (if needed)**:
```rust
fn with_manifest_lock<F, R>(f: F) -> Result<R, ManifestError>
where
    F: FnOnce(&mut DispatchManifest) -> Result<R, ManifestError>,
{
    let lock_path = Self::manifest_path().with_extension("lock");
    
    // Try to acquire lock (with timeout)
    for attempt in 0..10 {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => {
                // Lock acquired
                let mut manifest = DispatchManifest::load()?;
                let result = f(&mut manifest)?;
                manifest.save()?;
                let _ = std::fs::remove_file(&lock_path);
                return Ok(result);
            }
            Err(_) if attempt < 9 => {
                // Lock exists; wait and retry
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => {
                return Err(ManifestError::LockTimeout);
            }
        }
    }
    
    Err(ManifestError::LockTimeout)
}
```

### 4.5 Manifest Grows Unbounded

**Scenario**: After months of use, manifest has 10,000+ old dispatch records, becomes slow to load/parse.

**Handling**: Implement `cleanup_merged()` strategy

---

## 5. File Size Management

### Strategy: Hybrid Approach

Combine **time-based pruning** with **entry-count keeping**.

```rust
/// Pruning policy
pub struct PrunePolicy {
    /// Remove merged entries older than this many days
    pub prune_merged_days: i64,
    
    /// Remove expired entries older than this many days
    pub prune_expired_days: i64,
    
    /// Always keep at least this many most recent entries (regardless of age)
    pub min_keep_recent: usize,
    
    /// Maximum entries to keep before forcing cleanup (hard limit)
    pub max_entries: usize,
}

impl Default for PrunePolicy {
    fn default() -> Self {
        Self {
            prune_merged_days: 7,
            prune_expired_days: 3,
            min_keep_recent: 50,
            max_entries: 1000,
        }
    }
}

pub fn cleanup_merged_and_expired(
    &mut self,
    policy: &PrunePolicy,
) -> Result<(), ManifestError> {
    let cutoff_merged = Utc::now() - Duration::days(policy.prune_merged_days);
    let cutoff_expired = Utc::now() - Duration::days(policy.prune_expired_days);
    
    // Sort by created_at descending (newest first)
    self.data.dispatches.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    // Mark entries for removal
    let mut to_remove = Vec::new();
    for (idx, record) in self.data.dispatches.iter().enumerate() {
        // Always keep min_keep_recent
        if idx < policy.min_keep_recent {
            continue;
        }
        
        // Remove merged entries older than threshold
        if record.status == Status::Merged {
            if let Some(merged_at) = record.status_details.merged_at {
                if merged_at < cutoff_merged {
                    to_remove.push(record.id.clone());
                }
            }
        }
        
        // Remove expired entries older than threshold
        if record.status == Status::Expired && record.created_at < cutoff_expired {
            to_remove.push(record.id.clone());
        }
    }
    
    // Apply removals
    for id in &to_remove {
        self.data.dispatches.retain(|d| &d.id != id);
    }
    
    // If still above max, force removal of oldest (keeping min_keep_recent)
    if self.data.dispatches.len() > policy.max_entries {
        let to_remove = self.data.dispatches.len() - policy.max_entries;
        self.data.dispatches = self.data.dispatches[..policy.max_entries].to_vec();
        tracing::warn!(
            "Manifest exceeded max entries ({}); forcefully pruned {} records",
            policy.max_entries,
            to_remove
        );
    }
    
    if to_remove.len() > 0 {
        tracing::info!(
            "Manifest pruned {} entries; {} remaining",
            to_remove.len(),
            self.data.dispatches.len()
        );
    }
    
    Ok(())
}
```

### Cleanup Trigger Points

1. **On manifest load** (periodic check):
   - If entry count > 500, run cleanup with default policy
   
2. **Explicit command**:
   - `termlink dispatch cleanup [--policy aggressive|default|conservative]`
   
3. **On dispatch merge** (eager cleanup):
   - Run lightweight cleanup to remove completed entries
   
4. **Cron job** (optional):
   - Run cleanup daily: `0 2 * * * termlink dispatch cleanup --policy aggressive`

### Sizing Estimates

| Entry Count | YAML Size | Parse Time |
|-------------|-----------|-----------|
| 100 | ~8 KB | <1 ms |
| 500 | ~40 KB | ~5 ms |
| 1,000 | ~80 KB | ~10 ms |
| 5,000 | ~400 KB | ~50 ms |
| 10,000 | ~800 KB | ~100 ms |

**Recommendation**: Keep manifest under 500-1000 entries via default cleanup policy (7-day retention for merged, 3-day for expired, keep last 50).

---

## Summary

### Key Design Decisions

1. **YAML format**: Human-readable, same tech stack as existing hubs.toml
2. **Atomic writes**: Temp file + rename prevents partial writes
3. **Status enum**: Clear lifecycle (pending → merged/conflict/deferred/expired)
4. **Dual caching**: Track both `branch_exists` and `worktree_exists` flags
5. **Hybrid pruning**: Time-based + entry-count limits prevent unbounded growth
6. **Module placement**: `termlink-cli` crate (no need for new crate)
7. **Lock strategy**: Basic temp+rename sufficient; optional advisory lock file for stricter needs

### Files to Create/Modify

- Create: `crates/termlink-cli/src/manifest/mod.rs`
- Create: `crates/termlink-cli/src/manifest/schema.rs`
- Create: `crates/termlink-cli/src/manifest/manager.rs`
- Create: `crates/termlink-cli/src/manifest/error.rs`
- Modify: `crates/termlink-cli/src/commands/dispatch.rs` (add manifest calls)
- Modify: `crates/termlink-cli/src/commands/mod.rs` (export manifest module)
- Create: `.git/hooks/pre-commit` (optional, for dispatch-aware hook)
- Modify: `Cargo.toml` workspace (add serde_yaml, chrono if not already present)

### Dependencies to Add

```toml
# crates/termlink-cli/Cargo.toml
serde_yaml = "0.9"  # Already using serde, need YAML serialization
chrono = { version = "0.4", features = ["serde"] }  # Timestamp handling
```
