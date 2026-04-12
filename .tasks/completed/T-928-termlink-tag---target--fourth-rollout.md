---
id: T-928
name: "termlink tag --target — fourth rollout"
description: >
  termlink tag --target — fourth rollout

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T21:28:18Z
last_update: 2026-04-11T21:31:02Z
date_finished: 2026-04-11T21:31:02Z
---

# T-928: termlink tag --target — fourth rollout

## Context

Fourth per-command rollout of T-924's `TargetOpts` + `call_session` helper.
`termlink tag` is slightly more complex than ping/status/signal because it
issues **two** different RPCs depending on flags:

- **Read mode** (no `--set`/`--add`/`--remove`/`--name`/`--role*`): calls
  `termlink.ping` (observe scope) and prints tags+roles+display_name.
- **Write mode** (any mutation flag set): calls `session.update` (interact
  scope) with the computed delta params.

Both paths route through `call_session`; the helper's
`default_scope_for(method)` already knows `termlink.ping → observe` and
`session.update → interact`, so this rollout is purely plumbing.

## Acceptance Criteria

### Agent
- [x] `Tag` variant in `cli.rs` gains `hub`/`--target`, `secret_file`,
      `secret`, `scope` routing fields.
- [x] `cmd_tag` in `commands/metadata.rs` takes `&TargetOpts` and routes
      both the read path (`termlink.ping`) and the write path
      (`session.update`) through `call_session`.
- [x] Read-mode output (text + JSON) preserved.
- [x] Write-mode output (text + JSON pretty) preserved.
- [x] `cargo build --workspace` clean, no new warnings.
- [x] `termlink tag --help` shows the four new flags.
- [x] Existing `cargo test -p termlink -- tag` tests still pass (3/3:
      cli_tag_set_roles, cli_tag_json_output, cli_tag_rename_session).

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
./target/debug/termlink tag --help 2>&1 | grep -q -- --target
./target/debug/termlink tag --help 2>&1 | grep -q -- --secret-file
cargo test -p termlink --bin termlink -- target::

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

### 2026-04-11T21:28:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-928-termlink-tag---target--fourth-rollout.md
- **Context:** Initial task creation

### 2026-04-11T21:31:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
