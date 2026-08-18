---
id: T-219
name: "Fix GitHub repo URL in Cargo.toml and add github remote"
description: >
  Fix GitHub repo URL in Cargo.toml and add github remote

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:22:23Z
last_update: '2026-08-18T18:59:05Z'
date_finished: 2026-03-22T21:23:10Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:33Z'
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
  - ts: '2026-08-18T18:59:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-219: Fix GitHub repo URL in Cargo.toml and add github remote

## Context

Cargo.toml has `repository = "https://github.com/dimidev32/termlink"` but all docs, specs, README, and Homebrew formula reference `DimitriGeelen/termlink`. Also, no `github` remote is configured — only OneDev origin exists. The release workflow and Homebrew formula depend on the GitHub mirror.

## Acceptance Criteria

### Agent
- [x] Cargo.toml repository URL matches docs (`DimitriGeelen/termlink`)
- [x] `github` remote added pointing to `DimitriGeelen/termlink`

## Verification

grep -q 'DimitriGeelen/termlink' Cargo.toml
git remote get-url github

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

### 2026-03-22T21:22:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-219-fix-github-repo-url-in-cargotoml-and-add.md
- **Context:** Initial task creation

### 2026-03-22T21:23:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
