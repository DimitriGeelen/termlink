---
id: T-678
name: "Add --json support to discover --first (parity with list --first --json)"
description: >
  Add --json support to discover --first (parity with list --first --json)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:48:47Z
last_update: '2026-08-18T18:59:19Z'
date_finished: 2026-03-28T22:50:00Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:06Z'
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
  - ts: '2026-08-18T18:59:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-678: Add --json support to discover --first (parity with list --first --json)

## Context

`discover --first --json` doesn't output JSON — the --first check runs before --json and outputs plain text. Fix to output single JSON object like `list --first --json`.

## Acceptance Criteria

### Agent
- [x] `discover --first --json` outputs a single JSON session object
- [x] `discover --first` (no --json) still outputs display name
- [x] `discover --first --id` still outputs session ID
- [x] No-match case outputs JSON error when --json is set
- [x] Project compiles cleanly

## Verification

grep -q "if json" /opt/termlink/crates/termlink-cli/src/commands/metadata.rs

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

### 2026-03-28T22:48:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-678-add---json-support-to-discover---first-p.md
- **Context:** Initial task creation
