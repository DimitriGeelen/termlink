---
id: T-1261
name: "T-1191 deferred follow-up: integration tests + bench scaffold for vte mirror grid"
description: >
  Capture deferred work from T-1191: golden-input integration tests for vim/htop/ls --color/less, and cargo bench scaffold + corpus for mirror_render. Both deferred at completion 2026-04-23 because they require captured byte-stream corpora and bench infra that don't exist yet.

status: captured
workflow_type: test
owner: agent
horizon: later
tags: []
components: []
related_tasks: [T-1191, T-1199, T-1200, T-1201, T-235]
created: 2026-04-25T18:16:00Z
last_update: 2026-04-25T18:16:07Z
date_finished: null
---

# T-1261: T-1191 deferred follow-up: integration tests + bench scaffold for vte mirror grid

## Context

T-1191 (vte mirror grid renderer, completed 2026-04-23) deferred two ACs at
completion:

1. Integration tests using captured byte-stream corpora from real apps
   (vim, htop, ls --color, less) — depends on a fixture-capture pipeline
   that does not exist yet. Unit tests (12 in `mirror_grid.rs`) cover the
   dispatch table directly.
2. `cargo bench --bench mirror_render` performance assertion (<16 ms/frame
   on 1 MB of vim traffic) — depends on `cargo bench` scaffold + corpus.

Parked at horizon=later because the renderer ships fine without these and
the corpus-capture infra is its own project.

## Acceptance Criteria

### Agent
- [ ] Capture pipeline: a script (or CLI subcommand) that pipes a real app
      through `termlink mirror --raw`, records the byte stream, and writes
      a JSON fixture to `crates/termlink-cli/tests/fixtures/mirror_grid/`.
- [ ] At least 4 fixtures committed (vim opening /etc/passwd, htop 1s tick,
      ls --color, less paging).
- [ ] Integration tests in `crates/termlink-cli/tests/mirror_grid.rs` load
      each fixture, run it through `Grid` via `vte::Parser::advance`, and
      assert the rendered byte stream matches a golden snapshot.
- [ ] `cargo bench --bench mirror_render` scaffold added (criterion or std
      bench) — feeds the largest fixture through `Grid` and asserts median
      <16 ms/frame.
- [ ] T-1191 task file: deferred AC entries updated to `[x] Shipped via T-1261`.

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

### 2026-04-25T18:16:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1261-t-1191-deferred-follow-up-integration-te.md
- **Context:** Initial task creation

### 2026-04-25T18:16:07Z — status-update [task-update-agent]
- **Change:** horizon: later → later
