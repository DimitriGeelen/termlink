---
id: T-926
name: "termlink status --target — second --target rollout"
description: >
  termlink status --target — second --target rollout

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T21:22:26Z
last_update: 2026-04-11T21:24:37Z
date_finished: 2026-04-11T21:24:37Z
---

# T-926: termlink status --target — second --target rollout

## Context

Second per-command rollout of T-924's `TargetOpts` + `call_session` helper.
Mirrors the T-925 pattern exactly — `termlink status` gets four new flags
(`--target`, `--secret-file`, `--secret`, `--scope`) and `cmd_status` routes
through `call_session` for both local and cross-host paths. No new design
decisions vs. T-925 — this task just exercises the pattern on a second
command to prove the helper composes cleanly.

## Acceptance Criteria

### Agent
- [x] `Status` variant in `cli.rs` gains four cross-host routing fields
      (`hub`/`--target`, `secret_file`, `secret`, `scope`) alongside the
      existing positional session argument.
- [x] `cmd_status` in `commands/session.rs` takes `&TargetOpts` and routes
      through `call_session` for both local and cross-host paths.
- [x] `cmd_status` preserves the existing text / `--short` / JSON output
      formats so scripts keep working.
- [x] `cargo build --workspace` clean, no new warnings.
- [x] `termlink status --help` shows the four new flags.
- [x] Existing `cargo test -p termlink -- status` tests still pass (9/9).
- [x] Smoke test: `termlink status t1109-l006-sweep --short` →
      `t1109-l006-sweep ready 129536` on the local path.

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
./target/debug/termlink status --help 2>&1 | grep -q -- --target
./target/debug/termlink status --help 2>&1 | grep -q -- --secret-file
./target/debug/termlink status --help 2>&1 | grep -q -- --scope

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

### 2026-04-11T21:22:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-926-termlink-status---target--second---targe.md
- **Context:** Initial task creation

### 2026-04-11T21:24:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
