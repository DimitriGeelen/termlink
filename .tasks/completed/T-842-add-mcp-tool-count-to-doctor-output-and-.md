---
id: T-842
name: "Add MCP tool count to doctor output and version string"
description: >
  Add MCP tool count to doctor output and version string

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/infrastructure.rs, 
      crates/termlink-cli/src/main.rs, crates/termlink-mcp/src/lib.rs]
related_tasks: []
created: 2026-04-04T00:23:35Z
last_update: '2026-08-18T18:59:22Z'
date_finished: 2026-04-04T08:55:21Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:11Z'
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
      tier: 2
      effort: 6
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-842: Add MCP tool count to doctor output and version string

## Context

Add MCP tool count to `termlink version` and `termlink doctor` output so operators can verify MCP capability at a glance. The MCP `termlink_version` tool already returns tool count — this brings parity to the CLI.

## Acceptance Criteria

### Agent
- [x] `termlink_mcp` crate exposes a `pub fn tool_count() -> usize` function
- [x] `termlink version` text output includes MCP tool count (e.g. `termlink 0.9.414 (e6d55ea) [x86_64-unknown-linux-gnu] — 38 MCP tools`)
- [x] `termlink version --json` output includes `"mcp_tools"` field
- [x] `termlink doctor` version check includes MCP tool count
- [x] `cargo test --workspace` passes (755 tests)
- [x] `cargo clippy --workspace --all-targets` has no warnings

## Verification

cargo build --release -p termlink 2>&1 | tail -1
cargo test --workspace 2>&1 | tail -3
test "$(cargo clippy --workspace --all-targets 2>&1 | grep -c 'warning:')" = "0"

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

### 2026-04-04T00:23:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-842-add-mcp-tool-count-to-doctor-output-and-.md
- **Context:** Initial task creation

### 2026-04-04T00:23:46Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-04T08:55:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
