---
id: T-1307
name: "T-1304 follow-up: skip-list event.poll/event.collect from rpc-audit (perf)"
description: >
  T-1304 follow-up: skip-list event.poll/event.collect from rpc-audit (perf)

status: work-completed
workflow_type: build
owner: agent
horizon: next
tags: []
components: [crates/termlink-hub/src/rpc_audit.rs]
related_tasks: []
created: 2026-04-27T11:41:48Z
last_update: 2026-04-27T11:46:35Z
date_finished: 2026-04-27T11:46:35Z
---

# T-1307: T-1304 follow-up: skip-list event.poll/event.collect from rpc-audit (perf)

## Context

T-1306 e2e validation (2026-04-27) found that a single `termlink event collect --timeout 1` CLI invocation generated 13,229 entries in `rpc-audit.jsonl` — the long-poll subscriber loop fires `event.collect`/`event.poll` RPCs at ~13kHz internally. At fleet steady-state this would dominate audit log volume and obscure user-meaningful API calls (the actual signal T-1166 cares about).

**Fix.** Add a skip-list of "transport plumbing" methods that are not user-meaningful API calls and exclude them from `record()`. Initial set: `event.poll`, `event.collect`, `hub.heartbeat` (if any). Keep `event.broadcast`, `event.emit_to`, `channel.post`, `inbox.*`, `file.*` recorded — those are real API surface.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/src/rpc_audit.rs::record(method)` early-returns for methods in a `SKIP_METHODS` set: at minimum `event.poll`, `event.collect`
- [x] Set lives as a `const &[&str]` next to `record()` so the list is auditable + grep-friendly
- [x] Unit test: `record("event.poll")` does NOT append to the file; `record("event.broadcast")` DOES
- [x] E2E re-validate: drive the same `event collect --timeout 1` traffic; audit-log line count stays < 100 instead of 13K
- [x] Update `docs/operations/api-usage-metrics.md` "Performance notes" section to mention the skip-list and call it out as the v1.5 mitigation
- [x] `cargo test -p termlink-hub rpc_audit` 0 failures
- [x] `cargo clippy -p termlink-hub --tests -- -D warnings` clean

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo test -p termlink-hub rpc_audit 2>&1 | tail -10 | grep -qE "test result: ok"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q "SKIP_METHODS" crates/termlink-hub/src/rpc_audit.rs
grep -q "skip-list" docs/operations/api-usage-metrics.md

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

### 2026-04-27T11:41:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1307-t-1304-follow-up-skip-list-eventpolleven.md
- **Context:** Initial task creation

### 2026-04-27T11:46:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
