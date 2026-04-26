---
id: T-1296
name: "Migrate ring20-dashboard hub runtime_dir (.121) — same as T-1294"
description: >
  Mirror of T-1294 for the OTHER ring20 hub at .121 (proxmox4 ct 101 ring20-dashboard). T-1294 fixed .122 by moving runtime_dir from /tmp/termlink-0/ to /var/lib/termlink/ in ring20-watchdog.sh. .121 still runs on /tmp/termlink-0/ with the same systemd-tmpfiles 'D /tmp' wipe behavior, so it has the same G-011 cascade pattern (every CT 101 reboot wipes hub.secret, all peer caches go stale). Bonus: T-1294 introduced a regression where .122's watchdog peer-refresh function expands TERMLINK_RUNTIME_DIR to OUR local path (/var/lib/termlink/) but tries to extract from .121 — currently broken until .121 is also on /var/lib/termlink/. Completing this task heals both: .121 cascade prevention AND restores cross-host peer-refresh.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [auth, infrastructure, ring20-dashboard, G-011, runtime_dir, T-1294-followup]
components: []
related_tasks: [T-1294, T-1290, T-1291, T-935]
created: 2026-04-26T14:27:17Z
last_update: 2026-04-26T14:27:17Z
date_finished: null
---

# T-1296: Migrate ring20-dashboard hub runtime_dir (.121) — same as T-1294

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-26T14:27:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1296-migrate-ring20-dashboard-hub-runtimedir-.md
- **Context:** Initial task creation
