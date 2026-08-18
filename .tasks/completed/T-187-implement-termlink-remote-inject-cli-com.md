---
id: T-187
name: "Implement termlink remote inject CLI command"
description: >
  Implement `termlink remote inject` CLI command per T-186 design (Variant D).
  Wraps TOFU TLS + HMAC auth + hub-routed inject into a single command.
  Design: docs/reports/T-186-inject-remote-cli-design.md

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [cli, cross-machine, remote]
components: []
related_tasks: [T-186, T-182, T-183, T-184]
created: 2026-03-19T05:50:40Z
last_update: '2026-08-18T18:58:58Z'
date_finished: 2026-03-19T05:59:50Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:17Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 1
      D3: 0
      D4: 4
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=4 (body:cross-machine); F-RECALL=2 
      (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:58Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-187: Implement termlink remote inject CLI command

## Context

Implements Variant D from T-186 inception. Design: `docs/reports/T-186-inject-remote-cli-design.md`.

## Acceptance Criteria

### Agent
- [x] `Remote` subcommand with `Inject` action added to CLI enum
- [x] `cmd_remote_inject()` handles: secret parsing, TOFU connect, hub auth, inject with target routing
- [x] `--secret-file` and `--secret` options for hex secret input
- [x] `--enter`, `--key`, `--delay-ms`, `--scope`, `--json` options supported
- [x] Clear error messages for: bad secret, TOFU violation, auth failure, session not found, connection refused
- [x] `cargo build --package termlink` compiles without errors
- [x] `termlink remote inject --help` shows correct usage

### Human
- [x] [REVIEW] Test cross-machine inject against remote hub on 192.168.10.107:9100
  **Steps:**
  1. Ensure hub is running on remote: `ssh mint-dev 'termlink hub status'`
  2. Run: `termlink remote inject 192.168.10.107:9100 fw-agent "echo hello from remote inject" --secret-file ~/.termlink/hub.secret --enter`
  **Expected:** "Injected N bytes into fw-agent on 192.168.10.107:9100"
  **If not:** Check `termlink remote inject` error output, verify hub is running with `--tcp`

## Verification

# Must compile
/Users/dimidev32/.cargo/bin/cargo build --package termlink 2>&1 | tail -1 | grep -qv "^error"
# Help text works
/Users/dimidev32/.cargo/bin/cargo run --package termlink -- remote inject --help 2>&1 | grep -q "secret"

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

### 2026-03-19T05:50:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-187-implement-termlink-remote-inject-cli-com.md
- **Context:** Initial task creation

### 2026-03-19T05:59:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
