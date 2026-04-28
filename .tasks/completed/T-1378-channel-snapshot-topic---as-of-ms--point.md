---
id: T-1378
name: "channel snapshot <topic> --as-of <ms> — point-in-time canonical view (Matrix backfill semantics; combines T-1376 collapse with ts upper bound)"
description: >
  channel snapshot <topic> --as-of <ms> — point-in-time canonical view (Matrix backfill semantics; combines T-1376 collapse with ts upper bound)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T14:37:01Z
last_update: 2026-04-28T14:49:29Z
date_finished: 2026-04-28T14:49:29Z
---

# T-1378: channel snapshot <topic> --as-of <ms> — point-in-time canonical view (Matrix backfill semantics; combines T-1376 collapse with ts upper bound)

## Context

`channel snapshot <topic> --as-of <ms>` — point-in-time canonical view of a topic.
Matrix backfill semantics: simulate the room as it was at timestamp `ms`. Combines
T-1376 collapse logic (apply edits, hide redactions) with a temporal upper bound:
edits/redactions whose ts is GREATER than `as_of` are NOT applied — they hadn't
happened yet at that point in time.

Useful for:
- "what did the topic say last Tuesday at 3pm?"
- forensic replay before a particular event
- verifying when a piece of content was first changed

Distinct from T-1376 `state` (current truth), T-1352 `subscribe --until` (raw
envelope filter), and T-1366 `edits-of` (edit history per target).

## Acceptance Criteria

### Agent
- [x] `compute_snapshot(envelopes, as_of_ms, include_redacted)` pure helper
- [x] Reuses `StateRow` (same shape as T-1376; semantics: as-of view)
- [x] Filter rule: skip envelopes whose ts > as_of_ms (they haven't happened yet)
- [x] After temporal filter: apply T-1376 collapse logic (edits, redactions, meta-skip)
- [x] If a target message has ts > as_of_ms, it doesn't surface (didn't exist yet)
- [x] `cmd_channel_snapshot` async wrapper, human + JSON output
- [x] At least 5 unit tests: empty, before-any-msg, between-msgs, after-edit-only-original-shows, redaction-not-applied-yet
- [x] `ChannelAction::Snapshot` variant in cli.rs (topic, --as-of, --include-redacted, --hub, --json)
- [x] Dispatch arm in main.rs
- [x] E2E step exercising before/after timestamps with edits + redactions
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

### 2026-04-28T14:37:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1378-channel-snapshot-topic---as-of-ms--point.md
- **Context:** Initial task creation

### 2026-04-28T14:49:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
