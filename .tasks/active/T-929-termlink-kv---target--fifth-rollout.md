---
id: T-929
name: "termlink kv --target — fifth rollout"
description: >
  termlink kv --target — fifth rollout

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T21:31:48Z
last_update: 2026-04-11T21:31:48Z
date_finished: null
---

# T-929: termlink kv --target — fifth rollout

## Context

Fifth per-command rollout. `termlink kv` has four sub-actions (`get` /
`set` / `list` / `del`) that issue four different RPCs (`kv.get` / `kv.set`
/ `kv.list` / `kv.delete`). In the current implementation each branch
hand-rolls the same `client::rpc_call + timeout + json_error_exit` 25-line
boilerplate.

Scope for this task:
1. Route all four branches through `call_session` via a tiny local helper
   that collapses the boilerplate.
2. Preserve every existing output format (text + `--json` + `--raw` +
   `--keys`).
3. Wire the four cross-host routing fields into the `Kv` CLI variant.

Scopes per action (already in `default_scope_for`): `kv.get` and
`kv.list` → observe, `kv.set` and `kv.delete` → interact.

## Acceptance Criteria

### Agent
- [x] `Kv` variant in `cli.rs` gains `hub`/`--target`, `secret_file`,
      `secret`, `scope` routing fields.
- [x] `cmd_kv` in `commands/metadata.rs` takes `&TargetOpts` and routes
      all four action branches through `call_session` (via a local
      `call()` helper that collapses timeout + json_error_exit boilerplate).
- [x] Output formats preserved for every action and every flag combination
      (text / `--json` / `--raw` / `--keys`).
- [x] `Get` and `Del` still exit 1 with the "Key '{}' not found" stderr
      message when the key is absent (preserved via `std::process::exit(1)`).
- [x] `cargo build --workspace` clean, no new warnings (removed now-unused
      `use termlink_session::client;` import from metadata.rs).
- [x] `termlink kv --help` shows the four new flags.
- [x] Existing `cargo test -p termlink -- kv` tests still pass (3/3:
      cli_kv_json_value, cli_kv_json_set_get_list_del, cli_kv_set_get_list_del).

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
./target/debug/termlink kv --help 2>&1 | grep -q -- --target
./target/debug/termlink kv --help 2>&1 | grep -q -- --secret-file
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

### 2026-04-11T21:31:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-929-termlink-kv---target--fifth-rollout.md
- **Context:** Initial task creation
