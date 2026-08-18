---
id: T-045
name: "CLI clean command — reap stale sessions from runtime directory"
description: >
  CLI clean command — reap stale sessions from runtime directory

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T22:33:35Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T22:37:18Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
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
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-045: CLI clean command — reap stale sessions from runtime directory

## Context

Sessions that crash or exit without deregistering leave orphaned socket+JSON files. The `list` command cleans them on-demand, but there's no explicit command for deliberate batch cleanup with reporting.

## Acceptance Criteria

### Agent
- [x] `termlink clean` subcommand scans runtime dir and removes stale sessions
- [x] `--dry-run` flag shows what would be removed without deleting
- [x] Output table shows ID, name, PID, created time for each stale session
- [x] `clean_stale_sessions()` function in manager module for reuse
- [x] Unit test for dry-run vs actual cleanup
- [x] All existing tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -4

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

### 2026-03-08T22:33:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-045-cli-clean-command--reap-stale-sessions-f.md
- **Context:** Initial task creation

### 2026-03-08T22:37:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
