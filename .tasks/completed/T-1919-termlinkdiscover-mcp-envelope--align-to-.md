---
id: T-1919
name: "termlink_discover MCP envelope — align to {ok,sessions} matching CLI"
description: >
  termlink_discover MCP envelope — align to {ok,sessions} matching CLI

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T22:10:42Z
last_update: 2026-06-01T22:10:42Z
date_finished: 2026-06-01T22:33:11Z
---

# T-1919: termlink_discover MCP envelope — align to {ok,sessions} matching CLI

## Context

Same shape-divergence class as T-1910 (topics), T-1912 (version), T-1918
(list_sessions). The MCP `termlink_discover` tool at
`crates/termlink-mcp/src/tools.rs:8195` returns a bare JSON array
`[...]` of session items. The CLI `termlink discover --json` at
`crates/termlink-cli/src/commands/metadata.rs:297-316` returns
`{"ok": true, "sessions": [...]}`. Pipelines consuming both surfaces
need conditional unwrap. Align MCP to the CLI envelope.

## Acceptance Criteria

### Agent
- [x] MCP `termlink_discover` returns `{"ok": true, "sessions": [...]}` envelope
- [x] Parity test `parity_discover` added to `crates/termlink-mcp/tests/parity.rs` covering structural equality with field-ignore-list
- [x] `cargo test -p termlink-mcp --test parity` passes (9 passed, 1 ignored = T-1911)

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

cargo build -p termlink-mcp -p termlink 2>&1 | grep -E "^error" | grep -v "warning" && exit 1 || exit 0

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

### 2026-06-01T22:10:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1919-termlinkdiscover-mcp-envelope--align-to-.md
- **Context:** Initial task creation
