---
id: T-1025
name: "hub restart TCP detection — server-side hub.tcp recording for reliable restart"
description: >
  Inception: hub restart TCP detection — server-side hub.tcp recording for reliable restart

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T13:01:16Z
last_update: 2026-04-13T13:07:18Z
date_finished: 2026-04-13T13:07:18Z
---

# T-1025: hub restart TCP detection — server-side hub.tcp recording for reliable restart

## Problem Statement

`termlink hub restart` (T-1024) loses TCP config when restarting hubs started before T-1024, because `hub.tcp` is written at the CLI layer (`infrastructure.rs:43`), not the server layer. First restart after deploying new code drops TCP — hub becomes unreachable over the network. Observed on .121 during T-1023 deployment. Affects all remote hub upgrades via termlink.

## Assumptions

- A1: Writing `hub.tcp` in `server.rs` after `TcpListener::bind()` covers all start paths (validated: single bind site at server.rs:145)
- A2: Removing the CLI-layer write doesn't break anything (validated: `hub.tcp` is new from T-1024, no other consumers)
- A3: Cleanup on shutdown prevents stale TCP config (validated: shutdown already handles socket + pidfile at server.rs:182-188)

## Exploration Plan

1. [5min] Trace TCP address flow through code — DONE
2. [5min] Evaluate 3 options for persistence location — DONE (Option B: server-side)
3. [5min] Check cleanup paths — DONE (1 line addition)
4. [5min] Assess `--tcp` override for restart — DONE (orthogonal, out of scope)

See `docs/reports/T-1025-hub-restart-tcp-detection.md` for full research.

## Technical Constraints

- `hub.tcp` must be written after bind succeeds (not before — bind can fail)
- Must use `local_addr()` not the CLI arg (handles 0.0.0.0 binding correctly)
- Must be removed on clean shutdown to prevent stale config
- `discovery::runtime_dir()` is the canonical path source

## Scope Fence

**IN:** Move `hub.tcp` write to server layer, cleanup on shutdown, remove CLI-layer write
**OUT:** Hub federation, systemd integration, `hub restart --tcp` override (separate task)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (observed on .121 during T-1023)
- [x] Assumptions tested (code traced, all 3 validated)
- [x] Recommendation written with rationale (GO — 5 lines, 2 files)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** Root cause is clear (wrong persistence layer), fix is bounded (~5 lines in 2 files), fully testable, and reversible. The gap blocks reliable remote deployment via termlink — the core use case for T-1016.

**Evidence:**
- Only one TCP bind site in `server.rs:145` — writing `hub.tcp` after bind covers all paths
- CLI-layer write (`infrastructure.rs:43-50`) is redundant once server writes it
- Shutdown cleanup path exists at `server.rs:182-188` — adding `hub.tcp` removal is 1 line
- No existing consumers of `hub.tcp` beyond `hub restart` (T-1024) — no backward compat risk
- Real-world failure observed: .121 lost TCP during T-1023 deployment

## Decisions

### 2026-04-13 — hub.tcp persistence location
- **Chose:** Option B — server-side write in `server.rs` after `TcpListener::bind()`
- **Why:** Covers all start paths, records actual bound address, single source of truth
- **Rejected:**
  - Option A (CLI-only, current): doesn't cover non-CLI starts, bootstrapping gap
  - Option C (server + CLI override): unnecessary complexity, CLI can pass --tcp to hub start directly

## Decision

**Decision**: GO

**Rationale**: Server-side hub.tcp write covers all start paths, fixes bootstrapping gap

**Date**: 2026-04-13T13:07:18Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-13T13:01:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-13T13:07:18Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Server-side hub.tcp write covers all start paths, fixes bootstrapping gap

### 2026-04-13T13:07:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
