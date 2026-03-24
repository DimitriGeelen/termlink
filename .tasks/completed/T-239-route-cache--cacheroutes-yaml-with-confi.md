---
id: T-239
name: "Route cache — .cache/routes/ YAML with confidence, TTL, lazy invalidation"
description: >
  Per-agent route cache in .cache/routes/ keyed by capability slug. YAML entries with confidence scores, TTL, hit counts, schema validation. 3-way branch: hit+valid -> direct, partial match -> refinement query, miss -> orchestrator. See T-233 research: Q2b-routing-decision.md

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [T-233, orchestration, cache]
components: []
related_tasks: [T-233, T-237]
created: 2026-03-23T13:27:32Z
last_update: 2026-03-24T06:33:29Z
date_finished: 2026-03-24T06:33:29Z
---

# T-239: Route cache — .cache/routes/ YAML with confidence, TTL, lazy invalidation

## Context

Evaluate whether a route cache is needed given: (1) T-237 orchestrator.route RPC exists but no orchestrator dispatches routes yet, (2) T-240/T-241/T-242 all NO-GO due to absent specialist ecosystem, (3) current dispatch is direct (human/agent names target explicitly). Research: `docs/reports/T-233-Q2b-routing-decision.md`.

## Problem Statement

Is a per-agent route cache needed when the routing infrastructure it caches from (orchestrator.route, specialist discovery) doesn't exist?

## Acceptance Criteria

### Agent
- [x] Research artifact created at `docs/reports/T-239-route-cache-inception.md`
- [x] GO/NO-GO decision recorded with rationale

## Verification

test -f docs/reports/T-239-route-cache-inception.md
grep -q "GO\|NO-GO" docs/reports/T-239-route-cache-inception.md

## Decisions

**Decision**: NO-GO

**Rationale**: No routing lookups exist to cache. Dispatch is direct. Same root cause as T-240/T-241/T-242.

**Date**: 2026-03-24T06:33:29Z

## Updates

### 2026-03-23T13:27:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-239-route-cache--cacheroutes-yaml-with-confi.md
- **Context:** Initial task creation

### 2026-03-24T06:32:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-24T06:33:29Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** No routing lookups exist to cache. Dispatch is direct. Same root cause as T-240/T-241/T-242.

### 2026-03-24T06:33:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
