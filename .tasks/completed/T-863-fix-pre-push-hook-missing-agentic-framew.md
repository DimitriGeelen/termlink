---
id: T-863
name: "Fix pre-push hook missing .agentic-framework audit path"
description: >
  Fix pre-push hook missing .agentic-framework audit path

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-04T20:43:14Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T20:50:08Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 1
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-863: Fix pre-push hook missing .agentic-framework audit path

## Context

Pre-push hook at `.git/hooks/pre-push` only checks `.framework.yaml -> framework_path` and `$PROJECT_ROOT/agents/audit/audit.sh` for the audit script. Since T-498 removed `.framework.yaml` and the project uses vendored framework at `.agentic-framework/`, it never finds the audit script and blocks all pushes.

## Acceptance Criteria

### Agent
- [x] Pre-push hook checks `$PROJECT_ROOT/.agentic-framework/agents/audit/audit.sh` path
- [x] `git push origin main` succeeds (8 unpushed commits reach OneDev)
- [x] Error message lists all checked paths including `.agentic-framework/`

## Verification

grep -q '.agentic-framework/agents/audit/audit.sh' .git/hooks/pre-push

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

### 2026-04-04T20:43:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-863-fix-pre-push-hook-missing-agentic-framew.md
- **Context:** Initial task creation

### 2026-04-04T20:50:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
