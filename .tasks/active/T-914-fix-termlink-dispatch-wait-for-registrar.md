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
- [x] `dispatch.rs` worker shell template captures user_cmd exit code via `$?` after user_cmd runs
- [x] Template explicitly terminates the registrar (`kill $TL_PID`) after user_cmd finishes — both success and fail paths
- [x] Worker exits with user_cmd's exit code (`exit $USER_RC`), not the registrar's
- [x] Shell-template construction is extracted into a unit-testable helper (`build_worker_shell_cmd`)
- [x] Unit test asserts the helper output contains `USER_RC=$?`, `kill $TL_PID`, and `exit $USER_RC`
- [x] Unit test asserts the helper output's last line is `exit $USER_RC` (regression for the `wait $TL_PID` hang)
- [x] `cargo build --workspace` succeeds
- [x] `cargo test --package termlink --bin termlink commands::dispatch::tests` passes (11/11)

### Human
- [ ] [REVIEW] Smoke-test the fix end-to-end with a fast-failing user_cmd (REQUIRES T-916 fix or healthy hub)
  **Steps:**
  1. **Confirm the hub is alive** first: `pgrep -af "termlink hub"` should show a process. If not: `termlink hub &` (warning: may interfere with the t11xx-rca workers from the framework session — coordinate first).
  2. From `/opt/termlink`: `cargo build --release && sudo install -m 755 target/release/termlink /usr/local/bin/termlink` (or use `./target/debug/termlink` directly).
  3. Run: `time termlink dispatch --count 1 --backend background --timeout 30 --json -- bash -c 'exit 42'`
  **Expected:** Returns within ~5s (not 30s). JSON output shows `crashed_workers` populated and `elapsed_secs` < 10.
  **If not:** Likely T-916 (event.collect masking hub failure) is biting — check stderr for "Connection refused" and verify hub is actually responding. The G-002 fix itself was independently verified via `/tmp/t914-manual-test.sh` and `/tmp/t914-dispatch-watch.sh` (worker dies on schedule). See Decisions section.

## Verification

# Build and run unit tests on the dispatch helper.
# The crate is named `termlink` (binary), not `termlink-cli` — see Cargo.toml.
cargo build --workspace --quiet
cargo test --package termlink --bin termlink commands::dispatch::tests::worker_shell_cmd_captures_exit_kills_registrar -- --quiet
cargo test --package termlink --bin termlink commands::dispatch::tests::worker_shell_cmd_last_line_is_exit_user_rc -- --quiet

## Decisions

### 2026-04-11 — Smoke test exposed orthogonal bug (filed as T-916)

- **What was tried:** Built debug binary, ran `termlink dispatch --count 1 --backend background --timeout 10 -- bash -c 'exit 42'` to verify wall-clock improvement.
- **Result:** Dispatch still hung for the full timeout. NOT because the G-002 fix failed (it didn't — see below), but because the hub PID 1517402 was dead in this environment and `event.collect` returned "Connection refused (os error 111)" on every iteration. The collect loop's `continue` on RPC error path never reaches the early-crash detection.
- **Verification that the G-002 fix IS correct:** (a) 2/2 unit tests on `build_worker_shell_cmd` pass; (b) manual test (`/tmp/t914-manual-test.sh`) confirmed `kill $TL_PID` cleanly terminates the registrar via SIGTERM and `termlink list` correctly removes the dead session within 1s; (c) watch test (`/tmp/t914-dispatch-watch.sh`) showed the dispatched worker process tree was empty by T+5s, confirming the worker dies on schedule per my fix.
- **What blocks full end-to-end smoke test:** environmental hub being down. Full validation requires: restart hub + re-run dispatch + observe `crashed_workers` populated and elapsed_secs<10. Not done in-session because another active session has 5 t11xx-rca workers running against this same socket directory and a hub restart could disturb them.
- **Filed T-916** for the orthogonal bug: dispatch's event.collect continue-on-error path silently masks hub failures into infinite hangs.

## Updates

### 2026-04-11T12:30:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-914-fix-termlink-dispatch-wait-for-registrar.md
- **Context:** Initial task creation

### 2026-04-11T12:55:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
