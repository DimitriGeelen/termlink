---
id: T-1315
name: "channel read receipts (Matrix m.receipt inspired)"
description: >
  channel read receipts (Matrix m.receipt inspired)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:33:38Z
last_update: 2026-04-27T13:33:38Z
date_finished: null
---

# T-1315: channel read receipts (Matrix m.receipt inspired)

## Context

Matrix-style read receipts on top of T-1313/T-1314. A receipt is a typed
post (`msg_type=receipt`) carrying `metadata.up_to=<offset>` — the sender
asserts "I've seen everything up to and including this offset". Unlike
reactions (which target ONE parent), receipts are channel-level cursors,
one rolling pointer per sender. Latest receipt per sender wins.

Adds (a) `channel ack <topic> --up-to N` shorthand to post a receipt,
(b) `channel receipts <topic>` read-side helper that summarizes the
latest receipt per sender. No hub-side changes (msg_type is opaque).

## Acceptance Criteria

### Agent
- [x] CLI `termlink channel ack <topic> --up-to <offset>` posts a `msg_type=receipt` envelope with `metadata.up_to=<offset>` and payload `up_to=<offset>` (text body for human readability)
- [x] CLI `termlink channel ack <topic>` (no --up-to) auto-resolves to the topic's latest offset by calling `channel.list` first; convenience for "I've caught up"
- [x] CLI `termlink channel receipts <topic>` lists the latest receipt per sender: `<sender_id>  up to <offset>  (<ts>)`, sorted by sender_id
- [x] CLI `termlink channel receipts --json` emits `{topic, receipts: [{sender_id, up_to, ts_unix_ms}]}`
- [x] CLI build passes; clippy clean
- [x] Smoke evidence in task file (post, ack, receipts)

### Live smoke evidence (2026-04-27)

```
$ termlink channel post test:t-1315 --msg-type chat --payload "design review starting"   # 0
$ termlink channel post test:t-1315 --msg-type chat --payload "any objections?"           # 1
$ termlink channel post test:t-1315 --msg-type chat --payload "shipping in 5"             # 2

$ termlink channel ack test:t-1315                              # auto → up_to=2  (offset=3)
$ termlink channel ack test:t-1315 --up-to 1 --sender-id agent-b   # explicit (offset=4)
$ termlink channel ack test:t-1315 --sender-id agent-c              # auto → up_to=4 (offset=5)
$ termlink channel post test:t-1315 --msg-type chat --payload "P.S. minor change"   # 6

$ termlink channel receipts test:t-1315
Receipts on 'test:t-1315':
  agent-b  up to 1  (ts=1777296931023)
  agent-c  up to 4  (ts=1777296931050)
  d1993c2c3ec44c94  up to 2  (ts=1777296930999)

$ termlink channel ack test:t-1315-empty
Error: Topic 'test:t-1315-empty' is empty — nothing to ack
```

Note: agent-c is now "stale" (acked up to 4 but latest is 6) — the operator
can see this at a glance via `channel receipts`.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
cargo build -p termlink 2>&1 | tail -5
cargo clippy -p termlink -- -D warnings 2>&1 | tail -5
target/debug/termlink channel --help 2>&1 | grep -q ack
target/debug/termlink channel --help 2>&1 | grep -q receipts

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

### 2026-04-27T13:33:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1315-channel-read-receipts-matrix-mreceipt-in.md
- **Context:** Initial task creation
