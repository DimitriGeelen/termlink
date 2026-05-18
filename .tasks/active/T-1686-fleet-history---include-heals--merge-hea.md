---
id: T-1686
name: "fleet history --include-heals — merge heal.log events into the rotation history view"
description: >
  fleet history --include-heals — merge heal.log events into the rotation history view

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-18T06:28:59Z
last_update: 2026-05-18T06:28:59Z
date_finished: null
---

# T-1686: fleet history --include-heals — merge heal.log events into the rotation history view

## Context

T-1685 added `~/.termlink/heal.log` for the operator-actionable audit
trail of `--auto-heal` decisions. T-1671's `fleet history` reads
`~/.termlink/rotation.log` (state transitions only). To answer
"what happened with hub X yesterday?" the operator now has to read
both. Add `--include-heals` to `fleet history` so a single command
returns both event types interleaved in time order.

Behaviour:
- Default (no `--include-heals`): unchanged — rotation events only.
  Preserves the T-1671 surface.
- `--include-heals`: ALSO read heal.log; interleave entries by `ts`;
  rendering distinguishes by an `event_type` field (rotation vs heal).
- `--hub <name>` filter applies to both event types.
- Text mode: heal entries render as `<ts>  <hub>  HEAL/<mode>     trigger=<t> action=<a>`.
- JSON mode: each emitted line carries `event_type: "rotation" | "heal"` so
  downstream parsers can branch on it.
- Summary footer adds heal-count alongside rotation-count per hub when
  `--include-heals` is set.

## Acceptance Criteria

### Agent
- [x] `--include-heals` flag added to `fleet history` clap variant
- [x] Reads heal.log when present; gracefully skips when absent (rotation_path.exists() gate relaxed when --include-heals)
- [x] Time-ordered output: entries sorted by `ts` when --include-heals
- [x] Text mode renders heal entries: `<ts>  <hub>  HEAL/<mode> trigger=<t> action=<a>`
- [x] JSON mode tags each entry with `event_type: "rotation" | "heal"`
- [x] `--hub` filter respects both event sources (verified live with synthetic heal.log)
- [x] Summary footer shows rotation/heal counts per hub when `--include-heals` (e.g. `ring20-management        rotation= 0  heal= 1`)
- [x] Without `--include-heals`: output matches T-1671 exact format (verified — 3 events from existing rotation.log render identically before/after)
- [x] `cargo check -p termlink` passes clean
- [x] Smoke: synthesized 2-entry heal.log, ran all four modes (no flag, --include-heals, --hub filter, --json) — output as designed for each. State restored.

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
./target/debug/termlink fleet history --help 2>&1 | grep -q "include-heals"

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

### 2026-05-18T06:28:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1686-fleet-history---include-heals--merge-hea.md
- **Context:** Initial task creation
