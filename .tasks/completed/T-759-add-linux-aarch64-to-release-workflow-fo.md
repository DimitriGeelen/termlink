---
id: T-759
name: "Add Linux aarch64 to release workflow for ARM64 support"
description: >
  Add Linux aarch64 to release workflow for ARM64 support

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T20:02:23Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T20:03:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-759: Add Linux aarch64 to release workflow for ARM64 support

## Context

Release workflow builds macOS (aarch64 + x86_64) and Linux x86_64 but lacks Linux aarch64 for ARM64 (AWS Graviton, Raspberry Pi, etc).

## Acceptance Criteria

### Agent
- [x] Linux aarch64 build job added to release.yml using cross-compilation
- [x] termlink-linux-aarch64 artifact included in release assets
- [x] Checksums include all 4 binaries (darwin-aarch64, darwin-x86_64, linux-x86_64, linux-aarch64)
- [x] YAML is valid

## Verification

python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
grep -q "aarch64" .github/workflows/release.yml
grep -q "termlink-linux-aarch64" .github/workflows/release.yml

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

### 2026-03-29T20:02:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-759-add-linux-aarch64-to-release-workflow-fo.md
- **Context:** Initial task creation

### 2026-03-29T20:03:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
