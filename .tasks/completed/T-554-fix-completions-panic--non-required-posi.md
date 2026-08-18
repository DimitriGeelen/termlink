---
id: T-554
name: "Fix completions panic — non-required positional before required"
description: >
  Fix completions panic — non-required positional before required

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T09:57:09Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T10:02:02Z
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
      blast_radius: 3
      tier: 2
      effort: 5
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-554: Fix completions panic — non-required positional before required

## Context

`termlink completions bash` panics: "Found non-required positional argument with a lower index than a required positional argument: target". A hidden backward-compat command has an optional `target` before a required arg.

## Acceptance Criteria

### Agent
- [x] Root cause identified — Interact, Inject (hidden), PtyCommand::Inject had Optional target before required command/text
- [x] Fix applied — made target required (String) in all 3 variants
- [x] `termlink completions bash` runs without panic
- [x] `termlink completions zsh` and `termlink completions fish` also work
- [x] All tests pass

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

### 2026-03-28T09:57:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-554-fix-completions-panic--non-required-posi.md
- **Context:** Initial task creation

### 2026-03-28T10:02:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
