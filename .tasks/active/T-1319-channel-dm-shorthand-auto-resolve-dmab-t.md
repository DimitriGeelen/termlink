---
id: T-1319
name: "channel dm shorthand (auto-resolve dm:<a>:<b> topic + auto-create)"
description: >
  channel dm shorthand (auto-resolve dm:<a>:<b> topic + auto-create)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:48:09Z
last_update: 2026-04-27T13:48:09Z
date_finished: null
---

# T-1319: channel dm shorthand (auto-resolve dm:<a>:<b> topic + auto-create)

## Context

Closes the agent-conversation ergonomics loop. Today an agent sends a DM
to peer X by:
  1. Constructing canonical topic name (sort `[mine, peer]` alphabetically
     then format as `dm:<a>:<b>`)
  2. Calling `channel create` once (or remembering it's already created)
  3. Calling `channel post` with the topic name + sender_id

`channel dm <peer>` collapses all three into one verb. Convention-driven:
the topic is always `dm:<sorted-a>:<sorted-b>` so both ends agree without
coordination. Auto-creates the topic if missing (idempotent).

## Acceptance Criteria

### Agent
- [x] CLI `termlink channel dm <peer>` (no flags) subscribes to the canonical DM topic with `--resume` + `--reactions` defaults (the reading mode an agent wants 90% of the time)
- [x] CLI `termlink channel dm <peer> --send "<msg>"` posts the message to the DM topic; auto-creates topic if missing
- [x] CLI `termlink channel dm <peer> --send "<msg>" --reply-to N` posts a threaded reply
- [x] Topic name is deterministic — both sides resolve to identical topic when peer is the OTHER side's fingerprint (alphabetical sort, joined as `dm:<a>:<b>`)
- [x] CLI `termlink channel dm <peer> --topic-only` outputs the canonical topic name without doing anything else (helper for scripts)
- [x] Auto-create uses `forever` retention (DMs are conversational; you usually want full history)
- [x] CLI build + clippy clean
- [x] Smoke evidence in task file (--topic-only, --send, default subscribe)
- [x] Doc updated — Quick start section leads with `channel dm` instead of manual topic construction

### Live smoke evidence (2026-04-27)

Two test identities at `/tmp/term-alice/` and `/tmp/term-bob/` —
fingerprints `04c54d00a7485964` and `c2e7f1cff6c213a0`.

```
$ TERMLINK_IDENTITY_DIR=/tmp/term-alice termlink channel dm c2e7f1cff6c213a0 --topic-only
dm:04c54d00a7485964:c2e7f1cff6c213a0

$ TERMLINK_IDENTITY_DIR=/tmp/term-bob   termlink channel dm 04c54d00a7485964 --topic-only
dm:04c54d00a7485964:c2e7f1cff6c213a0    # ← identical (sort works)

$ TERMLINK_IDENTITY_DIR=/tmp/term-alice termlink channel dm c2e7f1cff6c213a0 \
      --send "hi bob, ready for the review?"
Posted to dm:04c54d00a7485964:c2e7f1cff6c213a0 — offset=0, ...

$ TERMLINK_IDENTITY_DIR=/tmp/term-bob   termlink channel dm 04c54d00a7485964 \
      --send "yes alice, joining now" --reply-to 0
Posted to dm:04c54d00a7485964:c2e7f1cff6c213a0 — offset=1, ...

$ TERMLINK_IDENTITY_DIR=/tmp/term-alice termlink channel react \
      dm:04c54d00a7485964:c2e7f1cff6c213a0 1 "👍"

$ TERMLINK_IDENTITY_DIR=/tmp/term-alice termlink channel dm c2e7f1cff6c213a0
[0] 04c... chat: hi bob, ready for the review?
[1 ↳0] c2e... chat: yes alice, joining now
    └─ reactions: 👍

$ TERMLINK_IDENTITY_DIR=/tmp/term-alice termlink channel dm c2e7f1cff6c213a0
(empty — alice's per-identity cursor caught up via T-1318)

$ TERMLINK_IDENTITY_DIR=/tmp/term-bob   termlink channel dm 04c54d00a7485964
[0] 04c... chat: hi bob, ready for the review?
[1 ↳0] c2e... chat: yes alice, joining now
    └─ reactions: 👍
```

T-1313 (threading), T-1314 (reactions), T-1315 (receipts via standalone
`channel ack`), T-1317 (--by-sender available via underlying flags),
T-1318 (per-identity cursor) all compose through `channel dm`.

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
target/debug/termlink channel dm --help 2>&1 | grep -q topic-only

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

### 2026-04-27T13:48:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1319-channel-dm-shorthand-auto-resolve-dmab-t.md
- **Context:** Initial task creation
