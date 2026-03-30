# T-789: Worktree Isolation — Test Plan

**Components under test:**
1. Dispatch manifest (YAML file tracking worktree branches)
2. `--isolate` flag on `termlink dispatch`
3. `--auto-merge` flag
4. Pre-commit hook gate
5. Audit check

---

## Unit Tests (cargo test, no I/O)

### Manifest CRUD (12 tests)

| Test | Verifies |
|------|----------|
| `test_manifest_load_empty` | Non-existent file returns empty manifest |
| `test_manifest_load_existing` | YAML with 3 entries (pending, merged, conflict) parses correctly |
| `test_manifest_add_dispatch_entry` | New entry defaults to status: pending |
| `test_manifest_update_status_pending_to_merged` | Valid transition with merged_at timestamp |
| `test_manifest_update_status_pending_to_conflict` | Valid transition with conflict reason |
| `test_manifest_update_status_conflict_to_deferred` | User acknowledges conflict |
| `test_manifest_update_status_pending_to_expired` | Branch older than TTL auto-expires |
| `test_manifest_prune_merged_entries` | Only merged entries removed, pending preserved |
| `test_manifest_prune_expired_entries` | Only expired entries removed |
| `test_manifest_handle_corrupt_yaml` | Corrupt YAML returns error, not panic |
| `test_manifest_handle_unknown_status` | Unknown status rejected or defaults to pending |
| `test_manifest_serialization_roundtrip` | Serialize→deserialize produces identical structure |

### Status Transitions (6 tests)

| Test | Verifies |
|------|----------|
| `test_transition_valid_pending_to_merged` | Happy path merge |
| `test_transition_valid_pending_to_conflict` | Merge fails |
| `test_transition_valid_conflict_to_deferred` | Human defers conflict |
| `test_transition_invalid_merged_to_pending` | Merged can't revert |
| `test_transition_invalid_expired_to_pending` | Expired can't revive |
| `test_transition_invalid_skip_conflict` | Must acknowledge conflicts |

### Stale Detection (5 tests)

| Test | Verifies |
|------|----------|
| `test_stale_branch_older_than_threshold` | 10-day branch with 7-day TTL = stale |
| `test_fresh_branch_younger_than_threshold` | 2-day branch = not stale |
| `test_boundary_exactly_at_threshold` | Edge case at exact TTL |
| `test_mixed_fresh_and_stale` | Filter identifies only stale subset |
| `test_merged_branches_not_counted_as_stale` | Merged handled by prune, not staleness |

---

## Integration Tests (need git repo)

### Worktree Lifecycle (7 tests)

| Test | Verifies |
|------|----------|
| `test_dispatch_isolate_creates_worktree` | Worktree created, manifest entry added |
| `test_dispatch_isolate_creates_multiple_worktrees` | 3 unique worktrees, 3 manifest entries |
| `test_dispatch_isolate_unique_branch_names` | No branch name collisions |
| `test_dispatch_isolate_worker_commits_changes` | Auto-commit on branch, main unchanged |
| `test_dispatch_isolate_no_changes_no_commit` | No empty commits for idle workers |
| `test_manifest_written_before_worker_starts` | Manifest exists BEFORE worker dispatch |
| `test_manifest_persists_after_worker_failure` | Failed worker still tracked |

### Auto-Merge (5 tests)

| Test | Verifies |
|------|----------|
| `test_auto_merge_single_worker` | Branch merged, manifest=merged, branch deleted |
| `test_auto_merge_multiple_workers` | Sequential merge, both branches landed |
| `test_auto_merge_conflict_detected` | Worker 1 merges, worker 2 records conflict |
| `test_auto_merge_conflict_records_in_manifest` | Conflict state persisted (not just stderr) |
| `test_auto_merge_deterministic_order` | Workers merged in consistent order |

### Pre-Commit Hook (5 tests)

| Test | Verifies |
|------|----------|
| `test_hook_blocks_on_pending_branches` | Commit blocked, message shown |
| `test_hook_blocks_on_conflict_branches` | Commit blocked for conflicts too |
| `test_hook_allows_when_clean` | Merged entries don't block |
| `test_hook_allows_when_no_manifest` | First-run case (no dispatch yet) |
| `test_hook_blocks_on_every_commit` | Not a one-time check |

### Dispatch CLI (2 tests)

| Test | Verifies |
|------|----------|
| `test_dispatch_status_reads_manifest` | Shows counts: pending, merged, conflict |
| `test_dispatch_status_empty` | "No pending dispatches" |

---

## Regression Tests (5 failure modes)

### FM-1: Session Compaction (3 tests)

| Test | Verifies |
|------|----------|
| `test_regression_compaction_manifest_survives` | Manifest on disk, readable by fresh session |
| `test_regression_compaction_dispatch_status_works` | `dispatch status` in new session finds branches |
| `test_regression_compaction_hook_still_blocks` | Pre-commit hook reads disk, not memory |

### FM-2: Budget Exhaustion (2 tests)

| Test | Verifies |
|------|----------|
| `test_regression_budget_manifest_written_before_merge` | Manifest flushed before auto-merge starts |
| `test_regression_budget_partial_merge_recorded` | Kill mid-merge: merged branches tracked, pending ones remain |

### FM-3: Human Moves On (2 tests)

| Test | Verifies |
|------|----------|
| `test_regression_human_hook_blocks_unrelated_commits` | Can't commit new work with pending branches |
| `test_regression_human_hook_message_actionable` | Message tells user exactly what to run |

### FM-4: Merge Conflict (3 tests)

| Test | Verifies |
|------|----------|
| `test_regression_conflict_recorded_in_manifest` | Not just stderr — persistent state |
| `test_regression_conflict_branch_preserved` | Branch not deleted, can be manually merged |
| `test_regression_conflict_audit_reports_it` | `dispatch status` shows conflict count |

### FM-5: Accumulation (4 tests)

| Test | Verifies |
|------|----------|
| `test_regression_accumulation_audit_counts` | Counts pending across multiple dispatches |
| `test_regression_accumulation_warns_at_threshold` | Warn at 3 pending |
| `test_regression_accumulation_fails_at_threshold` | Fail at 10 pending |
| `test_regression_accumulation_counts_only_pending` | Merged/expired/deferred not counted |

---

## Edge Case Tests (8 tests)

| Test | Verifies |
|------|----------|
| `test_edge_zero_workers` | Dispatch rejects count=0 |
| `test_edge_one_worker_no_merge_needed` | Single worker: degenerate merge succeeds |
| `test_edge_manifest_deleted_mid_dispatch` | Dispatch recreates manifest, no crash |
| `test_edge_bare_git_repo` | Graceful error: "Cannot create worktree in bare repo" |
| `test_edge_non_git_directory` | Graceful error: "Not a git repository" |
| `test_edge_many_workers` | 50 worktrees: no fd limits, all cleaned up |
| `test_edge_different_files_no_conflict` | 3 workers editing different files all merge cleanly |
| `test_edge_same_file_non_overlapping` | 2 workers editing different lines merge cleanly |

---

## Summary

| Category | Count |
|----------|-------|
| Unit: Manifest CRUD | 12 |
| Unit: Status Transitions | 6 |
| Unit: Stale Detection | 5 |
| Integration: Worktree Lifecycle | 7 |
| Integration: Auto-Merge | 5 |
| Integration: Pre-Commit Hook | 5 |
| Integration: Dispatch CLI | 2 |
| Regression: 5 Failure Modes | 14 |
| Edge Cases | 8 |
| **Total** | **64** |

Every failure mode from the research (FM-1 through FM-5) has dedicated regression tests that verify the deterministic mitigation works.
