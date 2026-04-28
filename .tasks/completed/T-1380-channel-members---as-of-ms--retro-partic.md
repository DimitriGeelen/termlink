---
id: T-1380
name: "channel members --as-of <ms> — retro participant view (Matrix-style backfill membership query, parallel to T-1378 snapshot)"
description: >
  channel members --as-of <ms> — retro participant view (Matrix-style backfill membership query, parallel to T-1378 snapshot)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T15:04:54Z
last_update: 2026-04-28T15:18:22Z
date_finished: 2026-04-28T15:18:22Z
---

# T-1380: channel members --as-of <ms> — retro participant view (Matrix-style backfill membership query, parallel to T-1378 snapshot)

## Context

Add `--as-of <ms>` flag to existing `channel members` (T-1341). When provided,
filters envelopes by `ts <= as_of_ms` before computing the per-sender summary.
Matrix-style retro membership query — "who was active in this room as of last
Tuesday?". Parallel to T-1378 snapshot which does the same temporal slicing
for content state.

Without `--as-of`, behaviour is identical to T-1341.

## Acceptance Criteria

### Agent
- [x] `summarize_members_as_of(envelopes, include_meta, as_of_ms)` pure helper that filters then delegates to `summarize_members`
- [x] `--as-of <ms>` flag on `Members` CLI variant (Option<i64>)
- [x] Dispatch threads `as_of` through to a new `cmd_channel_members_as_of` (or extends `cmd_channel_members` to accept Option<i64>)
- [x] When `as_of` is None, behaviour is identical to T-1341 (no regression)
- [x] When `as_of` is Some(t), envelopes with ts > t are excluded; senders whose only activity was after t do not appear
- [x] At least 4 unit tests: as_of None equals existing, as_of before any → empty, as_of mid-history shows partial, sender-only-after excluded
- [x] Doc note added on `members` section
- [x] E2E step exercising as_of-mid-history vs no-flag
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

### 2026-04-28T15:04:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1380-channel-members---as-of-ms--retro-partic.md
- **Context:** Initial task creation

### 2026-04-28T15:18:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
