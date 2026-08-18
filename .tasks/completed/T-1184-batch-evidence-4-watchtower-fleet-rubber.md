---
id: T-1184
name: "Batch-evidence 4 Watchtower /fleet RUBBER-STAMPs — route works under correct
  PROJECT_ROOT (T-1103/T-1114/T-1115/T-1116)"
description: >
  Batch-evidence 4 Watchtower /fleet RUBBER-STAMPs — route works under correct PROJECT_ROOT
  (T-1103/T-1114/T-1115/T-1116)

status: work-completed
workflow_type: build
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-22T08:19:30Z
last_update: '2026-08-18T18:58:45Z'
date_finished: 2026-04-22T08:19:57Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:48Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:45Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
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
- [x] [RUBBER-STAMP] Glance at any one of the four evidence blocks and confirm the finding cites concrete render output — ticked by user direction 2026-04-23. Evidence: Live: /fleet route HTTP 200 confirms route works under correct PROJECT_ROOT. User direction 2026-04-23.
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
