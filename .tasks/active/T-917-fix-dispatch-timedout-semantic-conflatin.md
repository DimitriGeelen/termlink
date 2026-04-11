---
id: T-917
name: "Fix dispatch timed_out semantic conflating crash with timeout"
description: >
  Discovered 2026-04-11 during T-914 end-to-end smoke test. dispatch.rs:672 defines: 'let timed_out = collected_count < registered_count;'. This conflates two distinct failure modes: (a) actual timeout (collect_start.elapsed() exceeded --timeout), vs (b) workers crashed before emitting events (collected_count < registered_count but elapsed was 0.5s). Real-world result observed: smoke test ran in 1.01s wall clock with crashed_workers populated correctly, BUT JSON output also reported 'timed_out: true' which is misleading. The 'ok' field and exit code are also derived from 'timed_out' so this also affects success/failure signaling. FIX: define timed_out as 'collect_start.elapsed() >= collect_timeout' (the literal definition), and introduce 'any_failure = timed_out || !crashed_workers.is_empty() || collected_count < registered_count' for ok/exit-code logic. Trivial single-function change.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [termlink, dispatch, bug, semantics, observability]
components: []
related_tasks: [T-914, T-916]
created: 2026-04-11T13:54:36Z
last_update: 2026-04-11T13:54:43Z
date_finished: null
---

# T-917: Fix dispatch timed_out semantic conflating crash with timeout

## Context

`dispatch.rs:672` defines `let timed_out = collected_count < registered_count;` which is true whenever ANY worker doesn't emit a result, including legitimate crashes detected via early-crash. The label is then used for: the JSON `timed_out` field (misleading), the `ok` field (semantically OK as a fail signal but the cause is hidden), the human-readable "Timed out. Missing: ..." message (misleading), and the exit code. Splitting label from semantics fixes the user-visible issue without changing exit-code behavior.

## Acceptance Criteria

### Agent
- [x] `timed_out` is computed from elapsed time only (`collect_start.elapsed() >= collect_timeout`)
- [x] A separate `any_failure` value combines `timed_out || !crashed_workers.is_empty() || collected_count < registered_count` for `ok`/exit-code logic
- [x] JSON output: `timed_out` reflects actual timeout; `ok` reflects any failure
- [x] Human output: "Timed out. Missing: ..." message only prints when actually timed out (not when crashed) — uses unchanged `timed_out` variable which now has correct semantics
- [x] Exit code remains non-zero on any failure (verified — exit 1 for crashed_workers case)
- [x] All 11 existing dispatch tests still pass
- [x] Real-world verification: dispatch with `bash -c 'exit 42'` against the isolated hub at /tmp/termlink-t914-isolated now outputs `timed_out: false`, `ok: false`, `crashed_workers: ["t917-1"]`, exit 1, wall clock 1.01s

### Human
None — fully agent-verifiable.

## Verification

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

### 2026-04-11T13:54:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-917-fix-dispatch-timedout-semantic-conflatin.md
- **Context:** Initial task creation

### 2026-04-11T13:54:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
