---
id: T-1204
name: "Composer status footer — show N live / M total panels at bottom (T-236 follow-up)"
description: >
  Composer status footer — show N live / M total panels at bottom (T-236 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-23T15:06:38Z
last_update: 2026-04-23T15:06:38Z
date_finished: null
---

# T-1204: Composer status footer — show N live / M total panels at bottom (T-236 follow-up)

## Context

Small UX polish on the T-236 composer. Currently when a panel closes
(session ended), the panel label shows "[CLOSED]" but there's no global
awareness of how many sessions are still live. Add a 1-row footer at the
bottom of the host terminal showing "N live / M total — Ctrl+C to exit".
Recomputed on every tick, painted below the last panel row.

## Acceptance Criteria

### Agent
- [x] New helper `draw_status_footer(out, term_rows, live_count, total_count)` writes a reverse-video status bar at row `term_rows` (1-based)
- [x] `cmd_mirror_tag` tracks `term_rows` (from the existing `terminal_size()` call, also recomputed on SIGWINCH), and each tick counts live panels via `!p.closed`
- [x] Footer is repainted once per tick (cheap — ≤80 bytes per frame) after all panel paints complete
- [x] Binary builds clean (`cargo build -p termlink` no new warnings)
- [x] All 10 composer tests still pass
- [x] Live smoke test: spawn 2 tagged sessions, run `mirror --tag`, confirm footer text "2 live / 2 total" appears in stdout bytes

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

### 2026-04-23T15:06:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1204-composer-status-footer--show-n-live--m-t.md
- **Context:** Initial task creation
