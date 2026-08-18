---
id: T-203
name: "termlink remote events — watch events on remote hub"
description: >
  termlink remote events — watch events on remote hub

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [cli, cross-machine, remote]
components: []
related_tasks: []
created: 2026-03-21T00:02:27Z
last_update: '2026-08-18T18:59:01Z'
date_finished: 2026-03-21T00:06:24Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:23Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=4 
      (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:01Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-203: termlink remote events — watch events on remote hub

## Context

Poll events from all sessions on a remote hub using `event.collect`. Continuously polls
with cursor tracking and Ctrl+C to stop. Mirrors local `event collect` output format.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::Events` variant with args: hub, secret-file/secret, scope, topic, targets, interval, count, json
- [x] `cmd_remote_events()` connects, authenticates, polls `event.collect` in a loop with cursor tracking
- [x] Output format matches local collect: `[session_name#seq] topic: payload (t=timestamp)`
- [x] `--topic` filters events by topic
- [x] `--count` limits total events before stopping
- [x] `--json` outputs each event as a JSON line
- [x] Ctrl+C gracefully stops with event count summary
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --workspace` passes (297 passed, 0 failed)

### Human
- [x] [REVIEW] Cross-machine test: watch events on .107
  **Steps:**
  1. `termlink remote events 192.168.10.107:9100 --secret-file /tmp/termlink-107-secret.txt`
  2. On .107, emit an event: `termlink event emit fw-agent test.topic '{"hello":"world"}'`
  3. Verify event appears in the remote events stream
  **Expected:** Event shows up with correct topic and payload
  **If not:** Check hub is forwarding events, try `--topic test.topic`

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "RemoteAction::Events" crates/termlink-cli/src/main.rs

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

### 2026-03-21T00:02:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-203-termlink-remote-events--watch-events-on-.md
- **Context:** Initial task creation

### 2026-03-21T00:06:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
