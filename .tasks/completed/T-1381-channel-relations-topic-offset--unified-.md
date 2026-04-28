---
id: T-1381
name: "channel relations <topic> <offset> — unified per-offset navigation: replies + reactions + edits + redactions + forwards in one view"
description: >
  channel relations <topic> <offset> — unified per-offset navigation: replies + reactions + edits + redactions + forwards in one view

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T15:19:46Z
last_update: 2026-04-28T15:35:05Z
date_finished: 2026-04-28T15:35:05Z
---

# T-1381: channel relations <topic> <offset> — unified per-offset navigation: replies + reactions + edits + redactions + forwards in one view

## Context

`channel relations <topic> <offset>` — unified per-offset navigation. For one
target message, return ALL relationships pointing at it in one view: replies,
reactions, redactions, edits, forwards. Consolidates ~5 separate commands
(replies-of, reactions-on, edits-of, redactions, forwards-of) into a single
navigation point keyed on the target.

Output groups (counts + first 5 of each, JSON returns full):
- replies (msg_type != reaction AND in_reply_to == offset)
- reactions (msg_type == reaction AND in_reply_to == offset)
- edits (msg_type == edit AND replaces == offset)
- redactions (msg_type == redaction AND redacts == offset)

This matches Matrix Client API's "/relations/{eventId}" semantics. Forwards
are intentionally excluded — they are cross-topic relations and would
require walking multiple topics; same-topic forward query is degenerate.

## Acceptance Criteria

### Agent
- [x] `compute_relations(envelopes, target)` pure helper returning `RelationsReport`
- [x] `RelationsReport {target_offset, target_payload, target_sender, replies: Vec<RelationItem>, reactions: Vec<RelationItem>, edits: Vec<RelationItem>, redactions: Vec<RelationItem>}`
- [x] `RelationItem {offset, sender_id, ts_ms, payload}` (uniform shape across all relation types; payload=emoji for reactions, empty for redactions, etc)
- [x] When target offset not in envelopes → return None (or all-empty with target_offset=offset, payload=""); choose all-empty with bail-on-missing semantics matching `edits-of`
- [x] Filter: skip relation envelopes that are themselves redacted (consistency with audit-log conventions)
- [x] Each relation list sorted: ts_ms asc (chronological)
- [x] `cmd_channel_relations` async wrapper — human render shows counts + first 5 of each, JSON returns full
- [x] At least 5 unit tests: target-not-present, replies-only, reactions-only, all-four-types, redacted-relation-excluded
- [x] `ChannelAction::Relations` variant in cli.rs (topic, offset, --hub, --json)
- [x] Dispatch arm in main.rs
- [x] E2E step exercising 4+ relation types on one target + JSON shape
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

### 2026-04-28T15:19:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1381-channel-relations-topic-offset--unified-.md
- **Context:** Initial task creation

### 2026-04-28T15:35:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
