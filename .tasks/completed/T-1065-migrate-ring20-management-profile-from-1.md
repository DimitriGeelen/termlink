---
id: T-1065
name: "Migrate ring20-management profile from .109 to .126 (host renumbered)"
description: >
  Migrate ring20-management profile from .109 to .126 (host renumbered)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-15T17:22:40Z
last_update: '2026-08-18T18:58:43Z'
date_finished: 2026-04-15T17:24:19Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:44Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1065: Migrate ring20-management profile from .109 to .126 (host renumbered)

## Context

User reported "109 has become 126". Verified 2026-04-15: ping 192.168.10.126 OK (0.15ms), .109 no longer responds. Container `claude-dev` moved to .126. Hub process not yet running at new address (port 9100 refused). This task migrates the client-side profile config; reviving the hub process is T-1064 (owner=human).

## Acceptance Criteria

### Agent
- [x] `~/.termlink/hubs.toml` ring20-management address updated from `192.168.10.109:9100` to `192.168.10.126:9100`
- [x] Memory file `reference_ring20_infrastructure.md` updated to reflect .126
- [x] T-1064 updated with renumbering fact (still tracks hub-process-down investigation)
- [x] `termlink fleet doctor ring20-management` shows connection attempt against .126 (may still fail on port; success = new address reached)

## Verification

grep -q '192.168.10.126:9100' /root/.termlink/hubs.toml
! grep -q '192.168.10.109' /root/.termlink/hubs.toml

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

### 2026-04-15T17:22:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1065-migrate-ring20-management-profile-from-1.md
- **Context:** Initial task creation

### 2026-04-15T17:24:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
