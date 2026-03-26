---
id: T-280
name: "Dispatch readiness — structural completion signaling for TermLink agent orchestration"
description: >
  Inception: TermLink has a collect-based dispatch convention (T-257) but agents don't use it
  in practice — they fall back to manual polling or asking the human. Root cause: convention
  is documented but not structurally enforced, prompt templates lack event emission instructions,
  and TermLink emits zero lifecycle events when sessions die. Explore solutions: dispatch command,
  auto-lifecycle events, dispatch-aware spawn. 5-agent research completed.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [dispatch, orchestration, lifecycle, mcp]
components: []
related_tasks: [T-233, T-256, T-257]
created: 2026-03-25T14:27:11Z
last_update: 2026-03-25T14:27:11Z
date_finished: null
---

# T-280: Dispatch readiness — structural completion signaling for TermLink agent orchestration

## Problem Statement

TermLink's collect-based dispatch convention (T-257) works in E2E tests but fails in real
agent sessions. Observed: orchestrating agents fall back to manual `fw termlink status` polling
or ask the human when workers finish — despite the convention existing on paper.

**For whom:** Any AI agent orchestrating multi-worker dispatch via TermLink (Claude Code, MCP clients).
**Why now:** MCP server (26 tools) makes TermLink the primary AI agent integration surface. Dispatch
readiness is the gap between "tools exist" and "agents can orchestrate reliably."

### Root Causes (from 5-agent RCA)

1. **Prompt templates lack event emission instructions** — workers never told to emit `task.completed`
2. **dispatch.sh never retrofitted post-T-257** — orchestration layer unchanged after convention created
3. **No automatic lifecycle events** — when a session's PID dies, TermLink emits nothing; hub supervisor silently removes dead sessions
4. **T-257 Human AC never verified** — convention never stress-tested with real Claude agent dispatch

### Evidence

- Conversation transcript: agent admits "I don't have a listener wired to receive those events"
- `crates/termlink-hub/src/supervisor.rs`: sweep silently removes dead sessions, no event emission
- `crates/termlink-cli/src/commands/execution.rs:82-88`: deregistration emits nothing
- `tests/e2e/level9-dispatch-collect.sh`: works because test controls both sides

## Assumptions

- A1: A `termlink dispatch` command would be used by agents if it existed (structural > convention)
- A2: Auto `session.exited` events from supervisor are reliable enough (0-30s latency acceptable)
- A3: No new RPC methods needed — existing spawn/collect/discover suffice
- A4: ~430 LOC total implementation is achievable in one session

## Exploration Plan

Research phase COMPLETE (5 parallel agents dispatched 2026-03-25):
1. RCA Agent — traced causal chain from T-257 convention to real-world failure
2. Lifecycle Agent — analyzed session lifecycle, identified supervisor as emission point
3. Dispatch Command Agent — scoped `termlink dispatch` CLI command (~350 LOC)
4. Dimensions Agent — scored 3 solutions against 4 constitutional directives
5. Steelman/Strawman Agent — strongest case FOR and AGAINST each solution

## Technical Constraints

- Hub supervisor polls every 30 seconds — lifecycle event latency is 0-30s
- No push mechanism exists (T-256 NO-GO on push messaging — poll latency is negligible for minute-long tasks)
- Event ring buffer: 1024 events, ~34x headroom for current workloads
- Claude Code Bash tool: 10-minute timeout, supports `run_in_background`

## Scope Fence

**IN scope:**
- `termlink dispatch` CLI command (spawn-N + tag + collect atomic wrapper)
- `session.exited` lifecycle event (supervisor emits before cleanup)
- Integration with existing MCP tools

**OUT of scope:**
- Push-based messaging (T-256 NO-GO, revisit at 50+ workers)
- Cross-machine dispatch (T-163 handles this separately)
- Framework-side dispatch.sh changes (framework repo, not TermLink)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (5-agent RCA complete)
- [x] Assumptions documented
- [ ] Go/No-Go decision made

### Human
- [ ] [REVIEW] Review 5-agent research synthesis and agree with recommended path
  **Steps:**
  1. Read this task's Problem Statement and the 3 proposed solutions below
  2. Confirm or modify the recommended path (B+A: dispatch command + lifecycle events)
  **Expected:** Human confirms GO with chosen solution path
  **If not:** Discuss alternatives or scope changes

## Go/No-Go Criteria

**GO if:**
- Recommended solution scores well on all 4 directives (antifragility, reliability, usability, portability)
- Implementation scope fits in 1-2 sessions (~430 LOC)
- No new RPC methods needed (composes existing primitives)

**NO-GO if:**
- Solution requires protocol-breaking changes
- Implementation scope exceeds 2 sessions
- Existing collect pattern is "good enough" with just documentation fixes

## Proposed Solutions

### Solution A: Auto-Lifecycle Events (~80 LOC)
Supervisor emits `session.exited` before cleanup when PID dies.
- **Steelman:** Zero worker code changes, leverages existing liveness infrastructure
- **Strawman:** 0-30s latency, hub must be running, no crash vs clean exit distinction
- **Dimension score:** 3.1/5

### Solution B: `termlink dispatch` Command (~350 LOC)
New CLI: `termlink dispatch --count 3 --timeout 300 -- <cmd>`. Atomic spawn+tag+collect.
- **Steelman:** Structural guarantee, atomic failure handling, single command replaces 40-line scripts
- **Strawman:** Monolithic, can't dispatch pre-existing workers, adds CLI surface
- **Dimension score:** 4.7/5

### Solution C: Dispatch-Aware Spawn (~50 LOC)
`termlink spawn --dispatch-id D-001 --notify-on-exit` flag.
- **Steelman:** Opt-in, composable, lowest risk
- **Strawman:** Still convention-based (opt-in flag), doesn't reduce orchestrator complexity
- **Dimension score:** 2.8/5

### Recommended Path: B + A
Build the dispatch command (structural guarantee) with lifecycle events as crash safety net.
Skip C — opt-in flags repeat the convention problem that caused this gap.

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: 5-agent research validates B+A path: dispatch command (4.7/5 on directives) + lifecycle events as crash safety net. ~430 LOC, no new RPC methods, composes existing primitives.

**Date**: 2026-03-25T14:30:07Z
## Decision

**Decision**: GO

**Rationale**: 5-agent research validates B+A path: dispatch command (4.7/5 on directives) + lifecycle events as crash safety net. ~430 LOC, no new RPC methods, composes existing primitives.

**Date**: 2026-03-25T14:30:07Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-26 — research-artifact [agent]
- **Artifact:** `docs/reports/T-280-dispatch-readiness.md`

### 2026-03-25T14:30:07Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** 5-agent research validates B+A path: dispatch command (4.7/5 on directives) + lifecycle events as crash safety net. ~430 LOC, no new RPC methods, composes existing primitives.
