---
id: T-1318
name: "channel subscribe persistent local cursor (Matrix /sync next_batch analogue)"
description: >
  channel subscribe persistent local cursor (Matrix /sync next_batch analogue)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:43:51Z
last_update: 2026-04-27T13:43:51Z
date_finished: null
---

# T-1318: channel subscribe persistent local cursor (Matrix /sync next_batch analogue)

## Context

Matrix `/sync` returns a `next_batch` token clients store and replay as
the `since` parameter on the next call — so each client gets exactly the
events they haven't seen, no more, no less. TermLink's `channel subscribe`
doesn't have this today: each invocation starts at offset 0 unless the
caller explicitly provides `--cursor N`. For agents running in
short-lived shells (each invocation a fresh process), this means either
re-streaming everything or remembering the offset out-of-band.

Add a per-(topic, identity-fingerprint) local cursor at
`~/.termlink/cursors.json`. After a successful `subscribe`, write
`{topic, fingerprint} → next_cursor`. New `--resume` flag reads it;
`--reset` clears it before starting. Default behavior (no flag)
unchanged so existing callers don't regress.

The cursor is **per-identity** so two agents sharing the same machine
(different identity files) get independent cursors, mirroring Matrix's
per-user `next_batch` tokens.

Distinct from T-1315 receipts: receipts are PUBLIC ("I want others to
know I saw this"), cursors are PRIVATE ("I don't need to re-process
this"). Different semantics, both valid.

## Acceptance Criteria

### Agent
- [x] CLI `termlink channel subscribe <topic> --resume` reads `~/.termlink/cursors.json` for `(topic, identity_fingerprint)` and starts from that cursor; falls back to `--cursor` value (default 0) if no entry exists
- [x] After a successful subscribe (one-shot or follow loop), writes the latest `next_cursor` back to `~/.termlink/cursors.json` for the same key
- [x] CLI `termlink channel subscribe <topic> --reset` deletes the persisted cursor entry before starting; starts from offset 0 unless `--cursor` overrides
- [x] `--resume` and `--reset` mutually exclusive at clap level (`Usage: ... --resume` error on combined use)
- [x] Cursor file is JSON with shape `{"<topic>::<fingerprint>": <offset>, ...}` — single flat map; atomic write via `.tmp` rename
- [x] Existing default behavior unchanged — no flags = no cursor reads or writes (backward compatible)
- [x] CLI build + clippy clean
- [x] Smoke evidence in task file (post 3 → resume → post 1 more → resume shows only the new one → reset → resume from 0)
- [x] Agent-conversations doc gains "Persistent local cursor (T-1318)" section AND the deferred T-1317 `--by-sender` note

### Live smoke evidence (2026-04-27)

```
$ termlink channel post test:t-1318 --payload "msg-A"   # 0
$ termlink channel post test:t-1318 --payload "msg-B"   # 1
$ termlink channel post test:t-1318 --payload "msg-C"   # 2

$ termlink channel subscribe test:t-1318 --resume
[0] sender chat: msg-A
[1] sender chat: msg-B
[2] sender chat: msg-C

$ cat ~/.termlink/cursors.json
{"test:t-1318::d1993c2c3ec44c94": 3}

$ termlink channel post test:t-1318 --payload "msg-D (new)"   # 3
$ termlink channel subscribe test:t-1318 --resume
[3] sender chat: msg-D (new)

$ termlink channel subscribe test:t-1318 --resume   # nothing new
(empty)

$ termlink channel subscribe test:t-1318 --reset
[0] sender chat: msg-A
... (all 4 lines re-shown)

$ cat ~/.termlink/cursors.json
{}

$ termlink channel subscribe test:t-1318 --resume --reset
error: clap: --resume conflicts with --reset
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
target/debug/termlink channel subscribe --help 2>&1 | grep -q -- --resume
target/debug/termlink channel subscribe --help 2>&1 | grep -q -- --reset
grep -q "Persistent local cursor" docs/operations/agent-conversations.md
grep -q "by-sender" docs/operations/agent-conversations.md

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

### 2026-04-27T13:43:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1318-channel-subscribe-persistent-local-curso.md
- **Context:** Initial task creation
