---
id: T-156
name: "termlink-wrapped claude launch"
description: >
  Add a convenience script that launches Claude Code inside a TermLink-managed
  PTY session, making it discoverable and remotely observable via attach/stream.

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [remote-access, claude-fw]
components: []
related_tasks: [T-155, T-157, T-158]
created: 2026-03-17T09:45:39Z
last_update: '2026-08-18T18:58:51Z'
date_finished: 2026-03-20T13:41:12Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:03Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:51Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-156: termlink-wrapped claude launch

## Context

T-155 validated that Claude Code runs correctly inside a TermLink-managed PTY.
This task creates the launch script: `scripts/tl-claude.sh` — a thin wrapper
that starts Claude Code inside a TermLink session.

## Acceptance Criteria

### Agent
- [x] `scripts/tl-claude.sh` exists and is executable
- [x] Launches Claude Code inside a TermLink session (uses `termlink spawn`)
- [x] Session is named (default: `claude-master`, overridable with `--name`)
- [x] Tags session with `master,claude` for discovery
- [x] Supports `--backend` passthrough (auto/terminal/tmux/background)
- [x] Passes through all remaining args to `claude` (e.g., `-p`, `--resume`)
- [x] Cleanup on exit: deregisters TermLink session (handled by termlink spawn)
- [x] `--help` documents usage

### Human
- [x] [REVIEW] Launch `tl-claude.sh` and verify Claude Code TUI works normally
  **Steps:**
  1. Run `scripts/tl-claude.sh --name test-master`
  2. In another terminal: `termlink list` — verify session appears
  3. In another terminal: `termlink attach test-master` — verify TUI mirrors
  4. Type in the attached terminal — verify input reaches Claude Code
  **Expected:** Full bidirectional TUI mirroring
  **If not:** Note which step fails and what error appears

## Verification

test -x scripts/tl-claude.sh
scripts/tl-claude.sh --help | grep -q "TermLink"
grep -q "termlink" scripts/tl-claude.sh

## Decisions

## Updates

### 2026-03-17T09:45:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent

### 2026-03-17T10:38:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-20T13:12:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed

### 2026-03-20T13:41:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
