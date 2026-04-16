---
id: T-916
name: "Fix dispatch event.collect masking hub failures via continue path"
description: >
  Discovered 2026-04-11 while smoke-testing T-914 (G-002 fix). When the hub is down or unreachable, dispatch's collect loop in crates/termlink-cli/src/commands/dispatch.rs (around line 406) hits Connection refused on every event.collect RPC and falls through a 'continue' statement that skips the entire rest of the loop body — including the early-crash detection at lines 454-480. Result: dispatch hangs in a tight error loop until --timeout expires, with no signal to the user that the hub is unreachable. Confirmed reproduction: repeated 'I/O error: Connection refused (os error 111)' debug lines during a smoke test where the hub PID was dead but the socket file persisted on disk. RECOMMENDED FIX: move the early-crash check to the TOP of the collect loop (before event.collect) so it always runs regardless of RPC outcome. Additionally, track consecutive event.collect errors and bail with a clear 'hub unreachable' error after N consecutive failures (e.g., 5). PRE-FLIGHT OPTION: ping the hub once before entering the collect loop and fail fast with a clear error message. Symptom is identical to the G-002 fast-fail hang from the user's perspective (silent timeout) but the cause and fix are different. T-914 fix is correct; this is a separate orthogonal bug.

status: work-completed
workflow_type: build
owner: human
horizon: later
tags: [termlink, dispatch, bug, observability, error-handling]
components: [crates/termlink-cli/src/commands/dispatch.rs]
related_tasks: [T-914, T-282]
created: 2026-04-11T13:16:45Z
last_update: 2026-04-16T05:40:16Z
date_finished: 2026-04-11T13:30:15Z
---

# T-916: Fix dispatch event.collect masking hub failures via continue path

## Context

Two layers of defense are needed:

1. **Pre-flight liveness check** at dispatch startup. The current check at `dispatch.rs:93-99` only tests `hub_socket.exists()` (file existence), which passes even when the hub PROCESS is dead and only the stale socket file remains. Replace with an actual `UnixStream::connect` so a dead-hub-with-stale-socket fails fast with the existing "Hub is not running" error.

2. **Mid-loop resilience** in the collect loop. When event.collect errors (e.g., hub dies mid-dispatch, transient connectivity), the current `continue` statement masks the failure and skips early-crash detection. Track consecutive errors and bail with a clear message after N (5) failures. Reset counter on success.

## Acceptance Criteria

### Agent
- [x] Pre-flight check at `dispatch.rs:92-99` actually opens a connection to the hub socket (not just file-existence check)
- [x] Pre-flight error message remains backwards-compatible: still contains "Hub is not running" so existing tests (`workdir_none_accepted`, etc.) keep passing
- [x] Collect loop tracks `consecutive_collect_errors` counter; resets on each successful RPC
- [x] After 5 consecutive `event.collect` errors, the loop emits a clear "Hub unreachable" warning and breaks
- [x] All existing dispatch tests still pass (11/11)
- [x] `cargo build --workspace` succeeds
- [x] `cargo test --package termlink --bin termlink commands::dispatch::tests` passes
- [x] Real-world verified: dispatch with dead hub returns in 12ms instead of 30s (`time ./target/debug/termlink dispatch ... -- bash -c 'exit 0'`)

### Human
- [ ] [REVIEW] Confirm dispatch fails fast (within ~3s) when hub is dead
  **Steps:**
  1. Stop the hub if running: `pkill -f "termlink hub"`
  2. Confirm socket file persists: `ls -la /tmp/termlink-0/hub.sock` (should still show the .sock file)
  3. Run: `time ./target/debug/termlink dispatch --count 1 --backend background --timeout 30 -- bash -c 'exit 0'`
  **Expected:** Returns within ~1s with error message containing "Hub is not running" — does NOT enter the 30s collect loop.
  **If not:** Pre-flight check is not detecting the dead hub properly. Verify `UnixStream::connect` is being awaited and not silently swallowed.

## Verification

# Build and run all dispatch tests.
cargo build --workspace --quiet
cargo test --package termlink --bin termlink commands::dispatch::tests -- --quiet

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

### 2026-04-11T13:16:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-916-fix-dispatch-eventcollect-masking-hub-fa.md
- **Context:** Initial task creation

### 2026-04-11T13:26:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-11T13:30:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-04-16T05:40:16Z — status-update [task-update-agent]
- **Change:** horizon: now → later
