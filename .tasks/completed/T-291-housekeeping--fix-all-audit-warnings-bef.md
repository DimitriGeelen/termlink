---
id: T-291
name: "Housekeeping — fix all audit warnings before push to origin"
description: >
  Housekeeping — fix all audit warnings before push to origin

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-26T11:04:11Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-26T22:05:04Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=4 (body:fw-audit-or-doctor); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted); 
      F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-291: Housekeeping — fix all audit warnings before push to origin

## Context

Pre-push audit has 1 FAIL (CTL-009 T-258 inception without decision) and ~50 warnings (missing episodics, missing research artifacts, stale gaps.yaml, etc.). Fix all to get a clean push to origin before switching development to .107.

## Acceptance Criteria

### Agent
- [x] CTL-009 FAIL on T-258 resolved (decision added — prior session)
- [x] All missing episodic summaries generated (0 missing now)
- [x] 5 inception research artifacts created (T-205, T-206, T-208, T-209, T-245 — all exist)
- [x] Stale gaps.yaml removed (no longer present)
- [x] T-283 and T-287 research artifact references added to task Updates (prior session)
- [x] Pre-push audit passes with 0 FAILs (T-293 resolved all)
- [x] All changes committed and pushed to origin (f0879af pushed to onedev)

## Verification

# No missing episodics
test "$(python3 -c "import glob,os; completed=glob.glob('.tasks/completed/T-*.md'); missing=[f for f in completed if not os.path.exists('.context/episodic/'+os.path.basename(f).split('-')[0]+'-'+os.path.basename(f).split('-')[1]+'.yaml')]; print(len(missing))")" = "0"
# Research artifacts exist
test -f docs/reports/T-205-pyyaml-phantom-dependency.md
test -f docs/reports/T-206-remove-sudo-from-installer.md

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

### 2026-03-26T11:04:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-291-housekeeping--fix-all-audit-warnings-bef.md
- **Context:** Initial task creation

### 2026-03-26T22:05:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
