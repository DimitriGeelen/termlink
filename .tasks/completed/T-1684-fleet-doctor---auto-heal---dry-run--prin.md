---
id: T-1684
name: "fleet doctor --auto-heal --dry-run — print intended heals without firing them"
description: >
  fleet doctor --auto-heal --dry-run — print intended heals without firing them

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-18T06:17:07Z
last_update: 2026-05-18T06:23:25Z
date_finished: 2026-05-18T06:23:25Z
---

# T-1684: fleet doctor --auto-heal --dry-run — print intended heals without firing them

## Context

T-1680/T-1681/T-1683 made `--auto-heal` a fire-and-forget heal trigger.
Operators rolling out automation always want to preview what would happen
before they actually wire it in. Without dry-run the only way to know
what `--auto-heal` will do is to let it do it — which spawns subprocesses,
fetches secrets, writes hex files, and re-pins TOFU caches. Dry-run
flips off the actual fire while preserving all the classification and
output.

Semantics:
- `--dry-run` requires `--auto-heal` (alone it's meaningless — there's
  nothing to dry-run)
- Per affected hub: instead of `fire_auto_heal(name, ts)`, print the
  full intended command line to stderr with a `[DRY-RUN]` prefix
- Skip-no-anchor lines remain (same as live mode) so operator sees full
  preview including the gaps
- Header line says "Auto-heal: would fire N (dry-run, T-1684)" instead
  of "fired N"
- Applies to both single-shot and watch modes
- JSON mode (--json): include `dry_run: true` in auto-heal summary so
  tooling can distinguish dry vs live runs

## Acceptance Criteria

### Agent
- [x] `--dry-run` flag accepted with `requires = "auto_heal"`; clap rejects bare `--dry-run` (verified: `fleet doctor --dry-run` exits "the following required arguments were not provided: --auto-heal")
- [x] Single-shot path: dry-run branch prints `[DRY-RUN] would fire: termlink fleet reauth <name> --bootstrap-from auto` per affected hub (verified live with synth-drift on ring20-dashboard + temporary `bootstrap_from`)
- [x] Watch path: dry-run gate added to transition handler — same `[DRY-RUN] would fire` line on transitions instead of `fire_auto_heal`
- [x] Header line: "Auto-heal: would fire 1 (dry-run, T-1684)" vs "Auto-heal: fired N (one-shot, T-1683)" — verified live both paths
- [x] Without `--auto-heal`: clap rejects `--dry-run` ✓
- [x] `cargo check -p termlink` passes
- [x] Smoke: synth-drift on ring20-dashboard (sed-replaced its TOFU fingerprint to all-zeros). With NO `bootstrap_from`: SKIP line emitted, "would fire 0". With temp `bootstrap_from = "file:/tmp/fake-anchor"`: "[DRY-RUN] would fire: termlink fleet reauth ring20-dashboard --bootstrap-from auto" + "would fire 1". State restored.

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
./target/debug/termlink fleet doctor --help 2>&1 | grep -q "dry-run"

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

### 2026-05-18T06:17:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1684-fleet-doctor---auto-heal---dry-run--prin.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-be6e8c6e
- **Timestamp:** 2026-05-18T06:23:36Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-18T06:23:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
