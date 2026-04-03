# Enhancement: Add `cwd` parameter to `termlink_spawn`

**Source:** Project 050-email-archive, Task T-400
**Priority:** P2 | **Date:** 2026-03-30

## Problem
`termlink_spawn` command array runs in default directory. No way to set cwd for spawned processes. Parallel agents dispatched to git worktrees silently run in wrong location.

## Workaround
Spawn bash shell, then `termlink_interact` with `cd /target && command`. Adds latency.

## Proposed
Add optional `cwd: Option<String>` to `SpawnParams` in termlink_spawn.

## Additional Findings (2026-03-30)

### termlink_exec timeout ≠ failure
When a command completes successfully but after the timeout, exec returns an error. The work is done but the caller thinks it failed. Enhancement: return partial/status instead of hard error.

### MCP spawn vs CLI spawn behavior mismatch
`termlink_spawn` via MCP with tags/roles produced sessions that couldn't be found by name. `termlink run "termlink spawn --name X -- bash"` via CLI worked correctly. The MCP path may have a registration race condition.
