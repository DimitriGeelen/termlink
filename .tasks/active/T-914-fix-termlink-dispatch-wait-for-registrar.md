---
id: T-914
name: "Fix termlink dispatch wait-for-registrar hang on fast-failing user_cmd (G-002)"
description: >
  G-002 from concerns.yaml. crates/termlink-cli/src/commands/dispatch.rs:293 builds a sh-c template: 'termlink register ... &; TL_PID=$!; sleep 1; <user_cmd>; wait $TL_PID'. When user_cmd fast-fails (e.g., claude -p --dangerously-skip-permissions refuses root, exit 1 in <100ms), sh falls through to wait $TL_PID which blocks on the still-alive registrar. Dispatch sees 'ready' but times out only after --timeout seconds. Discovered 2026-04-11 during T-909 risk-eval: 3 workers appeared ready in termlink list but pstree showed no bash/claude grandchild — silent failure. Fix: capture user_cmd exit explicitly, kill registrar on non-zero, exit with user_cmd's rc. Regression test: 'termlink dispatch ... -- bash -c "exit 42"' must exit 42 within ~3s.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [termlink, dispatch, bug, observability]
components: []
related_tasks: []
created: 2026-04-11T12:30:39Z
last_update: 2026-04-11T12:55:11Z
date_finished: null
---

# T-914: Fix termlink dispatch wait-for-registrar hang on fast-failing user_cmd (G-002)

## Context

G-002 root cause: `dispatch.rs` builds a worker shell template ending in `wait $TL_PID`. The `termlink register` registrar is long-lived (waits for orchestrator instructions). When `user_cmd` fast-fails (e.g., `claude -p --dangerously-skip-permissions` refuses root and exits in <100ms), sh falls through to `wait $TL_PID`, which blocks on the still-alive registrar. Worker process never dies, `manager::find_session(name).is_err()` never returns true, so dispatch's existing early-crash detection never fires. Dispatch hangs in collect-loop until `--timeout` expires.

Fix: capture user_cmd's exit code, kill the registrar, exit with user_cmd's rc. This makes the worker process die promptly on user_cmd failure, which lets the existing crash-detection code path do its job.

## Acceptance Criteria

### Agent
- [ ] `dispatch.rs` worker shell template captures user_cmd exit code via `$?` after user_cmd runs
- [ ] Template explicitly terminates the registrar (`kill $TL_PID`) after user_cmd finishes — both success and fail paths
- [ ] Worker exits with user_cmd's exit code (`exit $USER_RC`), not the registrar's
- [ ] Shell-template construction is extracted into a unit-testable helper (`build_worker_shell_cmd`)
- [ ] Unit test asserts the helper output contains `USER_RC=$?`, `kill $TL_PID`, and `exit $USER_RC`
- [ ] Unit test asserts the helper output's last line is `exit $USER_RC` (regression for the `wait $TL_PID` hang)
- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --package termlink-cli dispatch::tests` passes

### Human
- [ ] [REVIEW] Smoke-test the fix end-to-end with a fast-failing user_cmd
  **Steps:**
  1. From `/opt/termlink`: `cargo build --release && sudo install -m 755 target/release/termlink /usr/local/bin/termlink`
  2. Ensure `termlink hub` is running
  3. Run: `time termlink dispatch --count 1 --backend background --timeout 30 --json -- bash -c 'exit 42'`
  **Expected:** Returns within ~5s (not 30s). JSON output shows `crashed_workers` populated.
  **If not:** The `find_session` early-crash check is not triggering. Check `manager::find_session` semantics — it may need a tighter check than process-presence.

## Verification

# Build and run unit tests on the dispatch helper
cargo build --workspace --quiet
cargo test --package termlink-cli --lib dispatch::tests::worker_shell_cmd --quiet

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

### 2026-04-11T12:30:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-914-fix-termlink-dispatch-wait-for-registrar.md
- **Context:** Initial task creation

### 2026-04-11T12:55:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
