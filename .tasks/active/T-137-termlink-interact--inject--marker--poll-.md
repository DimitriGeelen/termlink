---
id: T-137
name: "termlink interact — inject + marker + poll wrapper"
description: >
  CLI command that injects a command into a PTY session, waits for completion
  via marker detection, and returns the output. Wraps inject + poll into one call.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [cli, interactive, self-test]
components: []
related_tasks: [T-136]
created: 2026-03-14T17:05:41Z
last_update: 2026-03-14T17:05:41Z
date_finished: null
---

# T-137: termlink interact — inject + marker + poll wrapper

## Context

Phase 1 from T-136 inception (GO). The T-136 spike proved that inject + sleep +
query.output works, but requires 3 separate CLI calls + manual synchronization.
This task wraps that into a single `termlink interact <session> <command>` that:

1. Injects `<command>; echo ___MARKER_<uuid>___` into the PTY
2. Polls `query.output` until the marker appears (or timeout)
3. Returns the output between the command and the marker
4. Optionally strips ANSI escape sequences (`--strip-ansi`)

## Acceptance Criteria

### Agent
- [ ] `termlink interact <session> <command>` CLI subcommand implemented
- [ ] Marker-based synchronization: injects command + unique marker, polls until marker found
- [ ] Configurable timeout (`--timeout <secs>`, default 30)
- [ ] `--strip-ansi` flag strips ANSI escape sequences from output
- [ ] `--json` flag returns structured JSON `{output, exit_marker_found, elapsed_ms}`
- [ ] Poll interval configurable (`--poll-ms <ms>`, default 200)
- [ ] All existing tests pass (253+)
- [ ] New tests for interact command (at least 2: basic command, timeout)

### Human
- [ ] [REVIEW] Run `termlink interact <session> "fw doctor"` and verify output matches what you'd see in a terminal
  **Steps:**
  1. `termlink register --name test --shell` (in another terminal)
  2. `termlink interact test "ls -la"`
  3. `termlink interact test "fw doctor" --strip-ansi`
  **Expected:** Clean output matching the command result
  **If not:** Check if marker detection failed or output is truncated

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | grep -q "test result: ok"
/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | grep -qv "^error"

## Updates

### 2026-03-14T17:05:41Z — task-created
- Phase 1 build task from T-136 inception
