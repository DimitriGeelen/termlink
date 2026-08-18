---
id: T-825
name: "Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback"
description: >
  Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-03T20:15:36Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-04-03T20:32:32Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-825: Fix pre-push hook — resolve audit.sh via .agentic-framework/ fallback

## Context

Pre-push hook blocks push because `framework_path:` was removed from `.framework.yaml` (T-498) and fallback only checks `$PROJECT_ROOT/agents/audit/audit.sh`. Audit script lives at `.agentic-framework/agents/audit/audit.sh`.

## Acceptance Criteria

### Agent
- [x] Pre-push hook resolves audit.sh via `.agentic-framework/agents/audit/audit.sh` fallback
- [x] `git push origin main` succeeds (8f29fce..6184b37)

## Verification

grep -q 'agentic-framework' .git/hooks/pre-push

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

### 2026-04-03T20:15:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-825-fix-pre-push-hook--resolve-auditsh-via-a.md
- **Context:** Initial task creation

### 2026-04-03T20:32:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
