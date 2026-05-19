---
id: T-1712
name: "termlink_doctor MCP — add strict flag (warn→fail verdict, G-057 punch-list #3)"
description: >
  termlink_doctor MCP — add strict flag (warn→fail verdict, G-057 punch-list #3)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-19T12:20:49Z
last_update: 2026-05-19T12:20:49Z
date_finished: null
---

# T-1712: termlink_doctor MCP — add strict flag (warn→fail verdict, G-057 punch-list #3)

## Context

CLI `termlink doctor --strict` promotes warnings to failures in the exit-code
verdict (`exit(1)` when `fail_count > 0 || (strict && warn_count > 0)`). The
MCP `termlink_doctor` has no params struct at all and returns
`{checks, summary}` without an `ok` rollup — agents have to walk the summary
themselves and apply their own threshold logic.

Value: an LLM agent gating "is the environment healthy enough to proceed?"
needs a single boolean. With strict=true the rollup becomes
`ok = fail==0 && warn==0`; without strict it's `ok = fail==0`. Matches CLI
exit-code semantics and gives agents a one-field check.

G-057 punch-list #3 (final from the diagnostic-verb-family audit's MED-tier
parity gaps; the larger `--fix` flag is deferred — fix performs
non-trivial remediation across multiple subsystems and is its own task).

## Acceptance Criteria

### Agent
- [x] New `DoctorParams` struct gains `strict: Option<bool>` field with rustdoc
- [x] `termlink_doctor` handler signature accepts `Parameters<DoctorParams>` (was no-params)
- [x] Response gains a top-level `ok: bool` field: `ok = fail==0 && !(strict && warn>0)`
- [x] Response gains a `strict: bool` echo so caller can confirm what was applied
- [x] Tool description string mentions `strict` opt-in
- [x] Unit tests: `DoctorParams` deserialization (default + strict=true)
- [x] Unit test for the ok-rollup logic (pure function): fail dominates; strict+warn → not ok; non-strict+warn → ok
- [x] `cargo build -p termlink-mcp` passes (no new warnings)

## Verification

cargo build -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-mcp --lib tests::doctor 2>&1 | tail -3 | grep -qE "test result: ok\..*0 failed"
grep -q "pub strict: Option<bool>" crates/termlink-mcp/src/tools.rs
grep -q "DoctorParams" crates/termlink-mcp/src/tools.rs

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

### 2026-05-19T13:05Z — implementation complete [agent]
- **Code:** crates/termlink-mcp/src/tools.rs
  - New `DoctorParams { strict: Option<bool> }` struct.
  - New pure `doctor_ok_rollup(fail, warn, strict) -> bool` — testable verdict logic.
  - `termlink_doctor` handler signature: was `&self`; now `&self, Parameters(p): Parameters<DoctorParams>`.
  - Response gains top-level `ok` (rollup) and `strict` (echo) fields.
  - Tool description updated to mention strict opt-in + CLI parity.
- **Tests:** 5 new (2 deserialization + 3 rollup-logic). Full mcp test suite 170/0/0.
- **Verification:** `cargo build -p termlink-mcp` clean (only pre-existing line-14111 warning); `cargo test tests::doctor` → 5 passed.

### 2026-05-19T12:20:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1712-termlinkdoctor-mcp--add-strict-flag-warn.md
- **Context:** Initial task creation
