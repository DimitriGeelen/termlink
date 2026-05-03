---
id: T-1454
name: "fleet doctor --topic-durability reports audit_unsupported on T-1446-bearing hubs"
description: >
  All 4 fleet hubs (.107 0.9.1701, .122/.141/.121 0.9.1702) report 'audit_unsupported (pre-T-1446 hub)' when running termlink fleet doctor --topic-durability — but hub-side handle_hub_bus_state exists in router.rs:920+ and T-1446 was committed at 204ad1d1 (in-tree before all hub binaries were built). Suspected bug: either client-side dispatch returning Err before reaching hub, or hub-side router not registering the method, or version-gate logic in remote.rs:1758-1772 misclassifying the response. Reproduce: termlink fleet doctor --topic-durability. Expected: each hub should return runtime_dir + audit_present + topic-list. Actual: all 4 hubs report audit_unsupported. Discovered 2026-05-03T10:15Z while completing G-051 mitigation.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-05-03T08:17:28Z
last_update: 2026-05-03T08:17:36Z
date_finished: null
---

# T-1454: fleet doctor --topic-durability reports audit_unsupported on T-1446-bearing hubs

## Context

Investigated 2026-05-03T10:18Z: **NOT A BUG.** Version math: T-1446 commit `204ad1d1` corresponds to derived version `0.9.1717` (count of commits from `v0.9.1` tag). Hub binaries running on the fleet are 0.9.1701 (.107) and 0.9.1702 (.122/.141/.121) — all 15-16 commits BEFORE T-1446 landed. The `audit_unsupported (pre-T-1446 hub)` message is the fleet-doctor's correct, accurate verdict.

**Root cause of confusion:** the build.rs version-derivation tags binaries with their commit-count-since-v0.9.1, not with feature flags. So a binary built between 2026-04-30 (when the 0.9.17xx series began) and 2026-05-02 (T-1446 commit) has 0.9.17xx in its version string but lacks T-1446 features. This is correct by design — see `build.rs` for the derivation.

**Resolution:** rebuild + redeploy hub binaries past 0.9.1717 to enable topic-durability. This is captured implicitly under T-1438's bake-cycle: next musl rebuild + fleet-deploy-binary.sh sweep would naturally pick up the new floor.

**Side-finding for documentation/UX:** the fleet-doctor hint says "upgrade to measure" but doesn't tell the operator the minimum version. A `0.9.1717+` qualifier in the hint string would shave one diagnostic step. Captured as Agent AC below.

## Acceptance Criteria

### Agent
- [x] Hypothesis disproved — hub-side dispatch IS correct (router.rs:173 + capabilities list line 1000). Issue is purely binary version, not code.
- [x] Version math confirmed: T-1446 = 0.9.1717; running hubs = 0.9.1701-0.9.1702.
- [x] UX hint updated (remote.rs:1770) to specify ">=0.9.1717" minimum version. Same patch applied to T-1432's legacy_usage hint (">=0.9.1640") for symmetry. `cargo check --bin termlink` clean.

## Verification

cargo check --bin termlink
grep -q ">=0.9.1717" crates/termlink-cli/src/commands/remote.rs
grep -q ">=0.9.1640" crates/termlink-cli/src/commands/remote.rs

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

### 2026-05-03T08:17:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1454-fleet-doctor---topic-durability-reports-.md
- **Context:** Initial task creation
