---
id: T-1901
name: "Test reliability — connect_addr_with_timeout_errors_on_unreachable assertion too narrow (rejects 'No route to host' fast-fail)"
description: >
  Test reliability — connect_addr_with_timeout_errors_on_unreachable assertion too narrow (rejects 'No route to host' fast-fail)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-31T19:16:33Z
last_update: 2026-05-31T19:16:33Z
date_finished: 2026-05-31T19:21:02Z
---

# T-1901: Test reliability — connect_addr_with_timeout_errors_on_unreachable assertion too narrow (rejects 'No route to host' fast-fail)

## Context

Discovered during T-1415 AC6 verification sweep this session: `cargo test -p termlink-session --lib` fails with 1 of 326 tests red — `client::tests::connect_addr_with_timeout_errors_on_unreachable` (introduced in T-1677, commit `7866b60d`). The test connects to TEST-NET address 192.0.2.1:9100, expects a fast error, and asserts the error string contains `"timeout"` OR `ErrorKind::TimedOut`. From this host the connect fast-fails with **"No route to host (os error 113)"** — a faster + sharper fast-fail than a timeout, but neither contains `timeout` nor maps to `TimedOut`. Behavior is environment-dependent: hosts with a default route to TEST-NET-1 black-hole would hit the 1s timeout; hosts with no route fast-fail immediately.

Scope: broaden the assertion to accept all "fast-fail" error categories that mean "host is unreachable" (NetworkUnreachable, HostUnreachable, ConnectionRefused, OR the timeout cases). Test's true intent — verify the connect-with-timeout path returns a fast error rather than hanging for 30-60s OS TCP retry — is preserved by the existing `elapsed < 3s` check.

## Acceptance Criteria

### Agent
- [x] `cargo test -p termlink-session --lib client::tests::connect_addr_with_timeout_errors_on_unreachable` passes on this host (from `/opt/termlink` repo). Verified 2026-05-31T19:18Z: `test result: ok. 1 passed; 0 failed`.
- [x] Full `cargo test -p termlink-session --lib` passes (326/326 green). Verified 2026-05-31T19:18Z: `test result: ok. 326 passed; 0 failed; 0 ignored; 0 measured`.
- [x] The test's `elapsed < 3s` fast-error invariant is preserved (not weakened) — the assertion still confirms the connect path doesn't hang for OS TCP retry. Confirmed by reading the diff; lines 420-423 unchanged.
- [x] Assertion accepts at minimum these error categories as valid fast-fail outcomes: `timeout` substring match (preserved), `ErrorKind::TimedOut` (preserved), `ErrorKind::NetworkUnreachable`, `ErrorKind::HostUnreachable`, `ErrorKind::ConnectionRefused`. Verified by reading the patched assertion at `crates/termlink-session/src/client.rs:435-442`.
- [x] No other tests touched; only `crates/termlink-session/src/client.rs` lines 424-446 edited. Verified by `git diff --stat`.

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

cargo test -p termlink-session --lib client::tests::connect_addr_with_timeout_errors_on_unreachable
cargo test -p termlink-session --lib 2>&1 | grep -q "test result: ok"

## RCA

**Symptom:** `cargo test -p termlink-session --lib` fails 1/326 — `client::tests::connect_addr_with_timeout_errors_on_unreachable` panics with `expected timeout-related error, got: No route to host (os error 113)`.

**Root cause (test brittleness):** The test was authored with the assumption that connecting to TEST-NET-1 (192.0.2.1) would time out — true on hosts whose default route silently black-holes TEST-NET. On hosts with no route to TEST-NET (most production / dev machines, including this one), the connect fast-fails with ECONNREFUSED / ENETUNREACH / EHOSTUNREACH (os error 113) which is a *better* outcome than the test expects (sharper fast-fail). The assertion is too narrow.

**Why structurally allowed:** Tests are written against the most common local environment. T-1677 introduced this test from a machine where TEST-NET black-holed; that became the implicit assumption. No CI matrix exercises the "no route" path because CI runners typically also black-hole TEST-NET. Local-dev hosts (like /opt/termlink on 192.168.10.107) are the only place this fails.

**Prevention:** Assertion broadened to accept all `std::io::ErrorKind` variants that mean "host unreachable / fast-fail" — TimedOut (existing), NetworkUnreachable, HostUnreachable, ConnectionRefused. The test's real invariant — `elapsed < 3s` — is preserved (verifies the connect doesn't hang on OS TCP retry). Future similar tests should follow the pattern: assert the *invariant* (fast-fail < N seconds) plus a wide-enough error category match.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-31T19:16:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1901-test-reliability--connectaddrwithtimeout.md
- **Context:** Initial task creation
