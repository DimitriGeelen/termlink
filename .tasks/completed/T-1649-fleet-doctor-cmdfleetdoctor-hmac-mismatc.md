---
id: T-1649
name: "fleet doctor cmd_fleet_doctor HMAC mismatch diagnosis use heal_bootstrap_hint (T-1648 parity)"
description: >
  fleet doctor cmd_fleet_doctor HMAC mismatch diagnosis use heal_bootstrap_hint (T-1648 parity)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [auth, fleet, ergonomic]
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: [T-1648, T-1291, T-1054, T-1055]
created: 2026-05-16T21:40:21Z
last_update: 2026-05-16T21:53:14Z
date_finished: 2026-05-16T21:53:14Z
---

# T-1649: fleet doctor cmd_fleet_doctor HMAC mismatch diagnosis use heal_bootstrap_hint (T-1648 parity)

## Context

T-1648 ergonomic-ized heal hints in `cmd_fleet_status` (the default `fleet`/`fleet status` action). The deeper layered `cmd_fleet_doctor` (the `fleet doctor` subcommand) carries the same regression at remote.rs:3531 — the `diagnosis` for HMAC mismatch is `Option<&'static str>` with a hardcoded `<profile>` placeholder. Operators running `fleet doctor` still see imperative SSH form even when their profile declares `bootstrap_from`. Same fix, slightly larger blast radius because diagnosis type must widen from `&'static str` to `String` to carry per-hub formatted output.

## Acceptance Criteria

### Agent
- [x] `cmd_fleet_doctor` diagnosis type widens from `Option<&'static str>` to `Option<String>`; all 7 existing assignments compile (added `.to_string()` to the 6 static literals)
- [x] HMAC mismatch diagnosis at remote.rs:3531 formats per-hub: substitutes profile name and uses `heal_bootstrap_hint(entry, &entry.address)` for the heal-source recommendation, factored as `format_hmac_mismatch_diagnosis(name, entry)`
- [x] One unit test asserts the formatted diagnosis includes profile name + `--bootstrap-from auto` when profile declares `bootstrap_from`
- [x] One unit test asserts the formatted diagnosis includes `--bootstrap-from ssh:<host>` + tip when profile lacks declaration
- [x] `cargo build --release -p termlink` succeeds (6m12s)
- [x] `cargo test --release -p termlink --bin termlink fleet_doctor_hmac_diagnosis` passes (2/2 ok)

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
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

cargo test --release -p termlink --bin termlink fleet_doctor_hmac_diagnosis 2>&1 | tee /tmp/t1649-test.log | tail -10
grep -E "test result: ok|0 failed" /tmp/t1649-test.log

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

### 2026-05-16T21:40:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1649-fleet-doctor-cmdfleetdoctor-hmac-mismatc.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-80d41889
- **Timestamp:** 2026-05-16T21:56:51Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-16T21:53:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
