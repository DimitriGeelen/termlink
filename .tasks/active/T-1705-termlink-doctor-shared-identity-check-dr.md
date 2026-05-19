---
id: T-1705
name: "termlink doctor: shared-identity check (drives T-1700 adoption from diagnostic path)"
description: >
  termlink doctor: shared-identity check (drives T-1700 adoption from diagnostic path)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-19T06:20:17Z
last_update: 2026-05-19T06:20:17Z
date_finished: null
---

# T-1705: termlink doctor: shared-identity check (drives T-1700 adoption from diagnostic path)

## Context

Companion to T-1704: `termlink whoami` now hints at shared identity
when the local session shares its FP with peers. `termlink doctor` —
the operator's go-to diagnostic — should surface the same condition.
Doctor already enumerates live sessions (check #3 at line 177 of
`crates/termlink-cli/src/commands/infrastructure.rs`); adding a
group-by-fingerprint pass and emitting a `warn` when 2+ sessions share
an FP piggybacks on the existing session list. Names `--identity-key`
+ T-1700 so the operator has a copy-pasteable next step.

## Acceptance Criteria

### Agent
- [x] `cmd_doctor` in `crates/termlink-cli/src/commands/infrastructure.rs` groups live sessions by `metadata.identity_fingerprint` and emits a `warn` named `identity` when 2+ sessions share any FP — implemented at section "7b. Identity attribution"
- [x] Message lists the shared FP short (8 chars) and the count of sessions sharing it; names `--identity-key (T-1700)` so the operator has a copy-pasteable next step — live output: `12 sessions share 1 identity FP [d1993c2c×12] — pass --identity-key at register for per-agent identity (T-1700)`
- [x] Emits `pass` named `identity` when no sessions share an FP, OR when there are <2 sessions with FPs at all — `pass` branch with `no shared identities (N session(s) with FP)` message
- [x] Sessions without `identity_fingerprint` (pre-T-1436) do not contribute to any FP group and do not silently pass-or-fail — `group_sessions_by_identity` excludes them via `if let Some(fp)` filter, asserted by test `group_sessions_by_identity_absent_fp_excluded`
- [x] Unit test for the pure grouping helper `group_sessions_by_identity`: (a) all-unique returns no shared groups, (b) host-shared returns one group with N members, (c) absent FP excluded — 3 tests added
- [x] `cargo test -p termlink --bins commands::infrastructure` passes (existing + new tests) — 23/23 ok
- [x] Live smoke on .107: `./target/release/termlink doctor` shows the new `identity` warn line — verified above

### Human
<!-- All ACs agent-verifiable. -->

## Verification

cargo test -p termlink --bins commands::infrastructure 2>&1 | tail -5 | grep -E "test result: ok"
./target/release/termlink doctor 2>&1 | grep -E "identity:.*sessions share" 

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

### 2026-05-19T06:20:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1705-termlink-doctor-shared-identity-check-dr.md
- **Context:** Initial task creation
