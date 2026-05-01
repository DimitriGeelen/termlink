# U-005 — Populate `model_used` / `fallback_used` in dispatch worker meta.json

**Task:** T-1442 — Populate model_used and fallback_used in dispatch meta.json
**Workflow:** build
**Status:** work-completed (pending Human REVIEW)

## Summary

The framework's `agents/termlink/termlink.sh` (T-1643/W4) writes `meta.json`
with `model_used: null` / `fallback_used: null`, deferring those fields to
the substrate. /opt/termlink's substrate-side dispatcher
(`scripts/tl-dispatch.sh`) is the dual that owns the resolution decision and
must populate them. Until this closure, Watchtower's
`/orchestrator` "Recent dispatches" panel renders `n/a` and the audit
detective WARN's the orchestrator-rethink arc as code-complete-without-closure.

This task closes the loop on the substrate side.

## Files changed

- `scripts/tl-dispatch.sh`
  - `cmd_spawn` accepts `--model` and `--task-type` flags.
  - New helper `_resolve_dispatch_model "<explicit>" "<task_type>"` returns
    `<model>|<fallback_used>` using the explicit → per-type
    (`DISPATCH_MODEL_FOR_<TYPE>`) → default (`DISPATCH_MODEL_DEFAULT`) →
    none chain. Mirrors the framework's resolution order so both halves
    converge on the same answer for a given inputs.
  - New helpers `_json_str_or_null` / `_json_bool_or_null` for honest JSON
    serialization (empty resolution → `null`, not the string `"null"`).
  - `meta.json` template updated to emit `task_type`, `model`,
    `model_used`, `fallback_used` keys with the resolved values.
  - The resolved model is propagated into the inner `run.sh` via the
    spawn-args tail and rendered as `--model <m>` only when non-empty.
- `tests/test_tl_dispatch_meta.sh` (new)
  - Pin 1: static check that the meta.json template includes the three
    orchestrator-substrate keys.
  - Pin 2: six assertions across all four `_resolve_dispatch_model`
    branches (explicit / per-type / default / none) plus precedence
    (per-type beats default, explicit beats both).
  - Pin 3: end-to-end `cmd_spawn` invocation with stubbed `termlink`,
    asserting the four expected `(model_used, fallback_used)` tuples
    materialize on disk and have the correct JSON types
    (`string` / `bool` / `null`).

## Verification (per task `## Verification` block)

| Command                                | Result |
| -------------------------------------- | ------ |
| `bash -n scripts/tl-dispatch.sh`       | OK     |
| `bash tests/test_tl_dispatch_meta.sh`  | 21 pass / 0 fail |
| `cargo check --workspace`              | OK     |

## Acceptance Criteria status

All 9 Agent ACs ticked. Human `[REVIEW]` AC pending live spot-check
(steps in task file): build the CLI, start a hub, run a real spawn,
inspect meta.json. The regression test exercises the same code path
end-to-end with stubbed termlink, so any divergence in the live case
points to spawn-arg plumbing.

## Closure note for upstream framework

`fw pending resolve U-005 --note '/opt/termlink T-1442 — meta.json
populated in scripts/tl-dispatch.sh; regression in tests/test_tl_dispatch_meta.sh
(21/21 pass); see /opt/termlink/docs/reports/U-005-meta-populate.md'`

## Commit

(SHA recorded post-commit in `git log` for T-1442.)
