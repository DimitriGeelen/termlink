---
id: T-017
name: "CLI commands: register, list, ping, status"
description: >
  CLI commands: register, list, ping, status

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T16:44:18Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T16:58:28Z
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
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-017: CLI commands: register, list, ping, status

## Context

CLI subcommands for TermLink: `register` (start a session), `list` (show sessions), `ping` (verify session), `status` (query session state). Uses clap for arg parsing, connects to session sockets for ping/status.

## Acceptance Criteria

### Agent
- [x] `termlink register` starts a session with optional `--name` flag
- [x] `termlink list` shows all registered sessions with state
- [x] `termlink ping <target>` connects to session socket, sends termlink.ping
- [x] `termlink status <target>` connects to session socket, sends query.status
- [x] `--help` works for all subcommands
- [x] `cargo test --workspace` passes
- [x] `cargo build --workspace` produces working binary

## Verification

PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace
PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings
PATH="$HOME/.cargo/bin:$PATH" cargo run -- --help

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

### 2026-03-08T16:44:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-017-cli-commands-register-list-ping-status.md
- **Context:** Initial task creation

### 2026-03-08T16:58:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
