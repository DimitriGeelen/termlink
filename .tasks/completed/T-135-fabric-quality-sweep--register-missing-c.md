---
id: T-135
name: "Fabric quality sweep — register missing cards, fix deps, sync subsystems"
description: >
  Fabric quality sweep — register missing cards, fix deps, sync subsystems

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-14T16:24:11Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-03-14T16:27:13Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:55Z'
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
  - ts: '2026-08-18T18:58:48Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-135: Fabric quality sweep — register missing cards, fix deps, sync subsystems

## Context

Fabric quality investigation found: 2 missing transport.rs cards, 4 lib cards with incomplete deps, subsystems.yaml out of sync with 5+ missing components.

## Acceptance Criteria

### Agent
- [x] transport.rs cards created for protocol and session crates
- [x] lib.yaml cards updated with complete depends_on (protocol: +events +transport, session: +auth +transport, hub: +pidfile +supervisor)
- [x] client.yaml updated with control.rs and transport.rs deps
- [x] subsystems.yaml synced with all registered components
- [x] All YAML files parse correctly

## Verification

python3 -c "import yaml; yaml.safe_load(open('.fabric/subsystems.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('.fabric/components/crates-termlink-protocol-src-transport.yaml'))"
python3 -c "import yaml; yaml.safe_load(open('.fabric/components/crates-termlink-session-src-transport.yaml'))"
test -f .fabric/components/crates-termlink-protocol-src-transport.yaml
test -f .fabric/components/crates-termlink-session-src-transport.yaml

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

### 2026-03-14T16:24:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-135-fabric-quality-sweep--register-missing-c.md
- **Context:** Initial task creation

### 2026-03-14T16:27:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
