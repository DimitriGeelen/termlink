---
id: T-028
name: "CLI signal subcommand — send signals to session child processes"
description: >
  CLI signal subcommand — send signals to session child processes

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:54:35Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T19:57:07Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:39Z'
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
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-028: CLI signal subcommand — send signals to session child processes

## Context

Add `termlink signal <target> <signal>` to send signals to session processes. Accepts names (TERM, INT, KILL, HUP, etc.) or numbers, with optional SIG prefix.

## Acceptance Criteria

### Agent
- [x] `Signal` variant in CLI with `target` and `signal` args
- [x] `parse_signal` maps names (case-insensitive, SIG prefix optional) to libc constants
- [x] Falls back to numeric parsing for raw signal numbers
- [x] Builds and all 102 tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test --workspace 2>&1 | tail -1

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

### 2026-03-08T19:54:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-028-cli-signal-subcommand--send-signals-to-s.md
- **Context:** Initial task creation

### 2026-03-08T19:57:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
