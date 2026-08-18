---
id: T-188
name: "Document upstream reporting workflow (dual-path)"
description: >
  Create docs/guides/upstream-reporting.md documenting the dual-path upstream
  workflow (TermLink primary, fw upstream fallback) per T-180 GO decision.
  Also send fw upstream proposal to framework agent via termlink remote inject.

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [docs, upstream, workflow]
components: []
related_tasks: [T-180, T-186, T-187]
created: 2026-03-19T11:44:09Z
last_update: '2026-08-18T18:58:58Z'
date_finished: 2026-03-20T13:12:26Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:17Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=2 
      (body:env-class-handled); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:58Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-188: Document upstream reporting workflow (dual-path)

## Context

Build task from T-180 GO. Design: `docs/reports/T-180-upstream-reporting-design.md`.

## Acceptance Criteria

### Agent
- [x] `docs/guides/upstream-reporting.md` created with both paths documented
- [x] TermLink primary path: prerequisites, command, prompt template, examples
- [x] fw upstream fallback path: proposed command spec, output format, delivery options
- [x] `fw upstream report` proposal injected into framework agent via `termlink remote inject` (1284 bytes)

### Human
- [x] [RUBBER-STAMP] Review upstream-reporting.md for clarity and completeness
  **Steps:**
  1. Read `docs/guides/upstream-reporting.md`
  2. Verify both paths have clear step-by-step instructions
  **Expected:** A developer unfamiliar with the project can follow either path
  **If not:** Note which sections need clarification

## Verification

test -f docs/guides/upstream-reporting.md
grep -q "termlink remote inject" docs/guides/upstream-reporting.md
grep -q "fw upstream report" docs/guides/upstream-reporting.md

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

### 2026-03-19T11:44:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-188-document-upstream-reporting-workflow-dua.md
- **Context:** Initial task creation

### 2026-03-19T11:46:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-20T13:12:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
