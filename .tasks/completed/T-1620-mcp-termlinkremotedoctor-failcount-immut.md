---
id: T-1620
name: "MCP termlink_remote_doctor fail_count immutable — summary.fail always 0"
description: >
  MCP termlink_remote_doctor fail_count immutable — summary.fail always 0

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-06T12:13:11Z
last_update: 2026-05-06T12:16:31Z
date_finished: 2026-05-06T12:16:31Z
---

# T-1620: MCP termlink_remote_doctor fail_count immutable — summary.fail always 0

## Context

In `crates/termlink-mcp/src/tools.rs::termlink_remote_doctor` (line ~6020), `fail_count` is declared as `let fail_count: u32 = 0;` (immutable). All sibling counters (`pass_count`, `warn_count`) are `let mut`. The summary block reports `"ok": fail_count == 0` (always true) and `"summary": {"fail": fail_count}` (always 0) regardless of probe outcomes. CLI sibling `commands/infrastructure.rs::cmd_doctor` has all 3 counters as `mut` via a macro. PL-152 (aggregation-counter regression rule) classifies this exactly: a counter that says "0" because the increment site is missing or unreachable.

Additionally, the inbox-probe path at line ~6117 currently classifies "both modern channel.list AND legacy inbox.status RPCs failed" as `warn`, but that's structurally a `fail` — the doctor literally cannot probe the inbox.

## Acceptance Criteria

### Agent
- [x] `fail_count` declared `let mut` (matches `pass_count`/`warn_count` siblings). Evidence: grep "let mut fail_count: u32 = 0;" matches at line ~6024.
- [x] Inbox-probe Err branch (both paths failed) increments `fail_count` and classifies as `"status": "fail"` instead of `warn`. Evidence: awk-grep across the Err(msg) block confirms `"status": "fail"` is present.
- [x] `cargo build -p termlink-mcp` succeeds (strict-clippy out-of-scope: pre-existing tools.rs warnings predate this task). Evidence: build output ends "Finished `dev` profile" with only 1 pre-existing unused_assignments warning at line 12822.
- [x] grep verifies `let mut fail_count: u32 = 0;` present in the function. Evidence: above.

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

# Build must succeed (not strict-clippy: pre-existing warnings in tools.rs from
# unrelated code predate T-1620 and are out-of-scope for this fix).
cargo build -p termlink-mcp 2>&1 | tail -5 | grep -q "^error" && exit 1; exit 0
# Confirm the mutable declaration is actually present (regression sentinel).
grep -q "let mut fail_count: u32 = 0;" crates/termlink-mcp/src/tools.rs
# Confirm the inbox-Err branch now classifies as fail (not warn).
awk '/Err\(msg\) => \{/,/checks\.push.*"check": "inbox"/' crates/termlink-mcp/src/tools.rs | grep -q '"status": "fail"'

## RCA

**Symptom:** `termlink_remote_doctor` MCP tool always returns `"ok": true` and `"summary": {"fail": 0}` even when probes fail. Operators / agent automation cannot trust the structured summary to detect doctor failures.

**Root cause:** `let fail_count: u32 = 0;` (immutable) at line ~6020 of `tools.rs`. Sibling counters (`pass_count`, `warn_count`) are `let mut`. Compiler accepts the asymmetric form because no path tries to mutate it. The `"ok": fail_count == 0` predicate is therefore a tautology.

**Why structurally allowed:** Rust permits an unused-mutability *omission* silently — there is no warning for "this binding is read by code that compares against zero, but no code path increments it". `clippy::needless_pass_by_value` and similar lints catch the inverse (over-mutability), not under-mutability. The CLI sibling uses a macro that bakes mutability into the call site; the MCP tool inlined the counter without the macro and dropped the mut. Identical to PL-152 (T-1615): aggregation counter regression — when a counter declaration is split from its increment sites, missed siblings silently undercount.

**Prevention:** PL-152 lessons applied at code-review time — when seeing `pass`/`warn`/`fail` counter trios, eyeball-verify all three are `mut` and have at least one increment site each. Verification step in this task explicitly greps for `let mut fail_count` so a regression on the same line fails the gate.
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

## Updates

### 2026-05-06T12:13:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1620-mcp-termlinkremotedoctor-failcount-immut.md
- **Context:** Initial task creation

### 2026-05-06T12:16:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
