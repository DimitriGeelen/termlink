---
id: T-1097
name: "Positive evidence for review queue — programmatic + termlink e2e + Playwright"
description: >
  Positive evidence for review queue — programmatic + termlink e2e + Playwright

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-16T22:05:26Z
last_update: '2026-08-18T18:58:43Z'
date_finished: 2026-04-16T23:04:09Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:45Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=0 (no-signal); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-1097: Positive evidence for review queue — programmatic + termlink e2e + Playwright

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Approach A: programmatic evidence — all 47 RUBBER-STAMP tasks now have evidence (T-936, T-940 + 5 new test tasks added this round)
- [x] Approach B: termlink e2e — 6 validations via local hub (inbox status, remote inbox, tofu list, discover, file send, fleet doctor JSON)
- [x] Approach C: Playwright — 3 Watchtower pages validated (review/T-1007, /approvals, /tasks) — all render correctly
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-16T22:05:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1097-positive-evidence-for-review-queue--prog.md
- **Context:** Initial task creation

### 2026-04-16T23:04:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
