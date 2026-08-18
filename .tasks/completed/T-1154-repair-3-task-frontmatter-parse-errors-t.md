---
id: T-1154
name: "Repair 3 task frontmatter parse errors (T-915, T-940, T-936) — content leaked
  out of description folded scalar"
description: >
  Repair 3 task frontmatter parse errors (T-915, T-940, T-936) — content leaked out
  of description folded scalar

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-20T07:41:07Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-20T07:44:10Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:47Z'
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
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-1154: Repair 3 task frontmatter parse errors (T-915, T-940, T-936) — content leaked out of description folded scalar

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] T-915 frontmatter parses cleanly (was: "while scanning a simple key")
- [x] T-940 frontmatter parses cleanly (was: "while scanning an alias")
- [x] T-936 frontmatter parses cleanly (was: "mapping values are not allowed here")
- [x] Content preserved — extended findings moved into body under a `## Findings` / `## Problem Context` / `## Migration Inventory` section, not deleted
- [x] Full task-file YAML scan: 0 broken frontmatter in .tasks/active + .tasks/completed

## Verification

python3 /tmp/verify-task-frontmatter.py

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

### 2026-04-20T07:41:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1154-repair-3-task-frontmatter-parse-errors-t.md
- **Context:** Initial task creation

### 2026-04-20T07:44:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
