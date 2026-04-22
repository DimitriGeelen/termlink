---
id: T-789
name: "Worktree isolation for TermLink-dispatched agents — git worktree per spawned session"
description: >
  Inception: Worktree isolation for TermLink-dispatched agents — git worktree per spawned session

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-30T12:47:17Z
last_update: 2026-04-22T04:52:51Z
date_finished: 2026-03-30T13:07:53Z
---

# T-789: Worktree isolation for TermLink-dispatched agents — git worktree per spawned session

## Problem Statement

The Rust `termlink dispatch` command — the primary recommended dispatch mechanism — has no filesystem isolation. All spawned workers inherit the parent CWD and share the same git branch. Concurrent workers editing the same files will conflict. The shell prototype (`dispatch.sh --isolate`) solves this with git worktrees but uses a separate code path from the Rust dispatch infrastructure. The trust model (trust.rs) already recognizes worktree isolation as "Low blast radius" but dispatch doesn't enforce it.

## Assumptions

1. The dispatch.sh `--isolate` pattern (git worktree per worker) is proven and transferable to Rust
2. Rust lifecycle management (Drop) will be more reliable than bash trap cleanup
3. Users expect `termlink dispatch` to be the single dispatch interface (not dispatch.sh)
4. Git worktree overhead is acceptable for dispatch workflows (< 1s per worker)

## Exploration Plan

1. **Research (done):** Map current dispatch.sh capabilities, Rust dispatch gaps, trust model intent
2. **Options analysis (done):** 4 options scored against 4 framework directives with steelman/strawman
3. **Prototype (if GO):** Add `--workdir` first (Option B stepping stone), then `--isolate`

## Technical Constraints

- Git worktrees require `.git` to be present (not bare clones, not non-git projects)
- Worktree creation on large repos may be slow (mitigated: worktrees are cheap — just a checkout)
- CARGO_TARGET_DIR must be per-worktree to avoid build conflicts
- macOS bash 3.2 issues (T-160) don't apply to Rust implementation

## Scope Fence

**IN scope:**
- `--isolate` flag for Rust `termlink dispatch`
- `--workdir` flag as stepping stone
- Auto-commit and branch preservation on worker exit
- JSON output of created branches

**OUT of scope:**
- Merge orchestration (separate task — how to merge N worker branches back)
- Non-git VCS support
- Remote dispatch isolation (TCP hub workers)

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Read the research artifact and recommendation in this task
  2. Evaluate go/no-go criteria against findings
  3. Run: `cd /opt/999-Agentic-Engineering-Framework && bin/fw inception decide T-XXX go|no-go --rationale "your rationale"`
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Option A scores highest against all 4 directives (confirmed: 7/8)
- dispatch.sh --isolate proves the worktree pattern works in practice
- Rust dispatch is the recommended dispatch path (T-533 enforcement)

**NO-GO if:**
- Worktree management adds unacceptable complexity to the Rust binary
- Non-git VCS support is a hard requirement
- The shell script is "good enough" and Rust dispatch isn't the primary path

## Verification

# Research artifact exists
test -f docs/reports/T-789-worktree-isolation-research.md

## Decisions

**Decision**: GO

**Rationale**: Option A (7/8). Dispatch manifest makes all 5 failure modes deterministically mitigated. 64 tests specified. Implementation:   
  --workdir, --isolate+manifest, --auto-merge, pre-commit gate, audit check.

**Date**: 2026-03-30T13:07:53Z
## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

**Decision (retro-captured from Decision block):** GO

**Rationale:** Option A (7/8). Dispatch manifest makes all 5 failure modes deterministically mitigated. 64 tests specified. Implementation:

## Decision

**Decision**: GO

**Rationale**: Option A (7/8). Dispatch manifest makes all 5 failure modes deterministically mitigated. 64 tests specified. Implementation:   
  --workdir, --isolate+manifest, --auto-merge, pre-commit gate, audit check.

**Date**: 2026-03-30T13:07:53Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-30T12:50:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T13:07:53Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Option A (7/8). Dispatch manifest makes all 5 failure modes deterministically mitigated. 64 tests specified. Implementation:   
  --workdir, --isolate+manifest, --auto-merge, pre-commit gate, audit check.

### 2026-03-30T13:07:53Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:40:15Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-22T04:52:51Z — status-update [task-update-agent]
- **Change:** horizon: later → next
