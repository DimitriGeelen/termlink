---
id: T-1314
name: "channel reactions (Matrix m.annotation inspired)"
description: >
  channel reactions (Matrix m.annotation inspired)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:28:27Z
last_update: 2026-04-27T13:28:27Z
date_finished: null
---

# T-1314: channel reactions (Matrix m.annotation inspired)

## Context

Matrix-style reactions as an additive convenience on top of T-1313 threading.
A reaction is a typed post (`msg_type=reaction`) that points at a parent
envelope's offset via `metadata.in_reply_to` — same wire shape as a
threaded reply, distinguished only by the well-known msg_type. This task
adds (a) a CLI shorthand `channel react`, (b) compact rendering for
reaction envelopes in non-JSON subscribe output, and (c) a `--reactions`
aggregator that groups reactions under their parent for read flow.

No hub-side changes — `msg_type` is already opaque to the hub. All work
is in CLI and ergonomics.

## Acceptance Criteria

### Agent
- [x] CLI `termlink channel react <topic> <parent_offset> <reaction>` posts a `msg_type=reaction` envelope with `metadata.in_reply_to=<parent_offset>` and the reaction string as payload
- [x] CLI `termlink channel subscribe` non-JSON output renders reaction envelopes compactly: `[<offset> ↳<parent> react] <sender> <reaction-payload>` (no `msg_type:` prefix; aligns with chat-style brevity)
- [x] CLI `termlink channel subscribe --reactions` (flag) aggregates reactions under their parent in non-JSON output: parent line gets a trailing `└─ reactions: 👍 ×3, 👀` summary; reactions themselves are NOT printed as standalone lines
- [x] CLI `termlink channel react --json` outputs delivery envelope shape consistent with `channel post --json`
- [x] CLI build passes; clippy clean
- [x] Smoke evidence captured in task file (post, react, subscribe, subscribe --reactions)

### Live smoke evidence (2026-04-27)

```
$ termlink channel post test:t-1314-v2 --msg-type chat --payload "ship it?"
$ termlink channel react test:t-1314-v2 0 "👍"
$ termlink channel react test:t-1314-v2 0 "👍" --sender-id agent-b
$ termlink channel react test:t-1314-v2 0 "👀" --sender-id agent-c
$ termlink channel post test:t-1314-v2 --msg-type chat --payload "merging now" --reply-to 0
$ termlink channel react test:t-1314-v2 4 "🚀"

$ termlink channel subscribe test:t-1314-v2
[0] d1993c2c3ec44c94 chat: ship it?
[1 ↳0 react] d1993c2c3ec44c94 👍
[2 ↳0 react] agent-b 👍
[3 ↳0 react] agent-c 👀
[4 ↳0] d1993c2c3ec44c94 chat: merging now
[5 ↳4 react] d1993c2c3ec44c94 🚀

$ termlink channel subscribe test:t-1314-v2 --reactions
[0] d1993c2c3ec44c94 chat: ship it?
    └─ reactions: 👍 ×2, 👀
[4 ↳0] d1993c2c3ec44c94 chat: merging now
    └─ reactions: 🚀
```

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
target/debug/termlink channel --help 2>&1 | grep -q react

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

### 2026-04-27T13:28:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1314-channel-reactions-matrix-mannotation-ins.md
- **Context:** Initial task creation
