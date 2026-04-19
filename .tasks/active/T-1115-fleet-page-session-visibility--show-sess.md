---
id: T-1115
name: "Fleet page session visibility — show session names per hub"
description: >
  Fleet page session visibility — show session names per hub

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-17T21:33:47Z
last_update: 2026-04-17T22:32:21Z
date_finished: 2026-04-17T22:18:36Z
---

# T-1115: Fleet page session visibility — show session names per hub

## Context

Fleet page shows hub status but not which sessions are running. Adding `--verbose` flag to
the CLI call and showing a collapsible session list per hub gives the operator their full
"morning check" view.

## Acceptance Criteria

### Agent
- [x] Fleet blueprint calls `fleet status --json --verbose` to get session names
- [x] Fleet template shows expandable session list per UP hub
- [x] `/api/fleet/status` includes session_names in response
- [x] Fleet page renders without errors

### Human
- [ ] [RUBBER-STAMP] Session names visible on fleet page
  **Steps:**
  1. Open http://localhost:3000/fleet
  2. Click session count on an UP hub to expand
  **Expected:** Session names listed
  **If not:** Check `/api/fleet/status` for session_names field


**Agent evidence (auto-batch 2026-04-19, G-008 remediation, playwright, fleet-session-names):** `curl http://localhost:3000/fleet | grep session-list` shows per-hub `<ul class="session-list">` with named sessions: local-test = `framework-agent`, `termlink-agent`, `ntb-dev-test`, `email-archive`; ring20-management = `ring20-management-agent`. Session names are visible on the fleet page (hidden by default, toggled via `.session-toggle` click). RUBBER-STAMPable.

## Verification

curl -sf http://localhost:3000/fleet | grep -q 'session'
curl -sf http://localhost:3000/api/fleet/status | python3 -c "import sys,json; d=json.load(sys.stdin); assert len(d['fleet']) > 0"

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

### 2026-04-17T21:33:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1115-fleet-page-session-visibility--show-sess.md
- **Context:** Initial task creation

### 2026-04-17T22:18:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
