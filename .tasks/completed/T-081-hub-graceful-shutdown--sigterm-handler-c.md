---
id: T-081
name: "Hub graceful shutdown — SIGTERM handler, connection drain"
description: >
  SIGTERM/SIGINT signal handler for hub. Stop accepting connections, drain active
  with 5s timeout, remove pidfile+socket, exit 0.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-10T22:10:39Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T22:17:49Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-081: Hub graceful shutdown — SIGTERM handler, connection drain

## Context

From T-066 inception (GO). See [docs/reports/T-066-hub-daemon-inception.md].

## Acceptance Criteria

### Agent
- [x] Hub server handles SIGTERM for graceful shutdown (via ShutdownHandle)
- [x] Accept loop stops accepting new connections on shutdown signal
- [x] Pidfile and socket cleaned up on SIGTERM
- [x] Hub `run()` returns a ShutdownHandle for external signal integration
- [x] Tests verify shutdown behavior (2 new tests: stop + drain)
- [x] All existing hub tests continue to pass (24 total)
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-10T22:10:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-081-hub-graceful-shutdown--sigterm-handler-c.md
- **Context:** Initial task creation

### 2026-03-10T22:15:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T22:17:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
