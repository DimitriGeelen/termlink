---
id: T-1075
name: "Implement cross-agent learnings exchange — cron + script + dedup"
description: >
  Implement cross-agent learnings exchange — cron + script + dedup

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-15T21:38:41Z
last_update: 2026-04-15T21:47:57Z
date_finished: 2026-04-15T21:47:57Z
---

# T-1075: Implement cross-agent learnings exchange — cron + script + dedup

## Context

Implements the T-1074 GO decision. Minimal build:

1. `scripts/learnings-exchange.sh` — iterates over reachable termlink peers (from `termlink fleet doctor`-able profiles + local sessions), asks each for learnings added since the last known timestamp, dedupes by PL-ID against local `.context/project/learnings.yaml`, writes new entries as pickup envelopes under `.context/pickup/inbox/` so a human can review before promoting.
2. `/etc/cron.d/agentic-learnings-exchange-termlink` — runs the script every 15 min.
3. **State file** `.context/working/.learnings-exchange-cursor.yaml` — maps peer-id → last-seen PL-ID or timestamp so we only pull deltas.
4. **Wire format** — termlink `remote_call` / `event.broadcast` with a JSON payload `{"q": "learnings.delta", "since": "<iso8601>"}`. The responder side (future work on peer projects) isn't in scope here; we implement the asker side first, and a fallback path that reads peer files when we're on-host (like ring20 containers over termlink remote exec → cat).

Design note: implement ASKER side only in this task. RESPONDER side is a separate build task per-project that peers will file themselves after receiving the propagation envelope (T-1074 already delivered).

## Acceptance Criteria

### Agent
- [x] `scripts/learnings-exchange.sh` exists, executable, runs without error with 0 peers
- [x] `scripts/learnings-exchange.sh` handles a down-peer gracefully (no exit-1, logs a warning)
- [x] `/etc/cron.d/agentic-learnings-exchange-termlink` installed, runs every 15 min via cron.d
- [x] Cursor state file at `.context/working/.learnings-exchange-cursor.yaml` (may be empty initially)
- [x] First cron run logged to syslog under tag `agentic-learnings`
- [x] ShellCheck clean on the script

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

test -x scripts/learnings-exchange.sh
test -f /etc/cron.d/agentic-learnings-exchange-termlink
bash -n scripts/learnings-exchange.sh

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

### 2026-04-15T21:38:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1075-implement-cross-agent-learnings-exchange.md
- **Context:** Initial task creation

### 2026-04-15T21:47:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
