---
id: T-1704
name: "whoami hint when identity is host-shared (drives T-1700 adoption)"
description: >
  whoami hint when identity is host-shared (drives T-1700 adoption)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs]
related_tasks: []
created: 2026-05-18T22:49:36Z
last_update: 2026-05-18T23:00:04Z
date_finished: 2026-05-18T23:00:04Z
---

# T-1704: whoami hint when identity is host-shared (drives T-1700 adoption)

## Context

T-1700 / T-1701 / T-1702 (shipped 2026-05-19) closed PL-166 structurally —
operators can now pass `--identity-key <PATH>` at `termlink register` to
sign envelopes with a per-agent ed25519 key instead of the host default.

The remaining gap is **discoverability**: `termlink whoami` prints
`Identity FP: d1993c2c3ec44c94` with no indication that this fingerprint
is shared with every other agent on the host. On `.107` today, five
co-resident sessions (framework-agent, termlink-agent, cohort-agent,
penelope, this Claude) all report the same FP. An operator has no
trigger to ask "should I switch to per-agent identity?".

This task adds a one-line hint under the `Identity FP:` row of
`termlink whoami` when ≥1 other session on this hub reports the same
identity_fingerprint. JSON output gains an `identity_shared_with`
field. The hint surfaces T-1700 to anyone who already runs whoami —
zero schema changes, zero risk to existing tooling, drives adoption.

## Acceptance Criteria

### Agent
- [x] `print_whoami_card` in `crates/termlink-cli/src/commands/metadata.rs` emits a `↳ shared with N other session(s) on this hub` line directly under `Identity FP:` when N ≥ 1 (counted as sessions other than self with the same `metadata.identity_fingerprint` on the local hub) — live evidence below
- [x] Hint text names `--identity-key` and references T-1700 so the operator has a copy-pasteable next step — text: `see \`termlink register --identity-key <PATH>\` for per-agent identity (T-1700)`
- [x] No hint emitted when N == 0 (single-session host) or when `identity_fingerprint` is absent (pre-T-1436 sessions) — guarded by `if shared_identity_count > 0` and `count_shared_identity` returns 0 when target FP is None
- [x] JSON path (`whoami_card_json`) emits `session.identity_shared_with: <N>` when an `identity_fingerprint` is present — JSON smoke confirms `"identity_shared_with": 11`
- [x] Unit tests cover: (a) zero share = no hint / `identity_shared_with == 0`, (b) shared = hint emitted with correct count, (c) absent FP = no hint and JSON field omitted — 4 new tests added (count_shared_identity_*, whoami_card_json_emits_identity_shared_with_when_fp_present, plus extended legacy-FP-absent assertion)
- [x] `cargo test -p termlink --bins commands::metadata::tests` passes including new tests — 11/11 ok (binary target — termlink-cli has no `[lib]`)
- [x] Live smoke on .107: `./target/release/termlink whoami --session tl-7zlfowtz` shows the hint with N ≥ 1 — observed N=11 on host .107 (12 co-resident sessions total, 11 share the host-default FP `d1993c2c3ec44c94`)

### Human
<!-- All ACs are agent-verifiable; this is a discoverability improvement with no UX requiring human judgment beyond the agent's own smoke. -->

## Verification

cargo test -p termlink --bins commands::metadata::tests 2>&1 | tail -5 | grep -E "test result: ok"
./target/release/termlink whoami --session tl-7zlfowtz 2>&1 | grep -E "shared with [0-9]+ other"

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

### 2026-05-18T22:49:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1704-whoami-hint-when-identity-is-host-shared.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-fd4d0f48
- **Timestamp:** 2026-05-18T23:00:18Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `print_whoami_card` in `crates/termlink-cli/src/commands/metadata.rs` emits a `↳ shared with N other session(s) on this hub` line directly under `Identity FP:` when N ≥ 1 (counted as sessions other th
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/metadata.rs in: `print_whoami_card` in `crates/termlink-cli/src/commands/metadata.rs` emits a `↳ shared with N other session(s) on this hub` line directly under `Iden`

### 2026-05-18T23:00:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
