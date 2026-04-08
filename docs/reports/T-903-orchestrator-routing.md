# T-903: Extend orchestrator.route with task-type routing

## Summary

Added optional `task_type` parameter to the `orchestrator.route` RPC, enabling the routing
chain to prefer specialists tagged for specific workflow types (build, test, audit, review).

## Changes

### File: `crates/termlink-hub/src/router.rs`

**1. Task-type extraction (line ~685)**
- New optional `task_type` string extracted from params
- Composite `routing_key` built as `method::task_type` (or just `method` when absent)

**2. Bypass registry (Layer 1)**
- Bypass check uses `routing_key` so task-type-specific commands earn separate Tier 3 promotions
- Bypass response includes `task_type` field when present

**3. Route cache (Layer 2)**
- Cache lookup, hit recording, and invalidation all use `routing_key`
- This means `termlink.ping` and `termlink.ping::build` are cached independently

**4. Discovery (Layer 3)**
- After collecting candidates from selector filters, candidates are stable-sorted
  by task-type tag match: sessions with `task-type:<type>` tag sort first
- Fallback: when no specialist has the matching tag, all candidates remain eligible

**5. Success/failure tracking**
- All bypass registry updates (success, command failure, infra failure) use `routing_key`

## Design decisions

- **Tag convention**: `task-type:<type>` (e.g., `task-type:build`) — simple, filterable, no schema changes to Registration
- **Composite key**: `method::task_type` for cache/bypass — separates task-type-specific routes without requiring new data structures
- **Preference not exclusion**: task-type sorts candidates but never filters them out — fallback to any available specialist is always possible
- **Backward compatible**: all existing routing works unchanged when `task_type` is absent

## Tests added

| Test | Purpose |
|------|---------|
| `orchestrator_route_task_type_prefers_tagged_specialist` | With task_type=build, build-specialist is preferred over generic |
| `orchestrator_route_task_type_falls_back_when_no_match` | With task_type=audit but only test-specialist available, routes successfully |
| `orchestrator_route_no_task_type_backward_compatible` | Without task_type, routing works normally with both tagged and untagged specialists |

All 155 hub tests pass.

## Helper added

`start_test_session_with_tags()` — creates test sessions with custom tags for task-type routing tests.
