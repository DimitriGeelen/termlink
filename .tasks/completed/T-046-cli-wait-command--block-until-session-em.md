---
id: T-046
name: "CLI wait command — block until session emits matching event"
description: >
  CLI wait command — block until session emits matching event

status: work-completed
workflow_type: build
owner: human
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T22:51:26Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-08T22:54:04Z
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
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-046: CLI wait command — block until session emits matching event

## Context

Enables shell scripting with TermLink events: `termlink wait my-session --topic build.complete && deploy`.

## Acceptance Criteria

### Agent
- [x] `termlink wait <target> --topic <topic>` blocks until matching event
- [x] Prints event payload (or topic name if empty) on match, exits 0
- [x] `--timeout` flag exits non-zero on expiry
- [x] Starts from current next_seq (only new events)
- [x] Ctrl+C interrupts cleanly
- [x] Builds and all tests pass

### Human
- [x] [RUBBER-STAMP] Verify wait + emit workflow across terminals
  **Steps:**
  1. Terminal 1: `termlink register --name test1`
  2. Terminal 2: `termlink wait test1 --topic hello --timeout 10`
  3. Terminal 3: `termlink emit test1 hello -p '{"msg":"hi"}'`
  **Expected:** Terminal 2 prints `{"msg":"hi"}` and exits 0
  **If not:** Report which step failed

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

### 2026-03-08T22:51:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-046-cli-wait-command--block-until-session-em.md
- **Context:** Initial task creation

### 2026-03-08T22:54:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
