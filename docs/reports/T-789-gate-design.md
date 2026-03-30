# T-789: Dispatch Manifest Branch Age Gate — Design Document

## Overview

This document specifies the pre-commit hook integration for monitoring and gating branches tracked in TermLink's dispatch manifest (`.termlink/dispatch-manifest.yaml`). The gate prevents commits when pending dispatch branches exceed an age threshold (default 30 minutes), protecting against stale branch accumulation that could cause dispatch protocol violations.

## 1. Hook Placement & Architecture

### Primary Hook: Pre-Commit via PreToolUse on Bash

**Location:** `.agentic-framework/agents/context/check-dispatch-manifest.sh` (new file)

**Type:** PreToolUse matcher on Bash tool with `git commit` pattern

**Why Pre-Commit (not audit.sh)?**

1. **Timing:** Blocks BEFORE the commit is made, not after (audit fires on pre-push)
2. **Scope:** Only blocks the specific action (commit), not the entire push operation
3. **User Experience:** Tighter feedback loop — developer sees block immediately when attempting commit
4. **Consistency:** Follows existing pattern of `check-tier0.sh` (PreToolUse Bash gate)
5. **Audit Integration:** Secondary audit section provides ongoing visibility without blocking

### Secondary Hook: Audit Section in audit.sh

**Location:** `.agentic-framework/agents/audit/audit.sh` (new section: `dispatch-manifest`)

**Type:** Non-blocking audit check (advisory output only)

**Purpose:** Track branch health trends over time, not enforce gate

## 2. Check Logic — Pseudocode

```
function check-dispatch-manifest-age():
    # Load manifest or pass if missing
    manifest_file = ".termlink/dispatch-manifest.yaml"
    if not exists(manifest_file):
        return PASS  # No manifest yet — bootstrap case
    
    try:
        manifest = parse_yaml(manifest_file)
    except:
        log_warning("Manifest corrupt — skipping check")
        return PASS  # Don't block on corruption
    
    if manifest is empty or null:
        return PASS  # Empty manifest — nothing to check
    
    # Extract pending branches with timestamps
    pending_branches = []
    threshold_minutes = env("TERMLINK_MANIFEST_AGE_THRESHOLD", "30")
    current_time = now_unix()
    
    for branch in manifest.get("branches", []):
        if branch.get("status") != "pending":
            continue  # Skip non-pending
        
        created_at = branch.get("created_at")  # ISO8601 or Unix timestamp
        if not created_at:
            continue  # Skip branches with no timestamp
        
        branch_age_minutes = (current_time - parse_timestamp(created_at)) / 60
        
        if branch_age_minutes > threshold_minutes:
            pending_branches.append({
                "name": branch.get("name"),
                "created_at": created_at,
                "age_minutes": branch_age_minutes
            })
    
    # Decision logic
    if len(pending_branches) == 0:
        return ALLOW  # No stale branches
    
    # Stale branches found — block with guidance
    return BLOCK_WITH_MESSAGE(pending_branches, threshold_minutes)
```

## 3. User Experience — Block Message

When stale branches are detected, the hook outputs:

```
══════════════════════════════════════════════════════════
  DISPATCH MANIFEST GATE — Stale Pending Branches
══════════════════════════════════════════════════════════

  Threshold: 30 minutes

  Stale branches found:
    • branch-001-feature: 45 minutes old (created 2026-03-30T14:15:00Z)
    • branch-002-docs:    62 minutes old (created 2026-03-30T13:58:00Z)

  Pending dispatch branches block commits to prevent accumulation
  of abandoned or stuck branches in the manifest.

  To unblock, choose one:

    1. ACKNOWLEDGE & CONTINUE:
       termlink dispatch defer branch-001-feature branch-002-docs
       
       (Sets branches to 'deferred' status — no longer pending)

    2. COMPLETE THE DISPATCH:
       termlink dispatch merge branch-001-feature
       
       (Merges and removes from manifest)

    3. CHECK MANIFEST STATUS:
       termlink dispatch status
       
       (Lists all tracked branches and their status)

    4. PRUNE EXPIRED/MERGED ENTRIES:
       termlink dispatch clean
       
       (Removes completed or expired entries)

  If you're confident this is a false positive, bypass with:
    FW_MANIFEST_BYPASS=1 git commit -m "..."

  Policy: T-789 (Dispatch Manifest Branch Age Gate)
  Framework: Agentic Engineering Framework

══════════════════════════════════════════════════════════
```

**Bypass mechanism:**
```bash
FW_MANIFEST_BYPASS=1 git commit -m "..."
```

This env var is checked early in the hook to allow explicit override when needed.

## 4. Bypass Patterns — Commands

The gate integrates with the `termlink dispatch` CLI. All bypass operations acknowledge the stale branch by changing its status, preventing the same branch from re-blocking.

### Pattern A: Defer — Acknowledge and Stop Blocking

```bash
termlink dispatch defer <branch> [branch...]
```

**Effect:** Sets `status: deferred` on listed branches

**Manifest before:**
```yaml
branches:
  - name: branch-001-feature
    status: pending
    created_at: 2026-03-30T14:15:00Z
    worker_id: agent-45
```

**Manifest after:**
```yaml
branches:
  - name: branch-001-feature
    status: deferred
    created_at: 2026-03-30T14:15:00Z
    deferred_at: 2026-03-30T14:50:00Z
    worker_id: agent-45
```

**Use case:** "I see this branch exists but I'm not working on it right now. Stop blocking commits on it."

### Pattern B: Merge — Complete the Dispatch

```bash
termlink dispatch merge <branch>
```

**Effect:** 
1. Merges the branch into main (or configured target)
2. Removes entry from manifest
3. Logs completion to `.context/dispatch-log.yaml`

**Use case:** "This dispatch is done. Merge it and clean up."

### Pattern C: Status — Inspect All Tracked Branches

```bash
termlink dispatch status
```

**Output:**
```
Dispatch Manifest Status
========================

  Pending (blocks commits):
    • branch-001-feature: 45 min old, worker: agent-45
    • branch-002-docs:    62 min old, worker: agent-23

  Deferred (acknowledged, no block):
    • branch-003-refactor: 8 hours old (deferred 30 min ago), worker: agent-12

  Merged/Complete:
    • branch-000-init: completed 3 hours ago

  Threshold: 30 minutes
  Next audit: 2026-03-30T14:55:00Z
```

**Use case:** "What branches am I tracking and what's their status?"

### Pattern D: Clean — Prune Expired/Merged Entries

```bash
termlink dispatch clean
```

**Effect:**
1. Removes all `status: merged` entries older than 24 hours
2. Removes all `status: abandoned` entries older than 7 days
3. Archives removed entries to `.context/dispatch-log.yaml`
4. Reports what was pruned

**Output:**
```
Dispatch Manifest Cleanup
=========================

  Pruned:
    • branch-000-init (merged, 3 days old)
    • branch-004-experiment (abandoned, 14 days old)

  Manifest size before: 12 branches
  Manifest size after:  10 branches
```

**Use case:** "Clean up old entries to keep the manifest readable."

## 5. Audit Integration — New Section in audit.sh

### Audit Section: `dispatch-manifest`

**File:** `.agentic-framework/agents/audit/audit.sh` (new section)

**Trigger:** Part of default audit (or invoked with `--section dispatch-manifest`)

**Output (YAML report):**

```yaml
dispatch_manifest:
  status: PASS  # or WARN/FAIL
  checked_at: 2026-03-30T14:52:00Z
  
  summary:
    total_branches: 12
    pending_count: 2
    pending_age_range: "45-62 minutes"
    deferred_count: 3
    merged_count: 5
    abandoned_count: 1
  
  pending_branches:
    - name: branch-001-feature
      created_at: 2026-03-30T14:15:00Z
      age_minutes: 45
      worker_id: agent-45
      status: pending
      
    - name: branch-002-docs
      created_at: 2026-03-30T13:58:00Z
      age_minutes: 62
      worker_id: agent-23
      status: pending
  
  threshold_minutes: 30
  gate_status: ACTIVE
  last_bypass: null
  
  recommendations:
    - "2 branches exceed threshold — run 'termlink dispatch status' to review"
    - "Consider merging or deferring branches idle >1 hour"
```

**Terminal output (from audit.sh):**

```
[dispatch-manifest] 12 tracked branches
  ✓ Pending: 2 (45-62 min old — check status)
  ✓ Deferred: 3 (acknowledged, no action needed)
  ✓ Merged: 5 (archive cleanup recommended)
  ⓘ Abandoned: 1 (>7 days, consider removal with 'termlink dispatch clean')
```

### Integration into audit schedule

Add to `.context/cron/agentic-audit.crontab`:

```bash
# Dispatch manifest health (every 30 min with structural audits)
*/30 * * * * root PROJECT_ROOT="$PROJECT_ROOT" "$FW_PATH" audit --section dispatch-manifest --cron
```

## 6. Edge Cases & Handling

### Case A: Manifest File Missing

**Condition:** `.termlink/dispatch-manifest.yaml` doesn't exist

**Behavior:** PASS (allow commit)

**Rationale:** Bootstrap case — project doesn't track branches yet

**Log:** Silent pass, no warning

### Case B: Manifest Corrupt (YAML parse error)

**Condition:** File exists but contains invalid YAML

**Behavior:** PASS (allow commit) + WARNING to stderr

**Output:**
```
MANIFEST WARNING: Failed to parse .termlink/dispatch-manifest.yaml
  Error: [yaml parse error details]
  Action: Check manifest format or run 'termlink dispatch init' to reset
  Gate: Bypassed (will not block on corrupt manifest)
```

**Rationale:** Don't block developer work for manifest corruption — let them commit and fix separately

### Case C: Manifest Empty or Null

**Condition:** File exists but contains empty dict `{}` or no branches list

**Behavior:** PASS (allow commit)

**Log:** Silent pass

**Rationale:** No branches tracked yet — nothing to enforce

### Case D: Branch Has No Timestamp

**Condition:** Manifest has a branch entry without `created_at` field

**Behavior:** Skip that branch (don't count toward threshold check)

**Log:** Optional debug: "Skipping branch XYZ (no created_at timestamp)"

**Rationale:** Can't age-check without timestamp — don't fail on malformed entries

### Case E: Timestamp Format Variations

**Supported formats:**
- ISO8601 with Z: `2026-03-30T14:15:00Z`
- ISO8601 with +HH:MM: `2026-03-30T14:15:00+00:00`
- Unix timestamp (seconds): `1743379500`

**Parsing:** Use Python's `dateutil.parser.parse()` with fallback to `int()` for Unix timestamps

**Case F: Threshold Override**

**Mechanism:** Environment variable `TERMLINK_MANIFEST_AGE_THRESHOLD`

**Example:**
```bash
# Allow stale branches up to 60 minutes
TERMLINK_MANIFEST_AGE_THRESHOLD=60 git commit -m "..."
```

**Use case:** Temporary extended deadline for known long-running dispatches

### Case G: No .termlink Directory

**Condition:** `.termlink/` doesn't exist

**Behavior:** PASS (silently)

**Rationale:** TermLink not initialized in project

## 7. Implementation Files

### File 1: `.agentic-framework/agents/context/check-dispatch-manifest.sh` (new)

**Purpose:** PreToolUse hook that checks manifest before commit

**Size:** ~150 lines (includes YAML parsing, age checking, block messaging)

**Dependencies:**
- `python3` (YAML parsing)
- `.agentic-framework/lib/paths.sh` (PROJECT_ROOT resolution)
- `date` command (current timestamp)

**Pattern matcher (settings.json):**
```json
{
  "event": "PreToolUse",
  "tool_name": "Bash",
  "command_pattern": "git commit|git c[^o]",
  "script": ".agentic-framework/agents/context/check-dispatch-manifest.sh"
}
```

### File 2: Audit section (inline in audit.sh)

**Location:** `audit.sh` lines ~XXX (new section after dispatch section)

**Function:** Outputs dispatch-manifest YAML report

**Size:** ~100 lines

### File 3: `termlink dispatch` CLI integration (existing)

No new files needed — CLI already exists at `.termlink/bin/termlink dispatch`

**Subcommands to implement/verify:**
- `defer` — set branch status to deferred
- `merge` — complete and remove
- `status` — list all tracked branches
- `clean` — prune old entries

## 8. Data Format — Manifest Structure

### .termlink/dispatch-manifest.yaml

```yaml
# Dispatch Manifest — tracks branches spawned by dispatch protocol
# Source of truth for TermLink's agent-spawned work
# 
# Statuses:
#   pending  — actively being worked on (blocks commits if age > threshold)
#   deferred — acknowledged but not active (no block)
#   merged   — completed, waiting for cleanup
#   abandoned— intentionally stopped (old entries can be cleaned)

manifest_version: "1.0"
project: termlink
created_at: 2026-02-15T08:00:00Z
last_updated: 2026-03-30T14:50:00Z
update_reason: "Defer branch-001-feature"

# Age threshold for gate (minutes) — can be overridden via env var
age_threshold_minutes: 30

branches:
  - id: "branch-001-feature"
    name: "feature/agent-45-optimization"
    status: pending
    created_at: 2026-03-30T14:15:00Z
    worker_id: "agent-45"
    dispatcher_task: "T-789"
    description: "Optimize dispatch protocol for TermLink"
    target_branch: "main"
    
  - id: "branch-002-docs"
    name: "docs/agent-23-dispatch-guide"
    status: pending
    created_at: 2026-03-30T13:58:00Z
    worker_id: "agent-23"
    dispatcher_task: "T-175"
    description: "Update dispatch CLI documentation"
    target_branch: "main"
    
  - id: "branch-003-refactor"
    name: "refactor/agent-12-manifest"
    status: deferred
    created_at: 2026-03-30T06:30:00Z
    deferred_at: 2026-03-30T14:20:00Z
    worker_id: "agent-12"
    dispatcher_task: "T-180"
    description: "Refactor manifest parsing logic"
    target_branch: "main"
    defer_reason: "Waiting for upstream changes"
    
  - id: "branch-000-init"
    name: "init/bootstrap-dispatch"
    status: merged
    created_at: 2026-02-15T08:00:00Z
    merged_at: 2026-02-15T09:30:00Z
    worker_id: "agent-1"
    dispatcher_task: "T-001"
    description: "Initial dispatch bootstrap"
    target_branch: "main"
```

**Key fields:**
- `status`: Gate decision point
- `created_at`: Age threshold comparison
- `worker_id`: Identifies which agent spawned the branch
- `dispatcher_task`: Links branch to task that spawned it
- `merged_at` / `deferred_at`: Timestamps for state transitions

## 9. Testing & Validation

### Test Case 1: Stale Pending Branch Blocks Commit

**Setup:**
- Create manifest with pending branch created 45 minutes ago
- Threshold: 30 minutes

**Action:** `git commit -m "test"`

**Expected:** Block with message listing stale branches

### Test Case 2: Recent Pending Branch Allows Commit

**Setup:**
- Create manifest with pending branch created 10 minutes ago
- Threshold: 30 minutes

**Action:** `git commit -m "test"`

**Expected:** Allow commit (no block)

### Test Case 3: Deferred Branch Doesn't Block

**Setup:**
- Create manifest with deferred branch created 2 hours ago

**Action:** `git commit -m "test"`

**Expected:** Allow commit (deferred branches skipped)

### Test Case 4: Manifest Missing → Allow Commit

**Setup:**
- No manifest file

**Action:** `git commit -m "test"`

**Expected:** Allow commit silently

### Test Case 5: Corrupt Manifest → Allow + Warn

**Setup:**
- Manifest contains invalid YAML

**Action:** `git commit -m "test"`

**Expected:** Allow commit + warning about manifest corruption

### Test Case 6: Bypass via Env Var

**Setup:**
- Stale pending branch exists

**Action:** `FW_MANIFEST_BYPASS=1 git commit -m "test"`

**Expected:** Allow commit (bypass succeeds)

### Test Case 7: Defer Unblocks

**Setup:**
- Commit blocked by stale branch
- Run `termlink dispatch defer <branch>`

**Action:** `git commit -m "test"`

**Expected:** Allow commit (deferred status removes block)

### Test Case 8: Audit Reports Health

**Setup:**
- Run audit with dispatch-manifest section

**Action:** `fw audit --section dispatch-manifest`

**Expected:** YAML report with branch counts, ages, recommendations

## 10. Error Handling & Diagnostics

### Diagnostic Commands

**Check hook status:**
```bash
grep -A10 "check-dispatch-manifest" ~/.claude/settings.json
```

**Inspect manifest:**
```bash
cat .termlink/dispatch-manifest.yaml
```

**Run audit manually:**
```bash
fw audit --section dispatch-manifest
```

**Override hook for one commit:**
```bash
FW_MANIFEST_BYPASS=1 git commit -m "..."
```

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| "Manifest corrupt" | Invalid YAML | Run `termlink dispatch init` to reset |
| "Branch creation_at missing" | Old manifest format | Update manifest with current schema |
| "Age calculation wrong" | Timestamp parsing | Ensure ISO8601 format or Unix timestamp |
| Hook not firing | Settings.json not loaded | Reload Claude Code or check `.claude/settings.json` |

## 11. Transition & Rollout

### Phase 1: Bootstrap (Week 1)

1. Implement `check-dispatch-manifest.sh` (PreToolUse hook)
2. Add to `.claude/settings.json` with low threshold (2 hours) to warn
3. Deploy to termlink project only
4. Document in AGENT.md

### Phase 2: Enforcement (Week 2)

1. Lower threshold to 30 minutes
2. Train team on `termlink dispatch` commands
3. Add audit section to schedule

### Phase 3: Generalization (Week 3+)

1. Move hook to framework repo for all projects
2. Make configurable per project
3. Add integration tests

## 12. Related Tasks & References

- **T-789:** Worktree isolation for TermLink dispatch (parent task)
- **T-280:** Dispatch readiness — structural completion
- **T-282:** Dispatch command — atomic spawn
- **T-175:** Agent mesh orchestration
- **T-184:** Audit scheduling (framework)
- **P-002:** Structural enforcement over agent discipline
- **B-005:** Enforcement config protection

## 13. Rollback Plan

If the gate causes too many false positives:

1. **Quick:** `FW_MANIFEST_BYPASS=1 git commit` (per-commit override)
2. **Temporary:** Raise threshold: `TERMLINK_MANIFEST_AGE_THRESHOLD=120`
3. **Disable:** Remove hook from `.claude/settings.json` and reload
4. **Full revert:** Delete `.agentic-framework/agents/context/check-dispatch-manifest.sh`

## Summary

This gate provides three layers of control:

1. **Pre-commit hook** (tight feedback) — blocks stale branches at commit time
2. **Audit section** (trend tracking) — reports on branch health without blocking
3. **CLI commands** (user control) — defer, merge, clean, or check status

The design balances **automation** (gatekeeping) with **agency** (clear escape hatches) and **observability** (audit reports).
