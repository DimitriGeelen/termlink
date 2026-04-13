---
id: T-1033
name: "Add tests for resolve_hub_paths — split-brain runtime dir logic"
description: >
  Add tests for resolve_hub_paths — split-brain runtime dir logic

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T18:22:06Z
last_update: 2026-04-13T18:22:06Z
date_finished: null
---

# T-1033: Add tests for resolve_hub_paths — split-brain runtime dir logic

## Context

resolve_hub_paths() in infrastructure.rs (T-1031/T-1032) has zero test coverage. It handles split-brain runtime dir detection — critical for systemd-managed hubs at /var/lib/termlink vs default /tmp/termlink-0. Also connect_addr_raw() in client.rs (T-1032) has no unit tests.

## Acceptance Criteria

### Agent
- [x] Integration test: hub status --short returns "not_running" when no hub in isolated dir
- [x] Integration test: hub status --json with isolated TERMLINK_RUNTIME_DIR does not discover /var/lib/termlink hub
- [x] Integration test: hub status --json detects stale pidfile correctly
- [x] Integration test: hub status --check + --short combined works
- [x] Unit test: connect_addr_raw creates working client for Unix socket connections
- [x] All new tests pass alongside existing test suite (0 failures)

### Human
- [ ] [RUBBER-STAMP] Verify test count increased
  **Steps:** `cd /opt/termlink && cargo test --workspace 2>&1 | grep "test result:" | head -10`
  **Expected:** Total test count higher than previous (was ~1002)
  **If not:** Check for compilation errors

## Verification

cargo test -p termlink --test cli_integration cli_hub_status_short 2>&1 | grep -q "test result: ok"
cargo test -p termlink --test cli_integration cli_hub_status_isolated 2>&1 | grep -q "test result: ok"
cargo test -p termlink --test cli_integration cli_hub_status_stale 2>&1 | grep -q "test result: ok"
cargo test -p termlink-session client::tests::connect_addr_raw 2>&1 | grep -q "test result: ok"

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-13T18:22:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1033-add-tests-for-resolvehubpaths--split-bra.md
- **Context:** Initial task creation
