---
id: T-1017
name: "send-file reports success when no receiver is running — silent data loss"
description: >
  termlink send-file reports 'Transfer complete' even when no session is running file receive on the target. The hub accepts the chunks but nobody assembles them, leading to silent data loss. Should either warn that no receiver is active or queue for later delivery.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T12:01:07Z
last_update: 2026-04-15T13:47:08Z
date_finished: 2026-04-13T12:05:49Z
---

# T-1017: send-file reports success when no receiver is running — silent data loss

## Context

send-file has two delivery paths: Direct (session online locally) and Hub (via hub, may spool to inbox). Both report "Transfer complete" regardless of whether a receiver is running `termlink file receive`. The hub path already shows "(via hub — may be spooled for later delivery)" but JSON consumers see `"ok": true`. The direct path hardcodes `{"delivered": true}` without checking anything. Fix: make JSON output include `spooled` field, add warning about receiver requirement, and distinguish acceptance from delivery.

## Acceptance Criteria

### Agent
- [x] JSON output includes `"spooled": true/false` to distinguish inbox spool from live delivery
- [x] Human-readable output warns "receiver must run 'termlink file receive' to assemble" when spooled
- [x] Direct delivery JSON passes through actual session response instead of hardcoded `{"delivered": true}`
- [x] Tests pass (`cargo test -p termlink`)
- [x] No clippy warnings on changed files

### Human
- [x] [REVIEW] Test send-file to offline target and verify warning appears — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — send-file warning fix in code (T-1017). Live test against offline target deferred (would disrupt session).
  **Steps:**
  1. `cd /opt/termlink && termlink file send some-offline-target /tmp/test-file`
  2. Check stderr output
  **Expected:** Warning about needing `file receive` or clear spool message
  **If not:** Check file.rs line ~250 for the output logic

  **Agent evidence (2026-04-15T19:45Z):** Verified fix present in
  `crates/termlink-cli/src/commands/file.rs`:
  - Line 247: JSON output includes `"spooled": spooled` field.
  - Lines 253–255: when `spooled=true`, emits two stderr lines —
    `File spooled to hub inbox for '<target>'. SHA-256: <hash>` followed by
    `Target must run 'termlink file receive <target>' to assemble the file.`
  Code path matches the AC literal expectation. A live end-to-end repro is
  blocked locally by a separate stale-pidfile issue (T-1030); the code audit
  confirms the fix shipped. Human may tick and close.

## Verification

cargo test -p termlink 2>&1 | tail -3 | grep -q "test result: ok"
cargo clippy -p termlink -- -D warnings 2>&1 | grep -v "^warning:" | grep -q "Finished"

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

### 2026-04-13T12:01:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1017-send-file-reports-success-when-no-receiv.md
- **Context:** Initial task creation

### 2026-04-13T12:05:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
