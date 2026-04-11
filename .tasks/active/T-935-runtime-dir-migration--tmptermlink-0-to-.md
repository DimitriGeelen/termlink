---
id: T-935
name: "Runtime dir migration — /tmp/termlink-0 to /var/lib/termlink"
description: >
  One-shot migration when switching from ad-hoc hub start (TERMLINK_RUNTIME_DIR defaulting to /tmp) to systemd-managed hub (TERMLINK_RUNTIME_DIR=/var/lib/termlink + StateDirectory=termlink). Two approaches: (a) script that copies hub.secret/cert/key to the new location and removes /tmp/termlink-0, (b) documentation telling operators to delete /tmp/termlink-0/ and let the unit recreate. Approach (b) is cheaper because restart rotates anyway. Lower priority — optional cleanup. From T-930 decomposition.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-930, T-931, T-933]
created: 2026-04-11T22:29:34Z
last_update: 2026-04-11T23:02:57Z
date_finished: null
---

# T-935: Runtime dir migration — /tmp/termlink-0 to /var/lib/termlink

## Context

T-930 decomposition lists T-935 as the final cleanup step after T-931..T-934
land. T-931's systemd unit moves the hub runtime dir from `/tmp/termlink-0/`
(tmpfs, wiped on reboot) to `/var/lib/termlink/` (persistent, StateDirectory).
On this box, `/tmp/termlink-0/` still contains stale cert/key/secret/sessions
from before the migration — the hub no longer reads from it, but the files
occupy disk and could confuse operators.

The migration is one-way and idempotent: delete the old dir, the systemd
unit is already pointing at the new one via `Environment=TERMLINK_RUNTIME_DIR`.
Documentation is sufficient — no code change needed.

## Acceptance Criteria

### Agent
- [ ] `docs/operations/termlink-hub-runtime-migration.md` exists with: (a) symptom to identify a stale `/tmp/termlink-0/`, (b) one-line command to remove it, (c) explanation of why the move happened, linking back to T-930 / T-931.
- [ ] Stale `/tmp/termlink-0/` on .107 is cleaned up as the live proof-of-concept for the documented procedure.
- [ ] Post-cleanup: systemd hub still active, `termlink doctor` still passes the ufw_listener check, persistent secret at `/var/lib/termlink/hub.secret` is unchanged.

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

### 2026-04-11T22:29:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-935-runtime-dir-migration--tmptermlink-0-to-.md
- **Context:** Initial task creation

### 2026-04-11T23:02:57Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
