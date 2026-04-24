---
id: T-1221
name: "Fix 4 pre-existing clippy errors in termlink-cli mirror_grid"
description: >
  Workspace clippy has 4 errors in mirror_grid.rs (279, 289, 564) and mirror_grid_composer.rs (222) — 3× collapsible_if + 1× manual_range_patterns. Surfaced by cargo clippy --workspace --tests -- -D warnings. Blocks full-workspace clippy verification on every Rust-touching task (G-015 adjacency). Safe fixes per clippy's own help output.

status: work-completed
workflow_type: refactor
owner: agent
horizon: now
tags: [clippy, tech-debt, termlink-cli]
components: []
related_tasks: []
created: 2026-04-24T15:23:16Z
last_update: 2026-04-24T15:27:30Z
date_finished: 2026-04-24T15:27:30Z
---

# T-1221: Fix 4 pre-existing clippy errors in termlink-cli mirror_grid

## Context

4 clippy errors block `cargo clippy --workspace --tests -- -D warnings`, surfaced while verifying T-1163. Surfaces:
- `crates/termlink-cli/src/commands/mirror_grid.rs:279` — collapsible_if (SGR 38 parser)
- `crates/termlink-cli/src/commands/mirror_grid.rs:289` — collapsible_if (SGR 48 parser)
- `crates/termlink-cli/src/commands/mirror_grid.rs:564` — manual_range_patterns (`0x0A | 0x0B | 0x0C` → `0x0A..=0x0C`)
- `crates/termlink-cli/src/commands/mirror_grid_composer.rs:222` — collapsible_if (scrollback output)

All four are mechanical lint rewrites — no behavior change. clippy's own help output shows the exact fix for each.

## Acceptance Criteria

### Agent
- [x] `mirror_grid.rs:279` SGR 38 nested `if` collapsed into single `if` with `&&` chain. Semantically identical — Rust `&&` short-circuits just like nested `if`.
- [x] `mirror_grid.rs:289` SGR 48 nested `if` collapsed (same pattern as 279).
- [x] `mirror_grid.rs:564` `0x0A | 0x0B | 0x0C` replaced with `0x0A..=0x0C` match arm. All three bytes still match (LF, VT, FF).
- [x] `mirror_grid_composer.rs:222` nested `if let Some(output) / if !output.is_empty()` collapsed into single `if let Some(output) = … && !output.is_empty()`.
- [x] `crates/termlink-mcp/tests/mcp_integration.rs:309` `.expect(&format!(...))` replaced with `.unwrap_or_else(|| panic!(...))` — surfaces under `-D warnings` as `clippy::expect_fun_call`. Found while verifying scope; same class (mechanical lint fix, no behavior change).
- [x] `cargo clippy --workspace --tests -- -D warnings` exits 0 — full workspace clean. Verified 2026-04-24.
- [x] `cargo test -p termlink` — no regressions (no mirror_grid-specific unit tests exist; module covered by integration tests, 4 ignored pre-existing). All 4 fixes are semantics-preserving per clippy's own help text.

## Verification

cargo clippy --workspace --tests -- -D warnings
grep -q "0x0A..=0x0C" crates/termlink-cli/src/commands/mirror_grid.rs
grep -q "unwrap_or_else" crates/termlink-mcp/tests/mcp_integration.rs

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

### 2026-04-24T15:23:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1221-fix-4-pre-existing-clippy-errors-in-term.md
- **Context:** Initial task creation

### 2026-04-24T15:23:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-24T15:27:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
