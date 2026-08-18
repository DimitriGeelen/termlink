---
id: T-058
name: "Pre-push hook audits wrong project — root cause analysis and remediation"
description: >
  Pre-push hook audits wrong project — root cause analysis and remediation

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-09T11:11:57Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-09T11:18:27Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=2 (body:lightly-promoted); 
      F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-058: Pre-push hook audits wrong project — root cause analysis and remediation

## Context

Pre-push hook audits the framework install directory instead of the project being pushed. Full RCA at `docs/reports/T-058-pre-push-hook-rca.md`. Root cause: hook doesn't pass PROJECT_ROOT env var to audit script.

## Acceptance Criteria

### Agent
- [x] Root cause identified and documented in RCA report
- [x] Local pre-push hook fixed (passes PROJECT_ROOT to audit script)
- [x] `.framework.yaml` framework_path uses stable symlink
- [x] Remediation instructions for framework agent written

## Verification

grep -q 'PROJECT_ROOT="\$PROJECT_ROOT"' .git/hooks/pre-push
grep -q '/usr/local/opt/agentic-fw/libexec' .framework.yaml
test -f docs/reports/T-058-pre-push-hook-rca.md

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

### 2026-03-09T11:11:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-058-pre-push-hook-audits-wrong-project--root.md
- **Context:** Initial task creation

### 2026-03-09T11:18:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
