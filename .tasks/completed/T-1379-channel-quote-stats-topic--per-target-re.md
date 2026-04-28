---
id: T-1379
name: "channel quote-stats <topic> — per-target reply rollup (per-target companion to T-1370 replies-of which is per-sender)"
description: >
  channel quote-stats <topic> — per-target reply rollup (per-target companion to T-1370 replies-of which is per-sender)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T14:50:54Z
last_update: 2026-04-28T15:03:30Z
date_finished: 2026-04-28T15:03:30Z
---

# T-1379: channel quote-stats <topic> — per-target reply rollup (per-target companion to T-1370 replies-of which is per-sender)

## Context

`channel quote-stats <topic>` — per-target reply rollup. For each target message
that has been replied to at least once, emits {target_offset, target_sender,
target_payload, reply_count, distinct_repliers, latest_reply_ts_ms}.

Per-target companion to T-1370 `replies-of` (per-sender). Answers the question
"what's getting the most discussion in this topic?" — useful for triage.

Filters:
- skip reactions (`msg_type=reaction`) — they carry `in_reply_to` but are not real replies
- skip redacted reply offsets — they don't count
- drop rows whose target is itself redacted

Sort: reply_count desc, target_offset asc tiebreak.

## Acceptance Criteria

### Agent
- [x] `compute_quote_stats(envelopes)` pure helper
- [x] `QuoteStatsRow {target_offset, target_sender, target_payload, reply_count, distinct_repliers, latest_reply_ts_ms}` (distinct_repliers: sorted Vec<String>)
- [x] Exclude reactions: `msg_type=reaction` envelopes are not replies even if they carry `in_reply_to`
- [x] Exclude redacted reply offsets and redacted target rows
- [x] Sort: reply_count desc, target_offset asc tiebreak
- [x] `cmd_channel_quote_stats` async wrapper, human + JSON
- [x] At least 5 unit tests: empty, single reply, multi-reply same target, two targets sorted, reactions excluded, redacted reply excluded, redacted target dropped
- [x] `ChannelAction::QuoteStats` variant in cli.rs (topic, --hub, --json)
- [x] Dispatch arm in main.rs
- [x] E2E step exercising 2 targets with 3 + 1 replies + JSON
- [x] Doc section
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

### 2026-04-28T14:50:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1379-channel-quote-stats-topic--per-target-re.md
- **Context:** Initial task creation

### 2026-04-28T15:03:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
