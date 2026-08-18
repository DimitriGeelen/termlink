---
id: T-1152
name: "Fix episodic generator — empty summary when task description is YAML fold marker"
description: >
  Fix episodic generator — empty summary when task description is YAML fold marker

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-20T07:22:39Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-20T07:24:21Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1152: Fix episodic generator — empty summary when task description is YAML fold marker

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] In `.agentic-framework/agents/context/lib/episodic.sh` line 87, sanitize description extraction so a bare YAML fold marker (`>`) becomes empty — not literal `>`
- [x] In `summary_text` fallback (line 218-220), treat `>` and `|` as empty so it proceeds to task-name fallback
- [x] Fall back to `task_name` when description is also empty so summary is never just `>`
- [x] Regenerate an episodic for a recent task (T-1150 or T-1151) and verify `summary:` contains real text, not `>`

## Verification

bash -c 'grep -q "summary: |" .context/episodic/T-1151.yaml && ! grep -A1 "^summary: |" .context/episodic/T-1151.yaml | tail -1 | grep -q "^  >$"'

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

### 2026-04-20T07:22:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1152-fix-episodic-generator--empty-summary-wh.md
- **Context:** Initial task creation

### 2026-04-20T07:24:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
