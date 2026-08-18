---
id: T-1377
name: "channel ack-history <topic> [user] — chronological receipt audit log (Matrix
  m.receipt audit, complements ack-status LWW dashboard)"
description: >
  channel ack-history <topic> [user] — chronological receipt audit log (Matrix m.receipt
  audit, complements ack-status LWW dashboard)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-28T14:21:55Z
last_update: '2026-08-18T18:58:49Z'
date_finished: 2026-04-28T14:35:59Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:49Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-1377: channel ack-history <topic> [user] — chronological receipt audit log (Matrix m.receipt audit, complements ack-status LWW dashboard)

## Context

`channel ack-history <topic> [user]` — chronological receipt audit. Lists every
`msg_type=receipt` envelope as a row in ts_ms asc order. Distinct from
`channel receipts` (T-1315 LWW snapshot) and `channel ack-status` (T-1361
dashboard with lag). Extends audit-log family from content mutations
(pin-history T-1372, redactions T-1373, edit-stats T-1375) to receipt activity.

## Acceptance Criteria

### Agent
- [x] `compute_ack_history(envelopes, user_filter)` pure helper exists
- [x] `AckHistoryRow {receipt_offset, sender_id, up_to, ts_ms}` struct + to_json
- [x] Filter: only `msg_type=receipt` envelopes with parseable `metadata.up_to`
- [x] When `user_filter` is `Some(uid)`, only rows where `sender_id == uid` survive
- [x] Sort: ts_ms asc, receipt_offset asc tiebreak
- [x] `cmd_channel_ack_history` async wrapper renders human + JSON
- [x] At least 5 unit tests: empty, single, multi-sender, user-filter, ts-asc-sort, malformed-up_to-skipped
- [x] `ChannelAction::AckHistory` variant in cli.rs with positional `user` Option, `--hub`, `--json`
- [x] Dispatch arm in main.rs
- [x] E2E step in agent-conversation.sh exercising positive + user filter + JSON shape
- [x] Doc section in agent-conversations.md
- [x] tests/clippy/build pass

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
cargo test -p termlink --bins --quiet 2>&1 | tail -5
cargo clippy --all-targets --workspace -- -D warnings 2>&1 | tail -5

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

### 2026-04-28T14:21:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1377-channel-ack-history-topic-user--chronolo.md
- **Context:** Initial task creation

### 2026-04-28T14:35:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
