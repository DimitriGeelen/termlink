# T-1590 Phase 4b — Route-Cache Model Tracking + Circuit-Breaker Fallback

**Date:** 2026-04-28
**Worker:** dispatch worker
**Status:** DONE

## Summary

Wired the previously-shipped (T-906/T-903) primitives — `RouteCache::record_model_*`,
`RouteCache::best_model_for`, and `ModelCircuitBreaker::resolve_model` — into the
`termlink_dispatch` MCP tool. The dispatch path now (a) resolves an effective
model via the fallback chain when a circuit is open, (b) consults the route
cache for the best-known model when no explicit `model` is given but a
`task_type` is, (c) records per-worker outcomes against both the breaker and
the cache, and (d) surfaces the model decision in the dispatch response.

## Design Decisions

1. **Single resolver, two inputs.** Added `resolve_dispatch_model(requested,
   task_type, &cache)` returning `(Option<String>, bool)` for
   `(effective_model, fallback_used)`. Centralises three cases:
   - explicit model → run through breaker, fall through chain on open;
   - no model + task_type → look up `best_model_for(task_type)` then breaker;
   - neither → return `(None, false)` (no learning recorded, env unchanged).

2. **Outcome attribution.** A worker that emits an event on the topic with
   `payload.ok != false` counts as a success; one whose payload sets
   `ok: false`, plus any worker that crashed before emitting, counts as a
   failure. This matches existing `task.completed` payload conventions
   without requiring schema changes.

3. **Persistence is best-effort.** `route_cache.save()` errors are swallowed
   (matches the load path's lazy semantics in `RouteCache::load_from`).
   The breaker is in-memory by design.

4. **Manifest surfacing.** Rather than mutate `dispatch-manifest.json`
   (which is owned by the `--isolate` worktree path in `termlink-cli`),
   the model decision is reported back in the JSON the orchestrator already
   reads: `model_requested`, `model_used`, `fallback_used`, `task_type`.
   This is the manifest the orchestrator consumes per-call, and the
   route-cache file remains the persistent learning store.

5. **Backward compatibility.** `task_type` is `Option<String>`; existing
   `DispatchParams` JSON with no `task_type` and no `model` produces
   identical behaviour to pre-T-1590 dispatch. `model_stats` already had
   `#[serde(default)]` from T-906.

## Files Modified

- `crates/termlink-mcp/src/tools.rs`
  - `DispatchParams`: added `task_type: Option<String>` field (after `model`).
  - Added free function `resolve_dispatch_model(requested, task_type, &cache)`.
  - `termlink_dispatch`: load route cache, resolve effective model before
    spawn, use it in the `TERMLINK_MODEL` env export, record outcomes after
    collection (success/failure into both `RouteCache` and `ModelCircuitBreaker`),
    persist cache, surface `model_requested` / `model_used` /
    `fallback_used` / `task_type` in the result JSON.
  - Test module: added 5 new tests:
    - `dispatch_params_with_task_type`
    - `dispatch_params_default_task_type_none`
    - `resolve_dispatch_model_passthrough_when_breaker_closed`
    - `resolve_dispatch_model_uses_best_for_task_type`
    - `resolve_dispatch_model_no_inputs_returns_none`

No changes to `route_cache.rs` or `circuit_breaker.rs` — Phase 4a/T-906/T-903
already delivered the primitives (`ModelStats`, `record_model_success/failure`,
`best_model_for`, `ModelCircuitBreaker::{record_failure, record_success,
should_skip, resolve_model}`, `DEFAULT_MODEL_FALLBACK`). Phase 4b is the
wiring step.

## Pre-Existing Test Coverage (verified passing)

`crates/termlink-hub/src/route_cache.rs`:
- Model tracking (≥6 tests): `model_stats_success_rate`, `model_stats_empty_returns_zero`,
  `record_model_success`, `record_model_failure`, `record_model_mixed`,
  `best_model_for_task_type`, `best_model_no_data`,
  `model_stats_per_task_type_isolation`, `model_stats_persistence_round_trip`.

`crates/termlink-hub/src/circuit_breaker.rs`:
- Transitions (≥3 tests): `model_breaker_closed_by_default`,
  `model_breaker_opens_after_failures`, `model_breaker_success_closes`,
  plus session-level analogues `closed_by_default`,
  `opens_after_threshold_failures`, `success_closes_circuit`,
  `success_resets_failure_count`, `half_open_after_cooldown`.
- Fallback chain (≥2 tests): `model_resolve_preferred_available`,
  `model_resolve_fallback_on_failure`, `model_resolve_fallback_chain`,
  `model_resolve_all_unavailable`, `model_resolve_independent_models`,
  `default_model_fallback_chain_order`.

## Test Counts

Command: `cargo test -p termlink-hub -p termlink-mcp` (run from `/opt/termlink`).

| Suite                        | passed | failed | ignored |
|------------------------------|-------:|-------:|--------:|
| termlink-hub unit            | 278    | 0      | 0       |
| termlink-mcp unit            | 103    | 0      | 0       |
| mcp_integration              | 99     | 0      | 0       |
| termlink-hub doc             | 0      | 0      | 0       |
| termlink-mcp doc             | 0      | 0      | 0       |
| **Total**                    | **480**| **0**  | **0**   |

`cargo test` exit code: **0**.

## Blockers

None. Build clean, all tests green.

## Followups (out of scope)

- Plumb `task_type` from CLI `termlink dispatch` flag through to the MCP
  call (current change is server-side only; the CLI command does not yet
  forward a `--task-type` flag).
- Consider promoting model outcome recording into a dedicated event type
  on the bus so other consumers (Watchtower, fleet doctor) can react.
- Decide whether to extend `dispatch-manifest.json` (the `--isolate`
  worktree manifest) with the same model-decision fields for non-MCP
  dispatch paths.
