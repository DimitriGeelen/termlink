---
id: T-026
name: "CLI inject subcommand — send keystrokes to PTY sessions"
description: >
  CLI inject subcommand — send keystrokes to PTY sessions

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T19:38:16Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T19:40:23Z
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
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-026: CLI inject subcommand — send keystrokes to PTY sessions

## Context

Add `termlink inject <target> <text> [--enter] [--key <name>]` for convenient keystroke injection into PTY sessions.

## Acceptance Criteria

### Agent
- [x] `Inject` variant in CLI Command enum with `target`, `text`, `--enter`, `--key` args
- [x] `cmd_inject` builds KeyEntry array and sends `command.inject`
- [x] `--enter` appends Enter key after text
- [x] `--key` sends a named key instead of text
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

### 2026-03-08T19:38:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-026-cli-inject-subcommand--send-keystrokes-t.md
- **Context:** Initial task creation

### 2026-03-08T19:40:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
