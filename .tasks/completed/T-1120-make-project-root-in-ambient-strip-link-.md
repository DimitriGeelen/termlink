---
id: T-1120
name: "Make project root in ambient strip link to /project"
description: >
  Make project root in ambient strip link to /project

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:20:19Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-18T09:21:06Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:46Z'
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
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1120: Make project root in ambient strip link to /project

## Context

The project root display in the ambient strip is plain text. Linking it to /project (project docs page) makes it a discoverable shortcut to project-specific documentation.

## Acceptance Criteria

### Agent
- [x] Project root in ambient strip is wrapped in an anchor pointing to /project
- [x] Existing pages still render

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

curl -sf http://localhost:3000/ > /dev/null
curl -sf http://localhost:3000/ | grep -q 'href="/project"'

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

### 2026-04-18T09:20:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1120-make-project-root-in-ambient-strip-link-.md
- **Context:** Initial task creation

### 2026-04-18T09:21:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
