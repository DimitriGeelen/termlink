---
id: T-114
name: "PoC — replace internal agent spawning with TermLink agent mesh"
description: >
  Prove that TermLink can replace Claude Code's internal sub-agent spawning mechanism.
  Minimum viable round-trip: orchestrator dispatches task via TermLink event → worker agent
  receives, executes, returns result via TermLink event. No pool, no parallelism, no
  cross-machine — just prove the communication pattern works.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [agent-mesh, poc, termlink-core, spawning]
components: []
related_tasks: [T-009, T-100]
created: 2026-03-12T08:27:33Z
last_update: 2026-03-12T09:34:53Z
date_finished: 2026-03-12T09:34:42Z
---

# T-114: PoC — replace internal agent spawning with TermLink agent mesh

## Problem Statement

Claude Code spawns sub-agents as internal subprocesses (sidechain JSONL). This is opaque,
locked-in, and non-portable. TermLink already has session registration, discovery, command
execution, and an event bus with request-reply. Can we wire these primitives to replace
internal spawning with TermLink-routed agent dispatch?

## Assumptions

- A-001: TermLink event round-trip works with existing CLI (no code changes)
- A-002: Claude Code can run inside a `termlink run` wrapper without conflicts
- A-003: Event payload size is sufficient for task prompts and result summaries

## Exploration Plan

1. **Spike 1** (~15 min): Manual round-trip with existing CLI commands
2. **Spike 2** (~30 min): Claude Code as TermLink agent via `termlink run`
3. **Spike 3** (~60 min): Automated dispatch script wiring spikes 1+2

## Technical Constraints

- Unix sockets only (local machine) — sufficient for PoC
- Claude Code `--print` mode for non-interactive single-prompt execution
- Shared filesystem for result artifacts (no network transfer needed)
- Hub must be running for session routing

## Scope Fence

**IN:** Single round-trip proof — orchestrator → TermLink → worker → result
**OUT:** Worker pools, parallelism, cross-machine, auth, transport abstraction

## Acceptance Criteria

### Agent
- [x] Spike 1 validated: manual event round-trip with existing CLI
- [x] Spike 2 validated: Claude Code runs inside `termlink run` wrapper
- [x] Spike 3 validated: automated dispatch end-to-end
- [x] Research artifact at `docs/reports/T-114-poc-agent-mesh-spawning.md`
- [x] GO/NO-GO framed

### Human
- [ ] Design reviewed and direction decided

## Go/No-Go Criteria

**GO if:**
- Spike 1 confirms event round-trip works with existing CLI
- Spike 2 confirms Claude Code can run inside `termlink run` wrapper
- Total PoC build effort fits in one session

**NO-GO if:**
- TermLink event system can't handle the payload sizes needed
- Claude Code subprocess model conflicts with TermLink session registration
- Hub routing introduces unacceptable latency

## Verification

test -f docs/reports/T-114-poc-agent-mesh-spawning.md

## Decisions

**Decision**: GO

**Rationale**: All 3 spikes pass. Event round-trip works, Claude Code runs as TermLink agent via wrapper, full automated dispatch returns correct results. Two bugs fixed along the way (wait cursor, run arg quoting).

**Date**: 2026-03-12T09:34:42Z
## Decision

**Decision**: GO

**Rationale**: All 3 spikes pass. Event round-trip works, Claude Code runs as TermLink agent via wrapper, full automated dispatch returns correct results. Two bugs fixed along the way (wait cursor, run arg quoting).

**Date**: 2026-03-12T09:34:42Z

## Updates

### 2026-03-12T08:27:33Z — task-created [task-create-agent]
- **Action:** Created inception task
- **Context:** User prioritized agent mesh PoC as highest-value work

### 2026-03-12T08:35:00Z — research [claude-code]
- **Action:** Explored TermLink capabilities and Claude Code spawning mechanism
- **Findings:** TermLink has all primitives needed (register, discover, exec, events, request-reply). Gap is orchestration glue.
- **Artifact:** docs/reports/T-114-poc-agent-mesh-spawning.md

### 2026-03-12T09:31:21Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All primitives work: event round-trip confirmed, Claude Code runs via agent-wrapper.sh inside termlink run, wait bug fixed. PoC build fits in one session.

### 2026-03-12T09:31:27Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-12T09:31:33Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All primitives work: event round-trip confirmed, Claude Code runs via agent-wrapper.sh inside termlink run, wait bug fixed. PoC build fits in one session.

### 2026-03-12T09:34:42Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All 3 spikes pass. Event round-trip works, Claude Code runs as TermLink agent via wrapper, full automated dispatch returns correct results. Two bugs fixed along the way (wait cursor, run arg quoting).

### 2026-03-12T09:34:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
