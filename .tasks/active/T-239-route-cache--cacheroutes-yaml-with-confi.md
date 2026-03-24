---
id: T-239
name: "Route cache — .cache/routes/ YAML with confidence, TTL, lazy invalidation"
description: >
  Per-agent route cache in .cache/routes/ keyed by capability slug. YAML entries with confidence scores, TTL, hit counts, schema validation. 3-way branch: hit+valid -> direct, partial match -> refinement query, miss -> orchestrator. See T-233 research: Q2b-routing-decision.md

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-233, orchestration, cache]
components: []
related_tasks: [T-233, T-237]
created: 2026-03-23T13:27:32Z
last_update: 2026-03-24T08:00:00Z
date_finished: null
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
- [x] `RouteCache` struct in `crates/termlink-hub/src/route_cache.rs` with JSON persistence
- [x] Cache entries: capability slug, specialist, confidence, TTL, hit_count, last_used, request_schema
- [x] 3-way lookup: hit+valid → CacheHit, expired/low-confidence → Stale, miss → CacheMiss
- [x] Confidence decay: 0.05/week of non-use
- [x] Record successful route (from orchestrator.route response) into cache
- [x] Invalidate on specialist rejection (schema mismatch or RPC error)
- [x] Integration: `handle_orchestrator_route` checks route cache between bypass and discovery
- [x] Tests: hit, miss, stale, confidence decay, invalidation, persistence round-trip (17 tests)

## Verification

test -f docs/reports/T-239-route-cache-inception.md
grep -q "GO\|NO-GO" docs/reports/T-239-route-cache-inception.md
test -f crates/termlink-hub/src/route_cache.rs
/Users/dimidev32/.cargo/bin/cargo test route_cache 2>&1 | grep -q "test result: ok"

## Decisions

### 2026-03-24T06:33:29Z — Original NO-GO (REVERSED)
- **Decision:** NO-GO (now overridden)
- **Original rationale:** No routing lookups exist to cache. Dispatch is direct. Same root cause as T-240/T-241/T-242.
- **Research:** `docs/reports/T-239-route-cache-inception.md` — found zero orchestrator.route queries in project history, current dispatch is human-directed (names target explicitly), T-237 RPC exists but unused
- **Valid finding preserved:** T-237 orchestrator.route RPC is built and passing — the upstream that the cache caches FROM is already in place

### 2026-03-24T08:00:00Z — Reversed to GO (human decision)
- **Chose:** GO — build the route cache
- **Why:** T-239 is the middle layer of the T-233 capability discovery system (D-007): bypass registry (done) → **route cache** → orchestrator.route (done). Without the cache, every non-bypass interaction hits the orchestrator RPC — no learning, no progressive autonomy. The 3-way branch (hit → direct, partial match → refinement query, miss → full discovery) is what makes the system practical at scale. T-237 RPC is already built; the cache completes the progressive autonomy path: first use = full discovery, second use = cached route, nth use = local bypass. Design validated in T-233 Q2b-routing-decision.md: YAML entries with confidence scores, TTL, hit counts, lazy invalidation via schema validation, confidence decay (0.05/week of non-use).
- **Rejected:** Original NO-GO — treated absence of current routing lookups as evidence they'll never exist, rather than recognizing the cache as infrastructure that enables them. Evaluated T-239 as isolated feature rather than building block of approved architecture (T-258 root cause analysis).

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

### 2026-03-24T08:00:00Z — reopened [human decision]
- **Action:** NO-GO reversed to GO by human
- **Reason:** T-239 is the middle layer of T-233 capability discovery (D-007). Completes the progressive autonomy path between bypass registry (done) and orchestrator.route RPC (done).
- **Context:** T-258 context amnesia investigation revealed NO-GO was based on missing architectural context
