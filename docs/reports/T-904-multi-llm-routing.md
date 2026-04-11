# T-904: Multi-LLM Model Routing for Dispatch

**Status:** implementation complete
**Related:** T-902 (MCP task-gate), T-903 (task-type routing)

## Summary

Extends `termlink dispatch` and the `termlink_dispatch` MCP tool with an optional
`model` parameter that selects the LLM a worker should use. The model flows through
the CLI -> DispatchOpts -> worker shell template -> `TERMLINK_MODEL` env var, and is
recorded on the `DispatchRecord` in the dispatch manifest for later accounting.

Routing gains per-model success-rate tracking on the hub's route cache, and the
circuit breaker grows a model-level fallback chain (default: opus -> sonnet -> haiku)
so unavailable models degrade cleanly.

Default behavior is unchanged: omitting `--model` keeps the existing
unspecified-model path, all existing DispatchRecord call sites pass `model: None`,
and the route cache returns zeroed ModelStats for unknown keys.

## Touched files

| File | Change |
|------|--------|
| `crates/termlink-cli/src/cli.rs` | `--model` clap flag on the `Dispatch` variant |
| `crates/termlink-cli/src/main.rs` | Threads `model` into `DispatchOpts` |
| `crates/termlink-cli/src/commands/dispatch.rs` | `DispatchOpts::model`, `TERMLINK_MODEL` env export in worker shell |
| `crates/termlink-cli/src/manifest.rs` | `DispatchRecord::model: Option<String>` |
| `crates/termlink-hub/src/route_cache.rs` | `ModelStats` + `model_stats` map keyed `model:task_type`, `best_model_for_task_type`, persistence round-trip |
| `crates/termlink-hub/src/circuit_breaker.rs` | `resolve_model(preferred, fallback_chain)` + default opus->sonnet->haiku chain |
| `crates/termlink-mcp/src/tools.rs` | `DispatchParams::model`, wire to `TERMLINK_MODEL` for MCP-spawned workers |

## Tests

All tests pass against the committed tree:

- `termlink-hub` route_cache: `record_model_success`, `record_model_failure`,
  `record_model_mixed`, `model_stats_success_rate`, `model_stats_per_task_type_isolation`,
  `model_stats_empty_returns_zero`, `best_model_for_task_type`, `best_model_no_data`,
  `model_stats_persistence_round_trip`.
- `termlink-hub` circuit_breaker: `model_resolve_preferred_available`,
  `model_resolve_fallback_chain`, `model_resolve_fallback_on_failure`,
  `model_resolve_independent_models`, `default_model_fallback_chain_order`.
- `termlink-mcp` tools: `dispatch_params_with_model`, `dispatch_params_without_model`,
  `dispatch_params_model_sonnet`.

Verification run from the commit that landed this series:

- `cargo test -p termlink-hub --lib -- model` -> 19 passed.
- `cargo test -p termlink-mcp --lib -- dispatch_params` -> 9 passed.
- `cargo build --workspace` -> clean.

## Out of scope

- Actually making Claude Code respect `TERMLINK_MODEL` at worker startup is the
  consumer's job — the worker's user-command (e.g. `claude --model $TERMLINK_MODEL`)
  is the integration point. TermLink only ensures the env var is set.
- Cross-model cost/latency accounting beyond success rate. Follow-up if needed.
- Auto-promotion of a model based on `best_model_for_task_type` — the accessor
  exists and route cache records data; using it to steer dispatch is a future task.
