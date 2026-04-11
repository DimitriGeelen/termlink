# T-906: Add model parameter to dispatch

**Status:** DONE — model parameter already fully implemented.

## Findings

The model parameter was implemented as part of prior work. All five integration points are complete:

| Component | File | Status |
|-----------|------|--------|
| MCP tool param | `crates/termlink-mcp/src/tools.rs:400-403` (DispatchParams.model) | Done |
| MCP env passthrough | `crates/termlink-mcp/src/tools.rs:2132-2133` (TERMLINK_MODEL) | Done |
| CLI --model flag | `crates/termlink-cli/src/cli.rs:760-762` | Done |
| CLI env passthrough | `crates/termlink-cli/src/commands/dispatch.rs:256-260` (TERMLINK_MODEL) | Done |
| Manifest recording | `crates/termlink-cli/src/manifest.rs:33-35` (DispatchRecord.model) | Done |

## Tests (all passing)

- `dispatch_params_with_model` — opus model parsed correctly
- `dispatch_params_without_model` — None when omitted
- `dispatch_params_model_sonnet` — sonnet model with task_id combo

## How it works

When `--model` (CLI) or `"model"` (MCP) is specified:
1. Set as `TERMLINK_MODEL` env var in the spawned worker shell
2. Recorded in the dispatch manifest for tracking
3. Default (None) = no env var set = current behavior unchanged
