---
id: T-1683
name: "fleet doctor --auto-heal one-shot — heal without --watch (T-1680 ergonomic extension)"
description: >
  fleet doctor --auto-heal one-shot — heal without --watch (T-1680 ergonomic extension)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-18T06:10:18Z
last_update: 2026-05-18T06:15:37Z
date_finished: 2026-05-18T06:15:37Z
---

# T-1683: fleet doctor --auto-heal one-shot — heal without --watch (T-1680 ergonomic extension)

## Context

T-1680 added `fleet doctor --watch --auto-heal` (built-in cert-drift heal)
gated by clap `requires = "watch"`. The gate made sense at the time —
T-1680's mental model was "watch-loop detects transitions, heal fires on
them." But operators also page-respond to a single suspected rotation
("doctor says drift, fix it"), and there the watch requirement is friction:
they must start a watch loop, wait one cycle for it to baseline, wait
another cycle for it to detect drift (when it's already on the wire),
then Ctrl-C the loop. One command should do this.

Single-shot semantics for `--auto-heal`:
1. Run the existing fleet-doctor sweep (with `--include-pin-check` if requested)
2. After the sweep, classify per-hub state from `hub_results` (already in JSON)
3. For each hub where current state shows drift OR auth-mismatch AND profile
   has declared `bootstrap_from`: fire `fire_auto_heal(hub, ts)`
4. Same R2 gate — declared `bootstrap_from` required; missing → stderr hint

The fire-and-forget heal subprocess is identical to the watch path.
Difference is purely "should we be in a loop to do this?" — answer: no.

## Acceptance Criteria

### Agent
- [x] `--auto-heal` accepted without `--watch` in clap — `requires = "watch"` removed; doc-comment updated to document both modes
- [x] Single-shot path: post-loop iteration over `hub_results` fires heal for any hub in `pin=drift` OR `conn=auth-mismatch` AND with declared `bootstrap_from`; reuses existing `fire_auto_heal` + `derive_watch_conn` helpers
- [x] Profiles without declared `bootstrap_from` printed to stderr — `[SKIP] <name>: no bootstrap_from declared (R2 ...)` lines after the "Auto-heal: fired N (one-shot, T-1683)" header
- [x] `--auto-heal` without `--include-pin-check`: info hint printed at command start — `[info] --auto-heal without --include-pin-check: only conn=auth-mismatch heals will fire. Pass --include-pin-check to also heal on cert drift.` — verified live
- [x] `cargo check -p termlink` passes (clean)
- [x] Smoke: `fleet doctor --auto-heal --include-pin-check` ran against 5-hub local fleet — pin=match=3, drift=0, no-pin=1, probe-fail=1; no heals fired (correct — nothing to heal); no panics
- [x] Smoke: `fleet doctor --auto-heal` (no pin-check) — info hint shown at top of output as designed
- [x] CLAUDE.md: detection-verbs row added for one-shot mode; recipe section now documents both continuous and one-shot forms

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

cargo check -p termlink 2>&1 | tail -5
./target/debug/termlink fleet doctor --help 2>&1 | grep -q "auto-heal"

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

### 2026-05-18T06:10:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1683-fleet-doctor---auto-heal-one-shot--heal-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-9e29c2fe
- **Timestamp:** 2026-05-18T06:15:45Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-18T06:15:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
