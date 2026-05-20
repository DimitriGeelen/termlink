---
id: T-1679
name: "fleet reauth --all-drifted: bulk-heal all drifted profiles declaring bootstrap_from (T-1291 + T-1660 composition)"
description: >
  fleet reauth --all-drifted: bulk-heal all drifted profiles declaring bootstrap_from (T-1291 + T-1660 composition)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T22:09:42Z
last_update: 2026-05-17T22:18:39Z
date_finished: 2026-05-17T22:18:39Z
---

# T-1679: fleet reauth --all-drifted: bulk-heal all drifted profiles declaring bootstrap_from (T-1291 + T-1660 composition)

## Context

`fleet verify` ends with: `Heal drifted hubs: termlink fleet reauth <profile> --bootstrap-from auto`. With N drifted hubs that's N copy-paste invocations. T-1291 already established declarative per-profile `bootstrap_from`, so the operator's intent in the bulk-heal case is unambiguous: "for every profile that has both drifted AND declared bootstrap_from, run the heal." Add `fleet reauth --all-drifted` as that composition.

Implementation reuses `cmd_fleet_verify`'s parallel-probe pattern (lines 5130-5189) for drift detection, then `cmd_fleet_reauth_bootstrap` for the per-profile heal. Profiles drifted-but-without-declared-bootstrap_from are skipped with a hint pointing at Tier-1 (`fleet reauth <profile>` without flag).

## Acceptance Criteria

### Agent
- [x] `fleet reauth --all-drifted` parses; mutex with positional `<profile>` enforced (error if both, error if neither — both verified via smoke)
- [x] Probes profiles in parallel with the 10s timeout (reuses `probe_cert_with_timeout`)
- [x] For each `drift` profile with declared `bootstrap_from`: invokes the bootstrap-heal path; failure on one profile doesn't abort the loop
- [x] For each `drift` profile without declared `bootstrap_from`: skipped with hint
- [x] Prints a summary table: PROFILE | STATUS | HEALED? | NOTE
- [x] Exit code: 0 if all drifted profiles were healed (or no drift); 1 if any drifted profile failed to heal or skipped
- [x] `cargo check --workspace` passes
- [x] Live smoke: `fleet reauth --all-drifted` on the local fleet runs without panic and prints a sensible empty-or-no-drift report — verified, exit 0 with "0 drifted, 0 healed"

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

### 2026-05-17T22:09:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1679-fleet-reauth---all-drifted-bulk-heal-all.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-6d0e40d0
- **Timestamp:** 2026-05-17T22:18:39Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T22:18:39Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
