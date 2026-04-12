---
id: T-927
name: "termlink signal --target — third rollout"
description: >
  termlink signal --target — third rollout

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T21:25:15Z
last_update: 2026-04-11T21:27:42Z
date_finished: 2026-04-11T21:27:42Z
---

# T-927: termlink signal --target — third rollout

## Context

Third per-command rollout of T-924's `TargetOpts` + `call_session`. Signal
is `command.signal` — requires `Control` scope
(`auth::method_scope(COMMAND_SIGNAL) == Control`). Because the signal
command has a **required** positional `target: String` (not `Option<String>`
like ping/status), the main.rs dispatch is simpler — no interactive-picker
fallback.

Small scope extension: `default_scope_for` in `target.rs` currently lists
`session.spawn` / `session.stop` → control but not `command.signal`. Add
`command.signal` to the control tier so a user running `termlink signal
SESS TERM --target host:4112` without `--scope` gets the right default
and is not rejected by the hub's scope gate.

## Acceptance Criteria

### Agent
- [x] `Signal` variant in `cli.rs` gains `hub`/`--target`, `secret_file`,
      `secret`, `scope` routing fields.
- [x] `cmd_signal` in `commands/session.rs` takes `&TargetOpts` and routes
      through `call_session` for both local and cross-host paths. Existing
      signal-name parsing (`parse_signal`) is preserved.
- [x] `default_scope_for` in `target.rs` classifies `command.signal` as
      `control` (matches session-side `auth::method_scope`).
- [x] New assertion added to `default_scope_matches_per_method_semantics`:
      `default_scope_for("command.signal") == "control"`. 20/20 tests pass.
- [x] `cargo build --workspace` clean, no new warnings.
- [x] `cargo test -p termlink --bin termlink -- target::` passes (20/20).
- [x] `termlink signal --help` shows the four new flags and "control for
      signal" default-scope hint.
- [x] Existing `cargo test -p termlink -- signal` tests still pass
      (cli_signal_not_found + 5 unrelated → all pass).

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

cargo build --workspace
cargo test -p termlink --bin termlink -- target::
./target/debug/termlink signal --help 2>&1 | grep -q -- --target
./target/debug/termlink signal --help 2>&1 | grep -q -- --secret-file

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

### 2026-04-11T21:25:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-927-termlink-signal---target--third-rollout.md
- **Context:** Initial task creation

### 2026-04-11T21:27:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
