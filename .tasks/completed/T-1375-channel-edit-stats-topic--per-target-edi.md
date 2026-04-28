---
id: T-1375
name: "channel edit-stats <topic> — per-target edit count summary"
description: >
  channel edit-stats <topic> — per-target edit count summary

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-28T13:28:55Z
last_update: 2026-04-28T13:43:19Z
date_finished: 2026-04-28T13:43:19Z
---

# T-1375: channel edit-stats <topic> — per-target edit count summary

## Context

`channel edits-of <topic> <offset>` (T-1366) shows full edit history for ONE target. `channel edit-stats <topic>` is the topic-wide aggregate: which messages have been edited, how often, and by whom most recently. Completes the audit trio (T-1372 pin-history, T-1373 redactions, T-1375 edit-stats) — three structurally similar pure helpers, each rolling up one mutation type.

## Acceptance Criteria

### Agent
- [x] `pub fn compute_edit_stats(envelopes: &[Value]) -> Vec<EditStatsRow>` — one row per target offset that has at least one non-redacted edit, with count, last editor, last ts, target sender + payload preview
- [x] `EditStatsRow { target_offset, target_sender, target_payload, edit_count, latest_editor, latest_ts_ms }` with to_json
- [x] Sort: edit_count desc, target_offset asc tiebreak
- [x] Skip edits whose own offset is redacted (don't count) and edits whose target is redacted (drop the whole row)
- [x] CLI: `ChannelAction::EditStats { topic, hub, json }`
- [x] main.rs dispatch
- [x] At least 4 unit tests: 1 target with 3 edits → count=3, latest editor + ts correct; 2 targets sorted desc; redacted edit not counted; redacted target → row dropped entirely
- [x] cargo test passes; clippy clean
- [x] e2e step 47 — 2 targets edited (target0 ×2 by alice+bob, target1 ×1 by alice), table + JSON shape, count check, ordering check

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

### 2026-04-28T13:28:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1375-channel-edit-stats-topic--per-target-edi.md
- **Context:** Initial task creation

### 2026-04-28T13:43:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
