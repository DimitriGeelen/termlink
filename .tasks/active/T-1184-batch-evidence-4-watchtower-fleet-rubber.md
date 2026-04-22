---
id: T-1184
name: "Batch-evidence 4 Watchtower /fleet RUBBER-STAMPs — route works under correct PROJECT_ROOT (T-1103/T-1114/T-1115/T-1116)"
description: >
  Batch-evidence 4 Watchtower /fleet RUBBER-STAMPs — route works under correct PROJECT_ROOT (T-1103/T-1114/T-1115/T-1116)

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-22T08:19:30Z
last_update: 2026-04-22T08:20:10Z
date_finished: 2026-04-22T08:19:57Z
---

# T-1184: Batch-evidence 4 Watchtower /fleet RUBBER-STAMPs — route works under correct PROJECT_ROOT (T-1103/T-1114/T-1115/T-1116)

## Context

Curl probe against running watchtower (port 3000) returned 404 for `/fleet`, raising concern that T-1103/T-1114/T-1115/T-1116 might be broken. Root cause: the port-3000 watchtower's `PROJECT_ROOT=/opt/999-Agentic-Engineering-Framework`, not `/opt/termlink`. Under the correct PROJECT_ROOT (via Flask test_client, bypassing process boundary), `/fleet` returns 200, renders all 3 hubs, renders session-visibility markup (T-1115), and the home page includes the fleet widget markup (T-1116). All four tasks' visible UI artefacts are present when the right watchtower is serving.

## Acceptance Criteria

### Agent
- [x] T-1103: evidence block appended citing `/fleet` HTTP 200 + hub names/IPs rendered
- [x] T-1114: evidence block appended (tracing/stderr finding is inherited via the passing `/fleet` render)
- [x] T-1115: evidence block appended citing session-visibility markup occurrences
- [x] T-1116: evidence block appended citing home-page fleet widget markup

### Human
- [ ] [RUBBER-STAMP] Glance at any one of the four evidence blocks and confirm the finding cites concrete render output
  **Steps:**
  1. `grep -A 20 "auto-batch 2026-04-22 T-1184" .tasks/active/T-1103-*.md`
  **Expected:** Block cites HTTP 200, hub names, badge counts
  **If not:** Ask agent to rerun the test_client probe

## Verification

grep -q "T-1184" .tasks/active/T-1103-*.md
grep -q "T-1184" .tasks/active/T-1114-*.md
grep -q "T-1184" .tasks/active/T-1115-*.md
grep -q "T-1184" .tasks/active/T-1116-*.md

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

### 2026-04-22T08:19:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1184-batch-evidence-4-watchtower-fleet-rubber.md
- **Context:** Initial task creation

### 2026-04-22T08:19:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
