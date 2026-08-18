---
id: T-676
name: "Add --check flag to vendor status that exits non-zero when update needed"
description: >
  Add --check flag to vendor status that exits non-zero when update needed

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-28T22:44:16Z
last_update: '2026-08-18T18:59:19Z'
date_finished: 2026-03-28T22:46:30Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:06Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
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

# T-676: Add --check flag to vendor status that exits non-zero when update needed

## Context

`vendor status` always exits 0. A `--check` flag that exits 1 when the vendor is outdated, MCP not configured, or gitignore missing would be useful for CI/startup scripts.

## Acceptance Criteria

### Agent
- [x] `--check` flag added to `VendorAction::Status` in cli.rs
- [x] `check` parameter threaded through to `cmd_vendor_status` in vendor.rs
- [x] Exit 1 when not vendored, version mismatch, MCP not configured, or gitignore missing
- [x] Works with `--json` (adds `"needs_update": true/false` field)
- [x] Project compiles cleanly

## Verification

grep -q "check" /opt/termlink/crates/termlink-cli/src/commands/vendor.rs

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

### 2026-03-28T22:44:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-676-add---check-flag-to-vendor-status-that-e.md
- **Context:** Initial task creation
