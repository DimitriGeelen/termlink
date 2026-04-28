---
id: T-1373
name: "channel redactions <topic> — chronological list of redaction events"
description: >
  channel redactions <topic> — chronological list of redaction events

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T12:28:08Z
last_update: 2026-04-28T12:28:08Z
date_finished: null
---

# T-1373: channel redactions <topic> — chronological list of redaction events

## Context

T-1322 added `channel redact` (post a redaction) and `redacted_offsets` (a set of "what's redacted now"). Symmetric to `pin-history` (T-1372): an audit listing of every redaction event chronologically. Each row carries event_offset, target_offset, redactor sender, optional reason, ts, and the original-target's payload preview when still in the snapshot.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_redactions(envelopes: &[Value]) -> Vec<RedactionRow>` in channel.rs — one row per `msg_type=redaction` envelope with parseable `metadata.redacts`, sorted by event_offset asc
- [x] `RedactionRow { event_offset, target_offset, redactor_sender, reason, ts_ms, target_payload }` with to_json
- [x] CLI: `ChannelAction::Redactions { topic, hub, json }`
- [x] main.rs dispatch wired
- [x] At least 4 unit tests: 2 redactions chronological asc; reason-with optional rendering; missing target → target_payload None; malformed redacts (non-numeric) skipped
- [x] cargo test passes; clippy clean
- [x] e2e step 45 — redact 2 distinct offsets with one reason, verify table+JSON shape, count, ordering

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

### 2026-04-28T12:28:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1373-channel-redactions-topic--chronological-.md
- **Context:** Initial task creation
