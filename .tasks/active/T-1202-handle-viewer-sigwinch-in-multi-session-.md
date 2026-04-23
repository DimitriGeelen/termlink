---
id: T-1202
name: "Handle viewer SIGWINCH in multi-session mirror composer (T-236 follow-up)"
description: >
  Handle viewer SIGWINCH in multi-session mirror composer (T-236 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-23T14:33:36Z
last_update: 2026-04-23T14:33:36Z
date_finished: null
---

# T-1202: Handle viewer SIGWINCH in multi-session mirror composer (T-236 follow-up)

## Context

T-236 shipped the multi-session composer (`termlink mirror --tag`). Gap: the
composer reads `terminal_size()` once at startup; if the viewer resizes
their terminal mid-session, panels stay at the initial layout and render
into the wrong rectangles. Fix: register a SIGWINCH handler, recompute
layout on signal, resize each panel's `Grid` to its new viewport, force
a full repaint.

## Acceptance Criteria

### Agent
- [x] `tokio::signal::unix::signal(SignalKind::window_change())` is registered alongside the existing SIGINT handler in `cmd_mirror_tag`
- [x] On SIGWINCH: re-fetch `terminal_size()`, recompute `compute_layout(n, cols, rows)`, call `Grid::resize` on each panel to the new `(grid_cols, grid_rows)`, mark each panel dirty, and repaint labels + full grids
- [x] New test `layout_recomputes_on_size_change`: runs `compute_layout(4, 80, 24)` and `compute_layout(4, 120, 40)`, asserts different dimensions
- [x] Binary builds clean: `cargo build -p termlink` emits no new warnings
- [x] All composer tests still pass (9 pre-existing + 1 new = 10)
- [x] Live smoke test: spawn 2 tagged sessions, start `termlink mirror --tag <tag>` under `stty cols X rows Y` changed mid-stream, confirm no crash/hang

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

cargo build -p termlink 2>&1 | grep -E "^(error|warning: unused)" && exit 1 || echo "build OK"
cargo test -p termlink --bin termlink mirror_grid_composer 2>&1 | tail -5 | grep -q "test result: ok"



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

### 2026-04-23T14:33:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1202-handle-viewer-sigwinch-in-multi-session-.md
- **Context:** Initial task creation
