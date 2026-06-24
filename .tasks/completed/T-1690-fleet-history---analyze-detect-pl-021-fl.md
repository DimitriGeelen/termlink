---
id: T-1690
name: "fleet history --analyze: detect PL-021 flap signatures from rotation.log"
description: >
  fleet history --analyze: detect PL-021 flap signatures from rotation.log

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-18T08:13:53Z
last_update: 2026-05-18T09:05:21Z
date_finished: 2026-05-18T09:05:21Z
---

# T-1690: fleet history --analyze: detect PL-021 flap signatures from rotation.log

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `fleet history --analyze` flag parses, scans rotation.log within `--since` window
- [x] Per-hub flap classifier: `clean` / `cert-only` / `secret-only` / `single-double-rotation` / `pl021-candidate`. PL-021 candidate fires at ≥2 double-rotations (same log row with both new_pin=drift AND new_conn=auth-mismatch) — empirically tighter than the original spec
- [x] Output cites CLAUDE.md "Special case — volatile runtime_dir (T-1290 / T-1294)" + emits diagnostic command set verbatim (ls, mount, tmpfiles.d)
- [x] Exit code 2 on any candidate, 0 otherwise (cron/CI alerting hook)
- [x] JSON mode emits structured per-hub verdicts (operator + agent friendly)
- [x] Unit tests (9/9 green): empty-log, only-new-entries-skipped, cert-only, secret-only, single-double, two-double-is-candidate, no-cross-hub-contamination, recovery-not-counted, stable-drifted-no-transition
- [x] `cargo build --release -p termlink` green; `target/release/termlink fleet history --analyze --since 30` smoke-test PASS on live log (no transitions → exit 0, expected)
- [x] CLAUDE.md table row added documenting the new flag

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

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink --bin termlink analyze_pl021 2>&1 | grep -q "test result: ok. 9 passed"
target/release/termlink fleet history --analyze --since 30 2>&1 | grep -q "PL-021 flap analysis"
target/release/termlink fleet history --analyze --json --since 30 2>&1 | grep -q '"pl021_candidates"'

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

### 2026-05-18T08:13:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1690-fleet-history---analyze-detect-pl-021-fl.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-3f3e88f0
- **Timestamp:** 2026-05-18T09:14:55Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T09:05:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
