---
id: T-1219
name: "Fix T-1217 subscriber — use event poll not event collect"
description: >
  T-1217 subscriber uses event collect which only delivers NEW events during the collect window, not events already accumulated in session buses. When broadcast comes from a different session than collector, collect returns 0 even though events ARE in the target sessions' buses (confirmed via event poll). Fix: change subscriber to event poll <session-name> --since=<cursor> against a session on the hub. Needs per-session cursor. Alternative: spawn a dedicated short-lived subscriber session that stays registered long enough to catch live events. Blocks T-1217 Human RUBBER-STAMP.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-24T13:39:45Z
last_update: 2026-04-24T13:50:52Z
date_finished: 2026-04-24T13:50:52Z
---

# T-1219: Fix T-1217 subscriber — use event poll not event collect

## Context

During T-1217 end-to-end validation (subscriber on email-archive peer),
discovered the subscriber's use of `termlink event collect` is wrong. Empirical
findings:

- `event broadcast` from /opt/termlink reaches all registered sessions
  (confirmed: 6/6 succeeded)
- Events accumulate in each session's event bus (confirmed:
  `event poll email-archive --topic channel:learnings` returns 23 events)
- `event collect --topic X` invoked from session A does **not** see events
  broadcast from a different session B — collect appears to only deliver
  events broadcast within its own session context, or only live events that
  arrive during the collect window but NOT into the collector's own session's bus

This is a fundamental design bug in T-1217's subscriber. The T-1217 discovery
spike mis-diagnosed it because broadcast+collect were run from the same shell
session, which masks the cross-session behavior.

The fix: poll a session's event bus directly with
`termlink event poll <session> --topic channel:learnings --since=<cursor>`.
Broadcasts fan out to every registered session's bus, so polling any one gets
everything. Track per-session cursor.

## Acceptance Criteria

### Agent
- [x] `lib/subscribe-learnings-from-bus.sh` rewritten to use `event poll`:
      picks first ready session from `termlink list --json`, calls
      `termlink event poll <session> --topic channel:learnings --since=<cursor>
      --json --timeout N`, parses `{count, events:[{seq, payload, ...}]}`,
      advances cursor to `max(seq)`. Landed in cf5b2dbf on onedev.
- [x] Cursor YAML at `.context/working/.subscribe-learnings-bus.cursor` with
      `target_session` + `since`. When target changes (session died), cursor
      resets to 0; composite-key dedup prevents re-ingest.
- [x] Composite-key dedup `(origin_project, learning_id)` retained. Smoke
      test showed 5 in-batch dupes caught during backlog drain.
- [x] Self-filter retained and verified.
- [x] **End-to-end validated live:** `fw context add-learning PL-058` on
      /opt/termlink → `posted via=event.broadcast` → subscriber on
      email-archive polled framework-agent bus with `--since=379` →
      `received=1 appended=1`, PL-058 entry with `origin_project: termlink`
      written to received-learnings.yaml, cursor advanced 379 → 380.
- [x] Upstream-mirrored commit cf5b2dbf pushed to onedev/master (PL-053
      pattern: dispatch timed out as expected, verified by direct refs).

**Additional finding (recorded here — not AC-blocking):** `termlink event
poll --since=N` is **exclusive** (returns events with `seq > N`), not
inclusive. Cursor stores `max_seq` as-is; next poll with `--since=max_seq`
correctly skips what we already have. Off-by-one was caught in first
post-fix test and corrected before commit.

### Human
<!-- Remove this section — validation is agent-verifiable via live e2e test. -->

## Verification

bash -n /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
grep -q "event poll" /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
grep -q "since" /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh
grep -q "FW_LEARNINGS_BUS_SUBSCRIBE" /opt/999-Agentic-Engineering-Framework/lib/subscribe-learnings-from-bus.sh

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

### 2026-04-24T13:39:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1219-fix-t-1217-subscriber--use-event-poll-no.md
- **Context:** Initial task creation

### 2026-04-24T13:45:20Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-24T13:50:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
