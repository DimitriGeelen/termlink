---
id: T-236
name: "Multi-session mirror — termlink mirror --tag with TUI grid"
description: >
  Multi-session mirror — termlink mirror --tag with TUI grid

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/mirror_grid_composer.rs, crates/termlink-cli/src/commands/mirror_grid.rs, crates/termlink-cli/src/commands/mod.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-23T09:09:59Z
last_update: 2026-04-23T14:24:01Z
date_finished: 2026-04-23T14:24:01Z
---

# T-236: Multi-session mirror — termlink mirror --tag with TUI grid

## Context

Follow-on to T-235 GO (vte+grid single-session mirror, landed via T-1199/1200/1201).
Scope: compose N sessions matching a tag into a single TUI grid view. Reuses the
per-session `Grid` primitive; adds a layout planner + composite renderer + parallel
reader tasks, gated behind `termlink mirror --tag <TAG>`.

Research artefact: `docs/reports/T-235-terminal-output-rendering.md`.

## Acceptance Criteria

### Agent
- [x] `termlink mirror --tag <TAG>` CLI surface exists (option added to both `mirror` and `pty mirror` enum variants, mutually exclusive with positional target)
- [x] Tag discovery uses `manager::list_sessions` + existing tag filter; errors cleanly when 0 sessions match
- [x] New module `crates/termlink-cli/src/commands/mirror_grid_composer.rs` contains: layout planner (N → rows × cols grid), `PanelLayout { row, col, cols, rows, label }`, composite dispatch loop
- [x] `Grid::render_diff_at(offset_row, offset_col, out)` + `Grid::render_full_at(...)` added to `mirror_grid.rs`; existing `render_diff`/`render_full` delegate to the `_at` variants with (0,0) offsets OR remain and `_at` siblings coexist — existing 11 tests must still pass
- [x] Layout planner unit tests: `compute_layout(1) == 1×1`, `compute_layout(4) == 2×2`, `compute_layout(6) == 3×2`, `compute_layout(9) == 3×3`
- [x] At least 2 additional tests: `render_diff_at_applies_offset`, `layout_divides_terminal_without_overlap`
- [x] Binary compiles clean on `cargo build -p termlink` (no new warnings introduced in composer or mirror_grid)
- [x] All existing tests still green: `cargo test -p termlink --bin termlink mirror_grid` passes 11/11
- [x] New tests green: composer tests pass

## Verification

cargo build -p termlink 2>&1 | grep -E "^(error|warning: unused)" && exit 1 || echo "build OK"
cargo test -p termlink --bin termlink mirror_grid 2>&1 | tail -5 | grep -q "test result: ok"
cargo test -p termlink --bin termlink mirror_grid_composer 2>&1 | tail -5 | grep -q "test result: ok"

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

### 2026-03-23T09:09:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-236-multi-session-mirror--termlink-mirror---.md
- **Context:** Initial task creation

### 2026-03-23T09:10:09Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next
- **Change:** status: started-work → captured (auto-sync)

### 2026-04-23T14:17:28Z — status-update [task-update-agent]
- **Change:** horizon: next → now

### 2026-04-23T14:17:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-23T14:24:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
