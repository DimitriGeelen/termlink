---
id: T-1413
name: "Test-cover the LEGACY_PRIMITIVES_ENABLED=false path via cargo feature flag (T-1411 follow-up)"
description: >
  Test-cover the LEGACY_PRIMITIVES_ENABLED=false path via cargo feature flag (T-1411 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T22:31:46Z
last_update: 2026-04-29T22:31:46Z
date_finished: null
---

# T-1413: Test-cover the LEGACY_PRIMITIVES_ENABLED=false path via cargo feature flag (T-1411 follow-up)

## Context

T-1411 made the cut a one-character const flip. But the flag-OFF path is currently unverified — the existing tests only run with the const at `true`. Convert the const to a Cargo-feature-driven value (`legacy_primitives_disabled`) and add tests guarded by that feature so CI can verify the OFF path. The operator running the cut then has CI evidence the cut works before deploying. Default behavior unchanged: feature is off → const is `true` → byte-identical to today.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/Cargo.toml` declares `[features]` table with `legacy_primitives_disabled = []` (empty deps; pure cfg switch)
- [x] `crates/termlink-hub/src/router.rs` const becomes `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = !cfg!(feature = "legacy_primitives_disabled");` — default-feature-off keeps current behavior
- [x] `cargo test -p termlink-hub --lib` (default features): 291 PASS
- [x] `cargo test -p termlink-hub --lib --features legacy_primitives_disabled` runs and passes (293 PASS); T-1405 + T-1215 + tcp_broadcast tests gated to default-only
- [x] New `cut_path` test module guarded by `#[cfg(feature = "legacy_primitives_disabled")]`: const-is-false invariant, capabilities-flag-off, methods-array-excludes-retired-names, route-returns-method-not-found-for-event_broadcast, route-returns-method-not-found-for-each-inbox-method (5 tests)
- [x] Migration doc updated: new step 3 in Operator Cut Procedure runs `cargo test --features legacy_primitives_disabled` as pre-flip CI verification; References list extended with T-1413

## Verification

cargo test -p termlink-hub --lib 2>&1 | grep -qE "test result: ok\. [0-9]+ passed"
cargo test -p termlink-hub --lib --features legacy_primitives_disabled 2>&1 | grep -qE "test result: ok\. [0-9]+ passed"
grep -q 'legacy_primitives_disabled = \[\]' crates/termlink-hub/Cargo.toml
grep -q 'cfg!(feature = "legacy_primitives_disabled")' crates/termlink-hub/src/router.rs

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

### 2026-04-29T22:31:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1413-test-cover-the-legacyprimitivesenabledfa.md
- **Context:** Initial task creation
