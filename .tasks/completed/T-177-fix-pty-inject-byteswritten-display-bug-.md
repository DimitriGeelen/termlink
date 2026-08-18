---
id: T-177
name: "Fix pty inject bytes_written display bug (always shows 0)"
description: >
  cmd_inject reads result[bytes_written] but handler returns bytes_len. One-line fix
  in main.rs:1765.

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [bug, cli, inject]
components: []
related_tasks: []
created: 2026-03-18T22:19:28Z
last_update: '2026-08-18T18:58:55Z'
date_finished: 2026-03-18T22:22:45Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:12Z'
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
  - ts: '2026-08-18T18:58:55Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-177: Fix pty inject bytes_written display bug (always shows 0)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] main.rs:1765 reads `bytes_len` instead of `bytes_written`
- [x] `cargo build --release` succeeds
- [x] Existing tests pass

<!-- No human ACs — purely mechanical fix -->

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

### 2026-03-18T22:19:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-177-fix-pty-inject-byteswritten-display-bug-.md
- **Context:** Initial task creation

### 2026-03-18T22:20:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T22:22:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
