---
id: T-984
name: "fw upgrade silently reverts local patches — do_vendor has no modification detection"
description: >
  fw upgrade silently reverts local patches — do_vendor has no modification detection

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-04-12T16:46:26Z
last_update: 2026-04-22T04:52:54Z
date_finished: 2026-04-12T17:17:21Z
---

# T-984: fw upgrade silently reverts local patches — do_vendor has no modification detection

## Problem Statement

`fw upgrade` on vendored consumer projects silently overwrites locally-patched framework files. The T-1157 upstream refactor replaced per-file sync with bulk `do_vendor`, removing T-978's checksum manifest + backup logic. Observed this session: 6 local fixes reverted (T-911, T-913, T-949, T-938, T-978 itself). Each session wastes time re-diagnosing and re-applying.

See `docs/reports/T-984-fw-upgrade-local-patch-reversion.md` for full analysis.

## Assumptions

1. `do_vendor()` currently does unconditional rsync with no modification check
2. Consumer projects will always need some local patches (PROJECT_ROOT resolution is structurally consumer-specific)
3. A manifest-based approach (`.local-patches`) can prevent accidental overwriting without blocking intentional upgrades

## Exploration Plan

1. Spike 1: Confirm `do_vendor()` behavior (read current code, verify no checksum logic)
2. Spike 2: Design `.local-patches` YAML format
3. Spike 3: Prototype manifest check in do_vendor — skip or backup patched files

## Technical Constraints

- Must work with the upstream `do_vendor` pattern (T-1157) — cannot revert to per-file sync
- Must be backward compatible (no manifest = normal upgrade behavior)
- Backup path must not collide with `.upstream-checksums` (T-978 artifact still exists)

## Scope Fence

**IN:** Manifest format, do_vendor modification, `fw patch register` command, upgrade warning
**OUT:** Upstream PR (separate effort per Option D), auto-detecting local patches from git diff

## Acceptance Criteria

### Agent
- [x] Problem statement validated (6 files reverted in this session, documented in research artifact)
- [x] Assumptions tested (do_vendor confirmed no checksum logic; consumer patches are structurally necessary)
- [x] Recommendation written with rationale (GO: Option C + D)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-984, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- The fix can be contained within the consumer project (no upstream dependency for the core mechanism)
- The patch manifest can be maintained with low overhead (1 command per patched file)

**NO-GO if:**
- All local patches can be upstreamed before the next fw upgrade cycle (eliminates the need)
- The do_vendor approach is being replaced upstream with something that already handles this

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** This session spent significant time re-diagnosing and re-applying 6 local fixes that `fw upgrade` silently reverted. The current `do_vendor` (T-1157) does unconditional rsync with no modification detection. A `.local-patches` manifest is low-effort, consumer-controlled, and immediately prevents the recurring reversion cycle.

**Evidence:**
- 6 files reverted in this session alone (T-911, T-913, T-949, T-938a, T-938b, T-978)
- T-978's backup mechanism was overwritten by the exact threat it protected against
- Previous session also spent time re-applying the same fixes
- Consumer-specific patches (PROJECT_ROOT resolution) cannot be upstreamed — they're structurally necessary

**Build scope (if GO):**
1. `.agentic-framework/.local-patches` YAML manifest
2. `do_vendor` check: skip/backup files listed in manifest
3. `fw patch register <file> --task T-XXX` CLI command
4. `fw upgrade` summary: "N patched file(s) preserved"

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: This session spent significant time re-diagnosing and re-applying 6 local fixes that `fw upgrade` silently reverted. The current `do_vendor` (T-1157) does uncondition...

**Date**: 2026-04-12T17:17:21Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: This session spent significant time re-diagnosing and re-applying 6 local fixes that `fw upgrade` silently reverted. The current `do_vendor` (T-1157) does uncondition...

**Date**: 2026-04-12T17:17:21Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:17:21Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: This session spent significant time re-diagnosing and re-applying 6 local fixes that `fw upgrade` silently reverted. The current `do_vendor` (T-1157) does uncondition...

### 2026-04-12T17:17:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:05:40Z — programmatic-evidence [T-1090]
- **Evidence:** fw upgrade --help shows --dry-run flag; patch recipe docs/patches/T-1066-fw-task-review-queue.md in place for re-application
- **Verified by:** automated command execution

### 2026-04-22T04:52:54Z — status-update [task-update-agent]
- **Change:** horizon: later → next
