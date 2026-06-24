---
id: T-1913
name: "MCP/CLI parity harness v0.2 — expand to channel_* pairs (T-1909 follow-on)"
description: >
  Add 3-5 channel_* parity pairs to crates/termlink-mcp/tests/parity.rs. Highest-trafficked subsystem (53 pairs total in census). v0.1 caught 3 real divergences; v0.2 has high expected-catch yield.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: [T-1904, T-1909]
created: 2026-06-01T12:58:08Z
last_update: 2026-06-01T13:23:35Z
date_finished: 2026-06-01T14:06:27Z
---

# T-1913: MCP/CLI parity harness v0.2 — expand to channel_* pairs (T-1909 follow-on)

## Context

v0.2 of the T-1909 parity harness. v0.1 covered 4 session-control pairs
and caught 3 real divergences (topics shape, version source, ping
transport — converged via T-1910, T-1912; ping deferred to T-1911).

Channel ops are the highest-traffic MCP/CLI subsystem (53 pairs in the
T-1904 census). v0.2 starts with two no-hub-required pairs to avoid
introducing hub-fixture machinery in this slice:

- `termlink_channel_queue_status` / `termlink channel queue-status`
  (reads local offline-queue file; no hub contact)
- `termlink_channel_list` / `termlink channel list` (errors when no
  hub running — tests error-path parity)

If catches surface, file convergence tasks (same pattern as T-1910/12).

## Acceptance Criteria

### Agent
- [x] `parity_channel_queue_status` test exists in
      `crates/termlink-mcp/tests/parity.rs`. Both MCP and CLI invoked
      against an explicit non-existent queue_path; diff-asserts the
      `{queue_path, exists: false, pending: 0}` shape on both sides.
- [x] `parity_channel_list_no_hub` test exists. Both surfaces called
      with no hub running; diff-asserts they produce the same error
      shape (parity covers error paths, not just success paths).
- [x] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0. Test count grows from 5 (v0.1, post
      T-1910/12) to 7 (v0.2). Catches surface as either PASS or
      `#[ignore]`d-with-follow-up — both outcomes are acceptable
      v0.2-completion states.
- [x] If any new catch surfaces, a follow-up task is filed (same
      pattern as T-1910/T-1912) and the test is `#[ignore]`d with a
      diagnostic comment.
- [x] No regression of any existing parity test (hub_status, topics,
      version, negative_self_test still pass).

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

grep -q "parity_channel_queue_status" crates/termlink-mcp/tests/parity.rs
grep -q "parity_channel_list_no_hub" crates/termlink-mcp/tests/parity.rs
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. [4567] passed; 0 failed; [0123] ignored"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-06-01T12:58:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1913-mcpcli-parity-harness-v02--expand-to-cha.md
- **Context:** Initial task creation

### 2026-06-01T12:58:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
