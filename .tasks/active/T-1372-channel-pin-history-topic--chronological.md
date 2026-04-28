---
id: T-1372
name: "channel pin-history <topic> — chronological pin/unpin audit log"
description: >
  channel pin-history <topic> — chronological pin/unpin audit log

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T12:13:12Z
last_update: 2026-04-28T12:13:12Z
date_finished: null
---

# T-1372: channel pin-history <topic> — chronological pin/unpin audit log

## Context

`channel pinned` (T-1345) shows live pin set after last-write-wins collapse. Useful for "what's pinned now" but loses the *why* — when did this get pinned, by whom, was it ever unpinned then re-pinned, and where in the topic timeline did each toggle happen. `pin-history <topic>` exposes the raw pin/unpin envelopes chronologically: every audit-relevant moment as a row.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_pin_history(envelopes: &[Value]) -> Vec<PinHistoryRow>` in channel.rs returns one row per `msg_type=pin` envelope, sorted by event_offset asc
- [x] `PinHistoryRow { event_offset, action ("pin"|"unpin"), target_offset, actor_sender, ts_ms, target_payload }` — target_payload is best-effort from the topic snapshot, may be None when target isn't in the slice
- [x] CLI: `ChannelAction::PinHistory { topic, hub, json }`
- [x] main.rs dispatch wired
- [x] At least 4 unit tests: pin-then-unpin renders 2 rows asc; multiple toggles all preserved (audit, not LWW); malformed pin envelope (missing pin_target) skipped; default action treated as "pin"
- [x] cargo test passes; clippy clean
- [x] e2e step 44: pin offset 0, unpin, re-pin → 3 rows in asc order with actions in correct sequence; positive payload preview check; JSON shape

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

cargo test -p termlink --bins --quiet 2>&1 | tail -3 | grep -q "test result: ok"
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -2 | grep -qE "(warning|error): 0|^$|^\s*Finished"
bash tests/e2e/agent-conversation.sh 2>&1 | grep -q "END-TO-END WALKTHROUGH PASSED"

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

### 2026-04-28T12:13:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1372-channel-pin-history-topic--chronological.md
- **Context:** Initial task creation
