---
id: T-559
name: "Add --count flag to termlink list for quick session counting"
description: >
  Add --count flag to termlink list for quick session counting

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/session.rs, 
      crates/termlink-cli/src/main.rs, 
      crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
created: 2026-03-28T10:07:58Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T10:09:48Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 7
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-559: Add --count flag to termlink list for quick session counting

## Context

Useful for scripting: `termlink list --count` just outputs the number.

## Acceptance Criteria

### Agent
- [x] `--count` flag added to List command in cli.rs
- [x] When --count is set, only the session count number is printed
- [x] Works with filters (--tag, --name, --role)
- [x] Builds without warnings
- [x] Integration test added (count with 1 session, count with 0 sessions)

### Human
<!-- No human ACs.
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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-28T10:07:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-559-add---count-flag-to-termlink-list-for-qu.md
- **Context:** Initial task creation

### 2026-03-28T10:09:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
