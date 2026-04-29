---
id: T-1406
name: "Regression test: forbid new in-repo direct legacy primitive callers during T-1166 bake"
description: >
  Regression test: forbid new in-repo direct legacy primitive callers during T-1166 bake

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T20:28:54Z
last_update: 2026-04-29T20:34:02Z
date_finished: 2026-04-29T20:34:02Z
---

# T-1406: Regression test: forbid new in-repo direct legacy primitive callers during T-1166 bake

## Context

The T-1166 bake window measures legacy-method traffic over a 60d window with a
1.0% gate. Right now the in-repo direct callers of legacy primitives are
6 known fallback paths (router, audit-list, CLI broadcast/doctor fallbacks,
MCP broadcast/doctor fallbacks, session inbox_channel.rs wrapper). A new
caller added by accident during the bake would slow the gate and contaminate
the metric. Add a structural test (Rust integration test under
`crates/termlink-hub/tests/`) that walks `crates/**/src/**/*.rs` and fails if
any quoted `"event.broadcast" / "inbox.list" / "inbox.status" / "inbox.clear"
/ "file.send" / "file.receive"` literal appears outside an explicit allowlist.

Predecessors: T-1162, T-1163, T-1164, T-1400, T-1401, T-1402, T-1403, T-1405.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/tests/no_legacy_callers.rs` exists with a
      `#[test] fn no_new_direct_legacy_callers()` that walks
      `<workspace>/crates` for `*.rs` files, allowlists 6 known fallback
      paths + `tests/` + `benches/`, and fails on any other quoted legacy
      method literal.
- [x] A second `#[test] fn allowlist_entries_exist()` asserts that every
      ALLOWLIST path resolves to a real file (catches rename rot).
- [x] `cargo test -p termlink-hub --test no_legacy_callers` passes against
      the current tree (proves the allowlist is correct today).
- [x] A negative-control experiment confirms the test would catch a
      regression: temporarily add a non-allowlisted source-file reference
      to one of the legacy methods, confirm the test fails with a clear
      file:line message, then revert. Evidence: injected
      `let m = "event.broadcast";` at `crates/termlink-mcp/src/lib.rs:6`,
      test output reported it verbatim, then reverted; final `cargo test`
      shows `3 passed; 0 failed`.

### Decisions
- **Line-pattern classifier over allowlist bloat.** Rather than allowlist
  every file that happens to contain a legacy method string (constants,
  routing classifiers, lint configs, inline tests), the test classifies
  the *shape* of each line: comments, `const X: &str = "..."`, match-arm
  patterns, and `#[cfg(test)] / #[test] / #[tokio::test]` blocks are
  skipped. Result: the allowlist stays tight (6 fallback paths) and any
  *new* file containing a real caller is caught. Confirmed today: 7
  pre-existing matches across `target.rs`, `control.rs`, `channel.rs`,
  `server.rs`, `topic_lint.rs` are all correctly classified as non-callers.

## Verification

cargo test -p termlink-hub --test no_legacy_callers --no-fail-fast 2>&1 | tail -20
test -f crates/termlink-hub/tests/no_legacy_callers.rs
grep -q "no_new_direct_legacy_callers" crates/termlink-hub/tests/no_legacy_callers.rs
grep -q "allowlist_entries_exist" crates/termlink-hub/tests/no_legacy_callers.rs

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

### 2026-04-29T20:28:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1406-regression-test-forbid-new-in-repo-direc.md
- **Context:** Initial task creation

### 2026-04-29T20:34:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
