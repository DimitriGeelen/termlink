---
id: T-1797
name: "Repair broken bin-test compilation — cmd_fleet_reauth test calls stale 2-arg arity"
description: >
  Repair broken bin-test compilation — cmd_fleet_reauth test calls stale 2-arg arity

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-22T07:01:30Z
last_update: 2026-05-22T07:04:27Z
date_finished: 2026-05-22T07:04:27Z
---

# T-1797: Repair broken bin-test compilation — cmd_fleet_reauth test calls stale 2-arg arity

## Context

Found while trying to run the T-1795 regression test: `cargo test -p termlink --bins` fails to compile because 7 test call sites for `cmd_fleet_reauth` still use the 2-arg signature, but T-1727 (commit 9d58391dd) added a 3rd `json: bool` parameter without updating the tests. The whole bin-test target is uncompilable, so NO tests run.

## Acceptance Criteria

### Agent
- [x] All 7 `cmd_fleet_reauth(...)` test call sites in `remote.rs` pass the `json` arg (`false` — tests assert on the returned `Result`, not JSON-mode process exit)
- [x] `cargo test -p termlink --bins --no-run` compiles the test target (no E0061 arity errors) — this is the deliverable
- [x] Note: full-suite RUN completion is blocked by a SEPARATE pre-existing bug (`cli::cli_tests::event_watch_without_hub_accepts_targets` stack-overflows / SIGABRT) — filed as T-1798, out of scope here

## Verification

cargo test -p termlink --bins --no-run 2>&1 | tail -3

## RCA

**Symptom:** `cargo test -p termlink --bins` aborts with 7× E0061 ("this
function takes 3 arguments but 2 arguments were supplied") at
`cmd_fleet_reauth` test call sites in remote.rs. No tests in the bin run.

**Root cause:** T-1727 (commit 9d58391dd, "fleet reauth --json") added a 3rd
`json: bool` parameter to `cmd_fleet_reauth` and updated the production call
site, but left the 7 in-module `#[test]` call sites at the old 2-arg arity.

**Why structurally allowed:** the change compiled and shipped because
`cargo build` does NOT compile `#[cfg(test)]` modules — only `cargo test`
does. Nothing in the normal flow (build, the verification gates on recent
tasks) ran `cargo test` for this crate, so the broken test target sat
undetected. This is the L-291 / "broken DLL on master" class: a build that
passes while the test target is uncompilable. Discovered only because T-1795
needed to add a regression test and tried to run the suite.

**Prevention:** (1) immediate — fix the 7 arities. (2) systemic — `cargo test`
(or at least `cargo test --no-run`) must be part of the pre-merge / task
verification gate for Rust crates so a signature change that breaks tests
fails fast. Captured as a learning: `cargo build` green ≠ tests compile;
gate on `cargo test --no-run`. Consider a follow-up to wire this into the
project's verification convention.

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

### 2026-05-22T07:01:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1797-repair-broken-bin-test-compilation--cmdf.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d98f28ec
- **Timestamp:** 2026-05-22T07:04:40Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-22T07:04:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
