---
id: T-874
name: "Extract format_age() to termlink-protocol shared crate — remove duplication
  between CLI and MCP"
description: >
  Extract format_age() to termlink-protocol shared crate — remove duplication between
  CLI and MCP

status: work-completed
workflow_type: refactor
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/session.rs, 
      crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/lib.rs]
related_tasks: []
created: 2026-04-04T22:56:29Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T23:02:07Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:12Z'
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
  - ts: '2026-08-18T18:59:22Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 3
      effort: 6
    rationale: blast_radius=3 (no-signal); tier=3 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-874: Extract format_age() to termlink-protocol shared crate — remove duplication between CLI and MCP

## Context

`format_age()` is duplicated identically in CLI session.rs and MCP tools.rs. Move to
termlink-protocol (lowest shared crate) and replace both call sites.

## Acceptance Criteria

### Agent
- [x] `format_age()` defined as pub fn in `termlink-protocol` crate
- [x] CLI session.rs uses `termlink_protocol::format_age` instead of local copy
- [x] MCP tools.rs uses `termlink_protocol::format_age` instead of local copy
- [x] Existing format_age unit tests moved to protocol crate (6 tests)
- [x] `cargo clippy --workspace` passes with no warnings
- [x] `cargo test --workspace` passes (857 tests, 0 failures)

## Verification

# format_age exists in protocol crate
grep -q "pub fn format_age" crates/termlink-protocol/src/lib.rs
# No local format_age in CLI or MCP
! grep -q "^fn format_age" crates/termlink-cli/src/commands/session.rs
! grep -q "^fn format_age" crates/termlink-mcp/src/tools.rs
# Tests pass
cargo test --workspace 2>&1 | tail -3 | grep -q "0 failed"

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

### 2026-04-04T22:56:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-874-extract-formatage-to-termlink-protocol-s.md
- **Context:** Initial task creation

### 2026-04-04T23:02:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
