---
id: T-912
name: "Define vendor-refresh workflow — fw upgrade from consumer re-syncs framework"
description: >
  Follow-up from T-909. After a consumer project vendors its framework via fw vendor, subsequent fw upgrade from that consumer becomes a no-op (or worse: upgrades the vendored copy from itself). Need a workflow where fw upgrade from a consumer project either (a) re-runs fw vendor from the live framework source, or (b) runs a git-like pull from the framework repo into the vendored copy. Scope: decide which pattern, prototype, document.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [infrastructure, vendor, upgrade, workflow]
components: []
related_tasks: []
created: 2026-04-11T12:28:41Z
last_update: 2026-04-12T11:41:37Z
date_finished: 2026-04-12T11:41:27Z
---

# T-912: Define vendor-refresh workflow — fw upgrade from consumer re-syncs framework

## Problem Statement

`fw upgrade` (upgrade.sh section 4b) syncs vendored `.agentic-framework/` scripts from the upstream framework via blind `cp`. This destroys any local modifications the consumer project has made. In session S-2026-0412-1253, fw upgrade v1.5.356 silently overwrote T-962 (compat.sh date helpers), T-970/T-971 (review.sh port/browser fixes), and T-963 (init.sh concerns) — all work done in the same week. No warning, no backup, no diff.

**For whom:** Any consumer project that vendors the framework and makes local fixes.
**Why now:** It already caused data loss. The pattern will repeat on every future upgrade.

## Assumptions

- A-1: Consumer projects will always need to make local modifications to vendored framework files (bug fixes, project-specific hooks)
- A-2: Most upgrade conflicts are in a small number of files (bin/fw, lib/*.sh), not in templates or seed data
- A-3: A checksum-based "dirty file" detection is sufficient — full git merge is overkill for this use case

## Exploration Plan

1. **Spike 1: Quantify the problem** — How many files does upgrade.sh sync? How many had local changes when the v1.5.356 incident happened? (10 min)
2. **Spike 2: Options analysis** — Compare approaches: checksum manifest, git subtree, patch-based, two-layer override. (15 min)
3. **Recommendation** — Pick approach, define build tasks. (5 min)

## Technical Constraints

- Must work with vendored copies (no symlinks, no git subtrees)
- Must be backward-compatible — old consumer projects without the manifest should upgrade without breaking
- Must work on both Linux and macOS (bash 3.2+)
- The framework is in `.gitignore` in most consumer projects, so git tracking of individual framework files is not available

## Scope Fence

**IN scope:** Detection and protection of local modifications during `fw upgrade`. Backup mechanism. Warning output.
**OUT of scope:** Two-way sync (pushing local fixes upstream), merge conflict resolution UI, git subtree migration.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `cd /opt/termlink && bin/fw task review T-912` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- A feasible approach exists that protects local modifications without requiring git subtrees or symlinks
- The approach is backward-compatible (no breakage for projects that haven't adopted it)
- Implementation fits in 1-2 build tasks

**NO-GO if:**
- Every viable approach requires git-level tracking of framework files (contradicts .gitignore convention)
- The approach adds >50 lines of complexity to upgrade.sh for a rare edge case

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** The problem is real (caused data loss in S-2026-0412-1253), the fix is simple (~30 lines to upgrade.sh), and backward-compatible. Option A+D (checksum manifest + backup) gives protection without blocking upgrades.

**Evidence:**
- v1.5.356 upgrade destroyed 3 files with local modifications (T-962, T-970/T-971, T-963)
- Only 3/60 synced files had local changes — checksum detection is sufficient
- No git-level tracking needed — works with .gitignore convention
- Implementation is 2 build tasks, each fitting in one session

**Approach:** Checksum manifest (`.upstream-checksums`) + backup directory (`.upgrade-backup/`). Detect local modifications before overwriting, back up modified files, warn, and provide `fw upgrade --restore` for recovery.

**Research artifact:** `docs/reports/T-912-vendor-refresh-workflow.md`

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: The problem is real (caused data loss in S-2026-0412-1253), the fix is simple (~30 lines to upgrade.sh), and backward-compatible. Option A+D (checksum manifest + backup) gives protection without blocking upgrades.

Evidence:
- v1.5.356 upgrade destroyed 3 files with local modifications (T-962, T-970/T-971, T-963)
- Only 3/60 synced files had local changes — checksum detection is sufficient
- No git-level tracking needed — works with .gitignore convention
- Implementation is 2 build tasks, each fitting in one session

Approach: Checksum manifest (`.upstream-checksums`) + backup directory (`.upgrade-backup/`). Detect local modifications before overwriting, back up modified files, warn, and provide `fw upgrade --restore` for recovery.

Research artifact: `docs/reports/T-912-vendor-refresh-workflow.md`

**Date**: 2026-04-12T11:41:37Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: The problem is real (caused data loss in S-2026-0412-1253), the fix is simple (~30 lines to upgrade.sh), and backward-compatible. Option A+D (checksum manifest + backup) gives protection without blocking upgrades.

Evidence:
- v1.5.356 upgrade destroyed 3 files with local modifications (T-962, T-970/T-971, T-963)
- Only 3/60 synced files had local changes — checksum detection is sufficient
- No git-level tracking needed — works with .gitignore convention
- Implementation is 2 build tasks, each fitting in one session

Approach: Checksum manifest (`.upstream-checksums`) + backup directory (`.upgrade-backup/`). Detect local modifications before overwriting, back up modified files, warn, and provide `fw upgrade --restore` for recovery.

Research artifact: `docs/reports/T-912-vendor-refresh-workflow.md`

**Date**: 2026-04-12T11:41:37Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T11:27:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T11:41:27Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: The problem is real (caused data loss in S-2026-0412-1253), the fix is simple (~30 lines to upgrade.sh), and backward-compatible. Option A+D (checksum manifest + backup) gives protection without blocking upgrades.

Evidence:
- v1.5.356 upgrade destroyed 3 files with local modifications (T-962, T-970/T-971, T-963)
- Only 3/60 synced files had local changes — checksum detection is sufficient
- No git-level tracking needed — works with .gitignore convention
- Implementation is 2 build tasks, each fitting in one session

Approach: Checksum manifest (`.upstream-checksums`) + backup directory (`.upgrade-backup/`). Detect local modifications before overwriting, back up modified files, warn, and provide `fw upgrade --restore` for recovery.

Research artifact: `docs/reports/T-912-vendor-refresh-workflow.md`

### 2026-04-12T11:41:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-12T11:41:37Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: The problem is real (caused data loss in S-2026-0412-1253), the fix is simple (~30 lines to upgrade.sh), and backward-compatible. Option A+D (checksum manifest + backup) gives protection without blocking upgrades.

Evidence:
- v1.5.356 upgrade destroyed 3 files with local modifications (T-962, T-970/T-971, T-963)
- Only 3/60 synced files had local changes — checksum detection is sufficient
- No git-level tracking needed — works with .gitignore convention
- Implementation is 2 build tasks, each fitting in one session

Approach: Checksum manifest (`.upstream-checksums`) + backup directory (`.upgrade-backup/`). Detect local modifications before overwriting, back up modified files, warn, and provide `fw upgrade --restore` for recovery.

Research artifact: `docs/reports/T-912-vendor-refresh-workflow.md`
