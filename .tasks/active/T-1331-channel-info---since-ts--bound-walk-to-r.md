---
id: T-1331
name: "channel info --since <ts> — bound walk to recent slice for long-lived topics"
description: >
  channel info --since <ts> — bound walk to recent slice for long-lived topics

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T16:51:47Z
last_update: 2026-04-27T16:51:47Z
date_finished: null
---

# T-1331: channel info --since <ts> — bound walk to recent slice for long-lived topics

## Context

`channel info` walks the entire topic for description/senders/receipts. For
long-lived topics that's linear. Add `--since <ms>` to bound the summary
to records with `ts_unix_ms >= since`. Total `Posts:` shows full count plus
bounded slice; description / senders / receipts reflect the slice.

## Acceptance Criteria

### Agent
- [x] `cli.rs` Info variant gains `--since <ms>: Option<i64>`.
- [x] `cmd_channel_info` accepts `since: Option<i64>`. When set, filters `all_msgs` by `ts >= since` before computing description/senders/receipts.
- [x] Pure helper `filter_msgs_since` with unit tests: bound is inclusive; empty input returns empty; all-before returns empty; all-after returns all.
- [x] Text output adds `(N since <ts>)` parenthetical to `Posts:` when --since is set.
- [x] JSON output adds `"since"` and `"posts_since"` fields when --since is set.
- [x] `cargo test -p termlink --bins filter_msgs_since` passes.
- [x] `cargo clippy --all-targets --workspace -- -D warnings` clean.
- [x] Smoke: `--since 0` matches no-flag output; `--since <future-ts>` shows zero senders/receipts.

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

cargo test -p termlink --bins filter_msgs_since
cargo clippy --all-targets --workspace -- -D warnings

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

### 2026-04-27T16:51:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1331-channel-info---since-ts--bound-walk-to-r.md
- **Context:** Initial task creation
