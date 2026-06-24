---
id: T-1648
name: "fleet doctor — suggest --bootstrap-from auto when profile declares it (T-1291 ergonomic follow-up)"
description: >
  fleet doctor — suggest --bootstrap-from auto when profile declares it (T-1291 ergonomic follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [auth, fleet, ergonomic]
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: [T-1291, T-1054, T-1055, T-1051]
created: 2026-05-16T21:21:59Z
last_update: 2026-05-16T21:34:15Z
date_finished: 2026-05-16T21:34:15Z
---

# T-1648: fleet doctor — suggest --bootstrap-from auto when profile declares it (T-1291 ergonomic follow-up)

## Context

T-1291 shipped the declarative `bootstrap_from` field per profile + `--bootstrap-from auto` as the one-flag heal path. But `fleet status`/`fleet doctor` still hard-code the heal hint to `--bootstrap-from ssh:<host>` for every auth-mismatch and secret-file-missing case — even when the profile already has `bootstrap_from` declared. Operators end up typing the literal SSH form because the tool tells them to, not because they have to. Add a helper that picks `auto` when the profile declares it, and otherwise prints the SSH form plus a one-line nudge to declare `bootstrap_from` for next time.

## Acceptance Criteria

### Agent
- [x] New helper `heal_bootstrap_hint(entry, address)` in `crates/termlink-cli/src/commands/remote.rs` returns `--bootstrap-from auto` when `entry.bootstrap_from.is_some()`, otherwise `--bootstrap-from ssh:<host>` + declarative tip
- [x] `cmd_fleet_status` auth-mismatch hint (line ~2240) and secret-file-missing hint (line ~2286) use the new helper
- [x] Two unit tests cover the helper: (1) profile with `bootstrap_from` declared yields `auto`, (2) profile without declaration yields `ssh:<host>` + tip text
- [x] `cargo test --release -p termlink --bin termlink heal_bootstrap_hint` passes (2 passed; 0 failed)
- [x] `cargo build --release -p termlink` succeeds (release build clean in 5m36s)

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

cargo test --release -p termlink --bin termlink heal_bootstrap_hint 2>&1 | tail -5 | grep -E "(test result: ok|0 failed)"

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

### 2026-05-16T21:21:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1648-fleet-doctor--suggest---bootstrap-from-a.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-38c25ee4
- **Timestamp:** 2026-05-16T21:37:48Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — New helper `heal_bootstrap_hint(entry, address)` in `crates/termlink-cli/src/commands/remote.rs` returns `--bootstrap-from auto` when `entry.bootstrap_from.is_some()`, otherwise `--bootstrap-from ssh:
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/remote.rs in: New helper `heal_bootstrap_hint(entry, address)` in `crates/termlink-cli/src/commands/remote.rs` returns `--bootstrap-from auto` when `entry.bootstrap`

### 2026-05-16T21:34:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
