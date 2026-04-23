---
id: T-1201
name: "T-1200 follow-up — dirty-cell diff render + unicode width for mirror grid"
description: >
  T-1200 follow-up — dirty-cell diff render + unicode width for mirror grid

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/mirror_grid.rs, crates/termlink-cli/src/commands/pty.rs]
related_tasks: []
created: 2026-04-23T14:01:10Z
last_update: 2026-04-23T14:04:22Z
date_finished: 2026-04-23T14:04:22Z
---

# T-1201: T-1200 follow-up — dirty-cell diff render + unicode width for mirror grid

## Context

Follow-up to T-1200. Two remaining T-1191 items: (a) dirty-cell diff render (current `render_full` repaints the entire grid per frame), and (b) consume the already-pulled `unicode-width` dep so wide chars (CJK, emoji) advance the cursor by 2 cells. The `unicode-width` dep is currently unused and will surface as a warning once anyone turns on `cargo build -- --warnings`.

## Acceptance Criteria

### Agent
- [x] `Grid` tracks a `prev_cells: Vec<Cell>` snapshot of the last rendered state
- [x] New `render_diff(&mut self, out) -> io::Result<()>` emits only changed cells — grouped into CUP + run of cells per dirty row segment
- [x] First call (prev buffer empty) falls back to `render_full` for a clean initial paint
- [x] `mirror_loop_grid` calls `render_diff` instead of `render_full` on each frame
- [x] Wide characters (unicode-width == 2) advance cursor by 2 cells; the next cell is marked as a continuation and skipped on render
- [x] `cargo test -p termlink --bin termlink mirror_grid` shows ≥11 passing tests — **11/11 passing** (9 existing + 2 new: `render_diff_emits_only_changes`, `wide_char_advances_cursor_by_two`)
- [x] `cargo build -p termlink` clean, no warnings about unused `unicode-width` (fixed `unused_assignments` warning in same pass)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-23T14:01:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1201-t-1200-follow-up--dirty-cell-diff-render.md
- **Context:** Initial task creation

### 2026-04-23T14:04:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
