---
id: T-771
name: "Add token create+inspect roundtrip integration test"
description: >
  Add token create+inspect roundtrip integration test

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:15:06Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T23:17:28Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-771: Add token create+inspect roundtrip integration test

## Context

Token create/inspect are security-critical commands. Only the inspect error case is tested. Need a full roundtrip: register with --token-secret, create token, inspect it, verify payload.

## Acceptance Criteria

### Agent
- [x] Add test: register with --token-secret, token create --json, verify ok:true + token field
- [x] Add test: create token then inspect --json, verify payload roundtrip (scope, session, expired fields)
- [x] Both tests pass via `cargo test -p termlink cli_token`

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

test -f crates/termlink-cli/tests/cli_integration.rs

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

### 2026-03-29T23:15:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-771-add-token-createinspect-roundtrip-integr.md
- **Context:** Initial task creation

### 2026-03-29T23:17:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
