# T-239: Route Cache Inception

## Problem Statement

Is a per-agent route cache (`.cache/routes/` with confidence, TTL, lazy invalidation) needed when the routing infrastructure it caches from doesn't exist?

## Findings

### The Route Cache Depends on Infrastructure That Doesn't Exist

The T-233 Q2b design assumes:

| Dependency | Status | Evidence |
|------------|--------|----------|
| Orchestrator.route RPC | **RPC exists** (T-237) but **no orchestrator dispatches routes** | Zero route queries in project history |
| Specialist discovery via tags | **TermLink discover works** but **no persistent specialists to discover** | T-241 NO-GO confirmed |
| Repeated capability-to-specialist lookups | **Zero instances** | No agent has ever resolved a capability to a specialist |
| Domain trigger taxonomy | **Designed** (T-233 Q2) but **not implemented** | No capability slug extraction exists |

### Current Dispatch Is Direct, Not Routed

Current patterns:
- **Human dispatch:** User tells agent exactly which session to target
- **Mesh dispatch:** `dispatch.sh` spawns N workers directly, no routing lookup
- **T-257 collect:** Workers emit to their own bus, orchestrator collects — no routing involved

The route cache optimizes a lookup that never happens. There's nothing to cache.

### Relationship to T-233 Child Tasks

| Task | Decision | Root Cause |
|------|----------|------------|
| T-240 (negotiation) | NO-GO | No iterative corrections exist |
| T-241 (template cache) | NO-GO | No specialist ecosystem |
| T-242 (supervision) | NO-GO | Workers already have max autonomy |
| **T-239 (route cache)** | **NO-GO** | **No routing lookups exist to cache** |

All four share the same root cause: the T-233 specialist orchestration model is well-designed but the project hasn't reached the scale where any of its optimization layers are needed.

## Decision: NO-GO

**The T-233 Q2b route cache design remains valid.** When persistent specialists exist and agents make repeated capability-to-specialist lookups, the cache will deliver value. Signal to revisit: >10 orchestrator.route queries per session, indicating real routing overhead worth caching.
