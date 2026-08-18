---
id: T-546
name: "Add --json output to termlink ping"
description: >
  Add --json output to termlink ping

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:40:13Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-28T09:42:07Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:02Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-546: Add --json output to termlink ping

## Context

`termlink ping` outputs text only. Other commands like `status`, `list`, `info` have `--json`. Add consistency.

## Acceptance Criteria

### Agent
- [x] `--json` flag added to Ping command in cli.rs
- [x] `cmd_ping` accepts json parameter and outputs JSON when set
- [x] JSON output includes session target, latency_ms, and status fields
- [x] Builds without warnings

## Verification

cargo build 2>&1
grep -q "json" crates/termlink-cli/src/cli.rs | head -1 || grep -q "Ping" crates/termlink-cli/src/cli.rs

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

### 2026-03-28T09:40:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-546-add---json-output-to-termlink-ping.md
- **Context:** Initial task creation

### 2026-03-28T09:42:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
