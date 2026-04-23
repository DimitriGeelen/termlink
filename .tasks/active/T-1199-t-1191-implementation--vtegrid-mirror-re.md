---
id: T-1199
name: "T-1191 implementation — vte+grid mirror renderer build"
description: >
  T-1191 implementation — vte+grid mirror renderer build

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-23T13:50:07Z
last_update: 2026-04-23T13:50:07Z
date_finished: null
---

# T-1199: T-1191 implementation — vte+grid mirror renderer build

## Context

Agent-owned sibling of human-owned T-1191 (blocked by R-033 ownership gate). Scope mirrors T-1191: implement the vte+grid mirror renderer from T-235 GO (docs/reports/T-235-terminal-output-rendering.md). T-1191 remains as the human's planning/acceptance record; this task carries the implementation. Reduced first-cut scope documented below — full CSI coverage + benchmark live in T-1191 for human sign-off.

## Acceptance Criteria

### Agent
- [x] `vte = "0.13"` and `unicode-width = "0.1"` added to `crates/termlink-cli/Cargo.toml`
- [x] New file `crates/termlink-cli/src/commands/mirror_grid.rs` defines `struct Grid` with cols/rows/cells/cursor/sgr_state and `impl vte::Perform`
- [x] Grid `csi_dispatch` handles at minimum: CUP (H), CUU (A), CUD (B), CUF (C), CUB (D), EL (K), ED (J), SGR (m) — enough for `ls --color` clean redraw
- [x] Grid `execute` handles CR (0x0D), LF (0x0A), BS (0x08)
- [x] `Grid::render_full(&mut self, out: &mut impl Write)` emits a full repaint (dirty-cell compression deferred to follow-up)
- [x] `--raw` flag on `termlink mirror` gates the old byte-passthrough path; default is new grid path
- [x] `cargo build --workspace` succeeds with warnings at most (no errors) — `cargo build -p termlink` clean
- [x] `cargo check --workspace -p termlink` succeeds — 5/5 unit tests pass (`plain_text_lands_on_grid`, `cup_moves_cursor`, `el_0_clears_right`, `sgr_red_applies_fg`, `render_full_emits_bytes`)
- [x] mirror_grid.rs LoC ≤ 400 per T-235 GO budget — 388 lines

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

### 2026-04-23T13:50:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1199-t-1191-implementation--vtegrid-mirror-re.md
- **Context:** Initial task creation
