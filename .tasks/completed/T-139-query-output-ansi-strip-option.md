---
id: T-139
name: "query.output --strip-ansi option"
description: >
  Add optional ANSI escape sequence stripping to the query.output RPC handler
  and CLI output command. Makes scrollback output clean text for automated parsing.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [protocol, scrollback, ansi]
components: []
related_tasks: [T-136, T-137]
created: 2026-03-14T17:07:00Z
last_update: '2026-08-18T18:58:49Z'
date_finished: 2026-03-14T22:59:56Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:57Z'
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
  - ts: '2026-08-18T18:58:49Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-139: query.output --strip-ansi option

## Context

T-136 spike found that scrollback contains raw ANSI escape sequences which
break JSON parsers and make pattern matching harder. Add a `strip_ansi` param
to the `query.output` RPC handler that strips escape codes server-side.

## Acceptance Criteria

### Agent
- [x] `query.output` RPC accepts optional `strip_ansi: true` param
- [x] When set, output has ANSI escape sequences removed before returning
- [x] `termlink output <session> --strip-ansi` CLI flag
- [x] Backward compatible — default behavior unchanged
- [x] Tests for ANSI stripping (at least 2)
- [x] All existing tests pass (260 total)

## Verification

/Users/dimidev32/.cargo/bin/cargo test --workspace

## Updates

### 2026-03-14T17:07:00Z — task-created
- Enhancement from T-136 spike findings (ANSI in scrollback breaks json.load)

### 2026-03-14T22:48:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-14T22:59:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
