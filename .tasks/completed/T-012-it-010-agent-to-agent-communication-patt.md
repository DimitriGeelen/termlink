---
id: T-012
name: "IT-010: Agent-to-agent communication patterns"
description: >
  Multi-agent workflows, delegation, peer review, conflict resolution, scaling limits

status: work-completed
workflow_type: inception
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:19:50Z
last_update: 2026-03-09T12:01:03Z
date_finished: 2026-03-09T12:01:03Z
---

# T-012: IT-010: Agent-to-agent communication patterns

## Problem Statement

How should TermLink enable the Agentic Engineering Framework to delegate tasks from an orchestrator agent to specialist agents in separate terminals — each with their own context window, governed by the same framework rules? Full research at `docs/reports/T-012-agent-to-agent-communication.md`.

## Assumptions

1. Existing TermLink primitives (discover, emit, wait, kv) cover 70%+ of agent-to-agent needs — **validated** by spike 1
2. Context budget multiplier is the killer feature (each specialist = fresh 200K) — **structurally true**
3. Framework governance applies naturally to specialists (same repo, same CLAUDE.md) — **validated**
4. Only `spawn` and `request` commands are truly missing — **validated** by gap analysis

## Scope Fence

**IN:** Task delegation protocol, `termlink spawn`, `termlink request`, event schema convention
**OUT:** Agent pools, load balancing, cross-machine topology, persistent warm caches

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- [x] Existing primitives cover 70%+ of the pattern
- [x] Phase 1 is bounded and deliverable (3 tasks)
- [x] Framework governance can be enforced on specialists

**NO-GO if:**
- [ ] Terminal spawning is platform-dependent nightmare
- [ ] Event-based delegation is too lossy

## Verification

test -f docs/reports/T-012-agent-to-agent-communication.md

## Recommendation

**Recommendation:** GO
**Rationale:** Existing TermLink primitives (discover, emit, wait, kv) cover 70%+ of agent-to-agent needs. Only `spawn` and `request` were structurally missing — both small, bounded additions. Context-budget multiplier (each specialist gets fresh 200K) is the killer feature and is structurally guaranteed.
**Evidence:**
- Research artifact: docs/reports/T-012-agent-to-agent-communication.md
- Spike 1 validated A1 (primitives cover 70%+)
- Gap analysis validated A4 (only spawn + request missing)
- Framework governance applies naturally (A3 validated) — same repo, same CLAUDE.md

## Decisions

### 2026-03-09 — GO decision
- **Chose:** GO — Build Phase 1 (spawn, request, event schema)
- **Why:** 70%+ coverage from existing primitives, context budget multiplier is structural, 3 bounded build tasks
- **Rejected:** Convention-only (missing spawn), Full agent framework (over-engineered)

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-09T10:29:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-09T12:00:13Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Existing primitives cover 70%+, context budget multiplier is structural, Phase 1 is 3 bounded tasks (spawn, request, event schema)

### 2026-03-09T12:01:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
