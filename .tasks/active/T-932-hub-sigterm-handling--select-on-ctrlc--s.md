---
id: T-932
name: "Hub SIGTERM handling — select! on ctrl_c + SignalKind::terminate"
description: >
  cmd_hub_start in crates/termlink-cli/src/commands/infrastructure.rs:56 only listens on tokio::signal::ctrl_c() which is SIGINT-only. SIGTERM from systemctl stop falls through without triggering handle.shutdown(), skipping clean-shutdown cleanup (socket/secret/pidfile removal). Fix: select! on ctrl_c + tokio::signal::unix::SignalKind::terminate(). Add unit test that spawns hub, sends SIGTERM, verifies runtime dir is cleaned. Unblocks removing KillSignal=SIGINT from T-931 unit file. From T-930 decomposition.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-930, T-931]
created: 2026-04-11T22:29:21Z
last_update: 2026-04-11T22:49:54Z
date_finished: null
---

# T-932: Hub SIGTERM handling — select! on ctrl_c + SignalKind::terminate

## Context

T-930 Spike 2 discovered that `cmd_hub_start` only listens on
`tokio::signal::ctrl_c()` (SIGINT). SIGTERM from `systemctl stop` falls
through without calling `handle.shutdown()`, so clean-shutdown cleanup
(socket, secret, pidfile, TLS) is skipped. T-931's unit file works around
this with `KillSignal=SIGINT`; this task deletes that workaround.

## Acceptance Criteria

### Agent
- [x] `cmd_hub_start` in `crates/termlink-cli/src/commands/infrastructure.rs` uses `tokio::select!` on both `ctrl_c()` and `SignalKind::terminate()` — either signal triggers graceful shutdown via `handle.shutdown()`. (via `wait_for_shutdown_signal()` helper, commit f62821a)
- [x] `cargo build --workspace` succeeds (no new warnings).
- [x] `cargo test -p termlink` existing tests pass (no regressions). 165/165.
- [x] `.context/systemd/termlink-hub.service` has the `KillSignal=SIGINT` line removed with its accompanying comment block.
- [x] Live test on .107: direct `kill -TERM $MainPID` triggered `Hub shut down cleanly` in journalctl via the select! path; `systemctl restart termlink-hub` also works via ExecStop; `ss -tln | grep 9100` shows listener after restart.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-11T22:29:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-932-hub-sigterm-handling--select-on-ctrlc--s.md
- **Context:** Initial task creation

### 2026-04-11T22:41:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
