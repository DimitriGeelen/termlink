---
id: T-1647
name: "MCP termlink_event_subscribe hub-level support (T-1645 PL-158 completion)"
description: >
  MCP termlink_event_subscribe hub-level support (T-1645 PL-158 completion)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-16T08:51:41Z
last_update: 2026-05-16T08:56:33Z
date_finished: 2026-05-16T08:56:33Z
---

# T-1647: MCP termlink_event_subscribe hub-level support (T-1645 PL-158 completion)

## Context

PL-158 (T-1645): when a hub event is emitted via `aggregator().inject()` with
`session_id: "hub"`, the canonical CLI/MCP path must reach the hub-aggregator
stream — not the per-session bus. T-1645 closed the CLI half via
`event watch --hub`. The MCP half (`termlink_event_subscribe`) still requires
`target: String` and routes to a session socket, so it cannot observe
aggregator-direct events like `inbox.queued`. This task closes the MCP half.

Single-line change in surface: `EventSubscribeParams.target: String` →
`Option<String>`. Branch in handler: `Some(t)` keeps existing session-socket
path; `None` resolves the hub socket via `infrastructure::resolve_hub_paths()`
and calls `event.subscribe` with no `target` field → routes to
`handle_hub_subscribe` → `aggregator.collect()` (same router branch the
CLI now uses).

## Acceptance Criteria

### Agent
- [x] `EventSubscribeParams.target` becomes `Option<String>` (additive change — existing callers with `target: "..."` keep working) — verified by `event_subscribe_params_defaults` test (passes with `assert_eq!(p.target.as_deref(), Some("s1"))`)
- [x] When `target` is `None`, `termlink_event_subscribe` routes to the hub socket and calls `event.subscribe` with no `target` field — handler branches on `match &p.target { Some(t) => session-socket, None => resolve_hub_paths() }`
- [x] When `target` is `Some(t)`, behavior is unchanged (session-socket path) — same `manager::find_session(t).socket_path()` resolution, same params (since + max_events still applied)
- [x] Updated tool description on `termlink_event_subscribe` documents the hub-level mode — description now reads "Two modes: (1) per-session ... (2) hub-level: omit `target` (or pass null) ..."
- [x] Unit test asserts `termlink_event_subscribe` with `target: None` constructs params — `event_subscribe_params_hub_mode_omits_target` + `event_subscribe_params_hub_mode_null_target` cover both omit-key and explicit-null paths
- [x] `cargo build --release -p termlink-mcp` clean — `Finished release profile [optimized] target(s) in 1m 11s` (pre-existing tools.rs warning unrelated to this change)
- [x] `cargo test --release -p termlink-mcp --lib event_subscribe` — 3 passed; 0 failed

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
#
# Build clean
cd /opt/termlink && cargo build --release -p termlink-mcp 2>&1 | tail -3 | grep -q "Finished"

# MCP lib tests pass (including new event_subscribe_hub_mode_params test)
cd /opt/termlink && cargo test --release -p termlink-mcp --lib event_subscribe 2>&1 | tail -5 | grep -qE "test result: ok"

# Tool description documents hub mode
cd /opt/termlink && grep -q "hub-level\|omit.*hub" crates/termlink-mcp/src/tools.rs

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

### 2026-05-16T08:51:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1647-mcp-termlinkeventsubscribe-hub-level-sup.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-12ffef6c
- **Timestamp:** 2026-05-16T08:58:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-16T08:56:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
