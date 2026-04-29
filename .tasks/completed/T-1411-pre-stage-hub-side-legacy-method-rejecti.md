---
id: T-1411
name: "Pre-stage hub-side legacy-method rejection guarded by LEGACY_PRIMITIVES_ENABLED const"
description: >
  Pre-stage hub-side legacy-method rejection guarded by LEGACY_PRIMITIVES_ENABLED const

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-04-29T22:08:54Z
last_update: 2026-04-29T22:18:08Z
date_finished: 2026-04-29T22:18:08Z
---

# T-1411: Pre-stage hub-side legacy-method rejection guarded by LEGACY_PRIMITIVES_ENABLED const

## Context

T-1166 cut as currently planned is a multi-step destructive change: flip the capabilities flag, remove router match arms, remove fallback paths in 6 files. Atomicity is poor — partial failure leaves the codebase in a messy state. T-1411 decouples the structural cut from the source-cleanup: introduce a single `LEGACY_PRIMITIVES_ENABLED: bool` const at the top of router.rs that controls both (a) the `features.legacy_primitives` value in `handle_hub_capabilities`, and (b) guarded `if !ENABLED` match arms above the existing legacy method handlers (event.broadcast, inbox.list/status/clear) that return a structured `method-not-found` error citing T-1166 + the migration doc. With the const at `true` (today), behavior is byte-identical. When T-1166 cut is authorized, flipping it to `false` produces the post-retirement behavior in one commit; the actual source-cleanup (deleting handlers + fallback paths) becomes a no-risk follow-up because tests already prove the flag-off behavior.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/src/router.rs` declares `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = true;` near the top
- [x] `handle_hub_capabilities` uses `LEGACY_PRIMITIVES_ENABLED` (a) to set `features.legacy_primitives` value and (b) to filter event.broadcast + inbox.* out of `methods` array when false
- [x] `route()` has guard arms `<METHOD> if !LEGACY_PRIMITIVES_ENABLED => legacy_method_retired_response(id, "...")` ABOVE each of the 4 existing legacy method handlers (event.broadcast, inbox.list, inbox.status, inbox.clear)
- [x] New helper `fn legacy_method_retired_response(id, method) -> RpcResponse` returns structured error code -32601 with message citing T-1166 + the migration doc
- [x] New unit test: `legacy_method_retired_response_shape` asserts code/message format (+ 2 more: `hub_capabilities_flag_value_matches_const`, `is_retired_legacy_method_predicate`)
- [x] All existing hub-lib tests still pass (291 pass — 288 prior + 3 new)
- [x] Release binary built, hub restarted (PID 2574661), capabilities still shows `legacy_primitives:true`, .143 inbox.status still succeeds — flag-on path is byte-identical

## Verification

cargo test -p termlink-hub --lib 2>&1 | grep -qE "test result: ok\. [0-9]+ passed"
grep -q 'pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = true;' crates/termlink-hub/src/router.rs
grep -q 'fn legacy_method_retired_response' crates/termlink-hub/src/router.rs

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

### 2026-04-29T22:08:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1411-pre-stage-hub-side-legacy-method-rejecti.md
- **Context:** Initial task creation

### 2026-04-29T22:18:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
