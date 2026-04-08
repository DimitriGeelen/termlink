# T-902: MCP Task-Gate Governance Checks

## Summary

Added opt-in task governance enforcement to four TermLink MCP tools:
`termlink_exec`, `termlink_spawn`, `termlink_interact`, and `termlink_dispatch`.

## What Changed

### New: Governance gate function (`check_task_governance`)
- Reads `TERMLINK_TASK_GOVERNANCE` env var
- When set to `"1"`: requires `task_id` parameter on governed tools
- When not set or any other value: allows all calls (backward compatible)
- Returns structured JSON error with clear remediation instructions

### Modified: Parameter structs
Added `task_id: Option<String>` to:
- `ExecParams`
- `SpawnParams`
- `InteractParams`
- `DispatchParams`

### Modified: Tool handlers
- All four tools check governance gate before processing
- `termlink_spawn`: passes `task_id` as `task:<id>` tag on spawned sessions
- `termlink_dispatch`: passes `task_id` as `task:<id>` tag on worker sessions
- `termlink_exec` and `termlink_interact`: governance check only (no session creation)

### New: Tests (16 added, 76 total unit + 98 integration = 174 pass)
- `governance_disabled_allows_without_task_id`
- `governance_disabled_allows_with_task_id`
- `governance_enabled_blocks_without_task_id`
- `governance_enabled_allows_with_task_id`
- `governance_enabled_blocks_empty_task_id`
- `governance_enabled_blocks_whitespace_task_id`
- `governance_other_values_treated_as_disabled`
- `governance_error_is_valid_json`
- `exec_params_with_task_id` / `exec_params_without_task_id`
- `spawn_params_with_task_id` / `spawn_params_without_task_id`
- `interact_params_with_task_id` / `interact_params_without_task_id`
- `dispatch_params_with_task_id` / `dispatch_params_without_task_id`

## Behavioral Contract

| Scenario | TERMLINK_TASK_GOVERNANCE unset | TERMLINK_TASK_GOVERNANCE=1 |
|----------|-------------------------------|----------------------------|
| No task_id | Allowed | Blocked (JSON error) |
| task_id present | Allowed | Allowed |
| Empty/whitespace task_id | Allowed | Blocked (JSON error) |

## Error Format

When blocked, tools return:
```json
{
  "ok": false,
  "error": "Task governance is enabled (TERMLINK_TASK_GOVERNANCE=1). The 'termlink_spawn' tool requires a 'task_id' parameter. Provide the task ID of the task you are working on (e.g., \"task_id\": \"T-123\")."
}
```

## Files Modified

- `crates/termlink-mcp/src/tools.rs` — governance function, param structs, tool handlers, tests
- `.tasks/active/T-902-add-mcp-task-gate-governance-checks.md` — task file
- `docs/reports/T-902-mcp-governance.md` — this report
