---
id: T-1317
name: "channel reactions: --by-sender per-reactor identity (T-1314 follow-up)"
description: >
  channel reactions: --by-sender per-reactor identity (T-1314 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-27T13:40:20Z
last_update: 2026-04-27T13:43:06Z
date_finished: 2026-04-27T13:43:06Z
---

# T-1317: channel reactions: --by-sender per-reactor identity (T-1314 follow-up)

## Context

T-1314 reactions aggregation shows counts only (`👍 ×3, 👀`). For agent
conversations the *who* is often the signal — "did the reviewer ack?",
"did CI pass?" — and a count loses that. Add a `--by-sender` flag to
`channel subscribe --reactions` that shows reactor identities grouped per
emoji: `👍 by alice, bob, carol`. Tiny CLI-only change.

## Acceptance Criteria

### Agent
- [x] CLI `termlink channel subscribe --reactions --by-sender` shows per-emoji reactor list (`👍 by alice, bob` instead of `👍 ×2`); same first-seen ordering of emojis
- [x] Without `--by-sender`, existing count-only behavior unchanged (backwards compatible)
- [x] `--by-sender` requires `--reactions` (clap-level enforcement via `requires` attribute)
- [x] CLI build + clippy clean
- [x] Smoke evidence in task file
- [x] Same-sender double-react dedups in `--by-sender` mode (alice++ shows once)

### Live smoke evidence (2026-04-27)

```
$ termlink channel react test:t-1317 0 "👍" --sender-id alice
$ termlink channel react test:t-1317 0 "👍" --sender-id bob
$ termlink channel react test:t-1317 0 "👀" --sender-id carol
$ termlink channel react test:t-1317 0 "✅" --sender-id ci
$ termlink channel react test:t-1317 0 "👍" --sender-id alice    # double — by-sender de-dups

$ termlink channel subscribe test:t-1317 --reactions
[0] author chat: ship it?
    └─ reactions: 👍 ×3, 👀, ✅

$ termlink channel subscribe test:t-1317 --reactions --by-sender
[0] author chat: ship it?
    └─ reactions: 👍 by alice, bob, 👀 by carol, ✅ by ci
```

Counts and identities are deliberately different: counts = raw event
count (alice's double 👍 → ×3), by-sender = unique-reactor list (alice
once). Both internally consistent.

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
target/debug/termlink channel subscribe --help 2>&1 | grep -q by-sender

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

### 2026-04-27T13:40:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1317-channel-reactions---by-sender-per-reacto.md
- **Context:** Initial task creation

### 2026-04-27T13:43:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
