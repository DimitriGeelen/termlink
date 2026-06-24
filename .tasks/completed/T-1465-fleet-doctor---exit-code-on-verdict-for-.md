---
id: T-1465
name: "fleet doctor --exit-code-on-verdict for cron/CI cut-readiness probing"
description: >
  fleet doctor --exit-code-on-verdict for cron/CI cut-readiness probing

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-04T05:50:53Z
last_update: 2026-05-04T06:01:34Z
date_finished: 2026-05-04T06:01:34Z
---

# T-1465: fleet doctor --exit-code-on-verdict for cron/CI cut-readiness probing

## Context

After T-1462/T-1463/T-1464 the cut-readiness CLI tells the operator
exactly where the fleet stands, what's producing residue, and how the
rate is trending. The piece still missing for hands-off automation is
a deterministic exit code: cron jobs and CI probes need to gate
follow-up actions on the verdict, but today fleet doctor exits 0
whenever any hub connected, regardless of cut-readiness state.

`--exit-code-on-verdict` (only meaningful with `--legacy-usage`) maps
the verdict to an exit code so operators can wire this into cron / CI
without a JSON-parse step:

| Verdict              | Exit code | Cron interpretation |
|----------------------|-----------|---------------------|
| CUT-READY            | 0         | Safe — proceed with cut. |
| CUT-READY-DECAYING   | 0         | Acceptable — residue is historical, no live callers. |
| WAIT                 | 10        | Live caller present — retry later. |
| UNCERTAIN            | 11        | Hub upgrade or audit-window age-out needed. |

Exit codes 10/11 are chosen above the typical 0..9 range used by shells
(rust binaries default to 1 on panic, 101 on rust panic, etc.) but below
the SIGTERM-derived 128+ range, so they're unambiguous.

## Acceptance Criteria

### Agent
- [x] `--exit-code-on-verdict` boolean flag added to `fleet doctor`. Optional. Requires `--legacy-usage` (errors otherwise — verified: exit code 1 with clear message).
- [x] When set, the command exits with the verdict-mapped code AFTER printing all output (verified: human summary appears before shell sees exit code).
- [x] Connectivity failures keep precedence: exit override gated on `total_fail == 0`.
- [x] Pure helper `verdict_to_exit_code(&str) -> i32` extracted; 5 tests cover CUT-READY=0, CUT-READY-DECAYING=0, WAIT=10, UNCERTAIN=11, and unknown→11 forward-compat.
- [x] Smoke: against current fleet (verdict=CUT-READY-DECAYING) — exit code 0 confirmed.
- [x] Helper coverage via cargo test (5/5 ok), supersedes the unviable WAIT-synthesis idea.
- [x] T-1166 migration doc gains a "Cron/CI integration" subsection with verdict→exit-code table and canonical case statement.
- [x] `cargo build --release -p termlink` succeeds.

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

cargo test -p termlink --bin termlink verdict_to_exit_code
cargo build --release -p termlink
grep -q "exit_code_on_verdict" crates/termlink-cli/src/cli.rs
grep -q "verdict_to_exit_code" crates/termlink-cli/src/commands/remote.rs
grep -q "Cron/CI integration" docs/migrations/T-1166-retire-legacy-primitives.md
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-04T05:50:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1465-fleet-doctor---exit-code-on-verdict-for-.md
- **Context:** Initial task creation

### 2026-05-04T06:01:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
