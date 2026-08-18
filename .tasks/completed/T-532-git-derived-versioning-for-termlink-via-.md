---
id: T-532
name: "Git-derived versioning for TermLink via build.rs"
description: >
  Git-derived versioning for TermLink via build.rs

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-27T17:26:25Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-27T17:30:46Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-532: Git-derived versioning for TermLink via build.rs

## Context

Port the framework's T-648 git-derived versioning to TermLink. `build.rs` uses `git describe --tags` to derive version at compile time: tagged commit = exact version, N commits after tag = `major.minor.N`. No more manual Cargo.toml bumps for patch versions.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-cli/build.rs` exists and derives version from `git describe`
- [x] Tagged commit `v0.9.0` produces version `0.9.0`
- [x] N commits after tag produces `major.minor.N` (verified: 1 commit → `0.9.1`)
- [x] Falls back to Cargo.toml version when no git tags exist
- [x] `cargo build` succeeds
- [x] `cargo clippy` has no errors in build.rs
- [x] `termlink --version` outputs the git-derived version

## Verification

test -f crates/termlink-cli/build.rs
PATH="$HOME/.cargo/bin:$PATH" cargo build 2>&1 | tail -1
PATH="$HOME/.cargo/bin:$PATH" cargo run -- --version 2>&1 | grep -qE '^termlink [0-9]+\.[0-9]+\.[0-9]+'

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

### 2026-03-27T17:26:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-532-git-derived-versioning-for-termlink-via-.md
- **Context:** Initial task creation

### 2026-03-27T17:30:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
