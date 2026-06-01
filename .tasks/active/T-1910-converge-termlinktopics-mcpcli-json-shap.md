---
id: T-1910
name: "Converge termlink_topics MCP/CLI JSON shape (T-1909 first-catch)"
description: >
  MCP returns sessions as object map; CLI returns sessions as array with extra total_sessions field. Choose one shape; update divergent side; un-ignore parity_topics test.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1904, T-1909]
created: 2026-06-01T11:34:45Z
last_update: 2026-06-01T12:43:36Z
date_finished: null
---

# T-1910: Converge termlink_topics MCP/CLI JSON shape (T-1909 first-catch)

## Context

Operator-visible drift: the same logical operation "list event topics
across sessions" returns two different JSON shapes depending on whether
it's invoked via MCP (`termlink_topics`) or CLI (`termlink topics
--json`):

- **MCP**: `{"ok": true, "sessions": {"name": [...topics...]},
  "total_topics": 0}` — sessions is an object map (BTreeMap serialized
  directly).
- **CLI**: `{"ok": true, "sessions": [{"session": "name", "topics":
  [...]}], "total_topics": 0, "total_sessions": N}` — sessions is an
  array of records with extra `total_sessions` field.

Both implementations read the same data (`event.topics` RPC against each
session, collected into `BTreeMap<String, Vec<String>>`) and only
diverge at the JSON serialization step. See
`crates/termlink-mcp/src/tools.rs:9888-9936` (MCP) and
`crates/termlink-cli/src/commands/events.rs:1006-1077` (CLI).

**Convergence direction:** align MCP to CLI shape (array-of-records +
`total_sessions`). The array form preserves BTreeMap-sorted session
ordering for human-readable output, and `total_sessions` is useful
telemetry (operators inspecting fleet activity want to know fleet
size, not just total topic count). MCP clients consuming this tool
should see the same shape a CLI inspection would produce.

T-1909 v0.1's `parity_topics` test caught this and is `#[ignore]`d
pending convergence.

## Acceptance Criteria

### Agent
- [x] `termlink_topics` in `crates/termlink-mcp/src/tools.rs` returns
      `sessions: [{"session": "name", "topics": [...]}]` (array of records)
      instead of `sessions: {...}` (object map). Both the
      empty-registrations early-return path AND the populated path emit
      the same shape.
- [x] `termlink_topics` adds `total_sessions: N` field to the JSON
      response (count of sessions with at least one topic, matching
      `session_topics.len()` in CLI's code).
- [x] `parity_topics` test in `crates/termlink-mcp/tests/parity.rs` is
      un-ignored. In-source diagnostic comment updated to note
      convergence date + the structural fix.
- [x] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0 with `test result: ok. 4 passed; 0 failed;
      1 ignored` (was: 3 passed; 0 failed; 2 ignored — topics now passes
      parity, ping remains ignored for T-1911).
- [x] No regression of any other parity test (hub_status, version,
      negative_self_test still pass).

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

grep -q "total_sessions" crates/termlink-mcp/src/tools.rs
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 4 passed; 0 failed; 1 ignored"

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

### 2026-06-01T11:34:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1910-converge-termlinktopics-mcpcli-json-shap.md
- **Context:** Initial task creation

### 2026-06-01T12:29:08Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
