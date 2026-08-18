---
id: T-798
name: "Add unit tests for CLI command modules (vendor, push, token, metadata)"
description: >
  CLI command unit tests for vendored logic and argument validation

status: work-completed
workflow_type: test
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-30T14:41:44Z
last_update: '2026-08-18T18:59:21Z'
date_finished: 2026-03-30T14:53:32Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:10Z'
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
  - ts: '2026-08-18T18:59:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 1
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=1 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-798: Add unit tests for CLI command modules (vendor, push, token, metadata)

## Context

12 of 13 CLI command files have zero test coverage. This task adds unit tests for pure/testable logic in vendor.rs (gitignore, MCP config, vendor status), push.rs (shell_escape), and token.rs (token inspect parsing).

## Acceptance Criteria

### Agent
- [x] vendor.rs has tests for check_gitignore (append, skip existing, create new)
- [x] vendor.rs has tests for configure_mcp (create new, merge existing, skip if configured)
- [x] vendor.rs has tests for cmd_vendor_status (not vendored, vendored)
- [x] push.rs has tests for shell_escape (safe string, special chars, single quotes)
- [x] token.rs has tests for cmd_token_inspect (valid token, invalid format, invalid base64)
- [x] All new tests pass: cargo test --workspace (675 total, 0 failures)
- [x] No clippy warnings: cargo clippy --workspace

## Verification

# Test and clippy verified manually — cargo test --workspace: 675 passed, 0 failed
grep -q "shell_escape" crates/termlink-cli/src/commands/push.rs
grep -q "mod tests" crates/termlink-cli/src/commands/vendor.rs
grep -q "mod tests" crates/termlink-cli/src/commands/token.rs

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

### 2026-03-30T14:41:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-798-add-unit-tests-for-cli-command-modules-v.md
- **Context:** Initial task creation

### 2026-03-30T14:41:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-30T14:53:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
