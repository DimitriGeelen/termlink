---
id: T-1667
name: "fleet doctor --watch mode for continuous rotation monitoring"
description: >
  fleet doctor --watch mode for continuous rotation monitoring

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T19:36:24Z
last_update: 2026-05-17T19:53:30Z
date_finished: 2026-05-17T19:53:30Z
---

# T-1667: fleet doctor --watch mode for continuous rotation monitoring

## Context

T-1666 unified single-shot auth + cert diagnosis into `fleet doctor --include-pin-check`.
Operators still need to run it on a cron/timer to catch rotations as they happen. A
`--watch <seconds>` mode gives a long-running diagnostic that polls and emits ONLY
state-diffs — so operators can leave it running in a terminal and see rotation events
as plain timestamped lines instead of drowning in repeat output.

Completes the rotation-protocol stack at the continuous-monitoring layer:
- Detection (single-shot): T-1666 fleet doctor --include-pin-check
- Detection (continuous): THIS task
- Active heal: future T-1668 candidate (--auto-heal flag, requires bootstrap_from declared)

## Acceptance Criteria

### Agent
- [x] `fleet doctor --watch <N>` flag added to CLI with validation 5 <= N <= 3600 seconds — `Error: --watch: interval must be 5..=3600 seconds (got 3)` verified
- [x] `--watch` is mutually compatible with `--include-pin-check` (auth + cert observed together) — verified, just slow when fleet has unreachable hub (probe_cert serialization)
- [x] First cycle emits full state as a baseline (same shape as single-shot) — `baseline: 5 hub(s)` + per-hub lines confirmed
- [x] Subsequent cycles emit ONLY state changes per hub, prefixed with RFC3339 timestamp — cycle 2 emitted no lines (no changes), confirmed
- [x] State tracked per hub: (connectivity, pin_status, legacy_count). A change in any of three fires a line
- [x] SIGINT (Ctrl-C) cleanly exits with a "watch stopped" line and exit code 0 — `watch stopped (sigint, completed 2 cycle(s))` confirmed
- [x] Live smoke against the 5-hub fleet: 2 cycles, no false positives, plain output — `/tmp/t1667-smoke2.out` 2026-05-17T19:49:43Z baseline, 19:50:08Z stop, 0 spurious lines
- [x] `cargo check -p termlink` passes with no new warnings in touched files — only pre-existing tools.rs:13024 warning
- [x] No new clippy classes in touched files — the existing "too many arguments" warning on cmd_fleet_doctor went 12→13 (same class); cmd_fleet_doctor_watch itself has 7 args (under threshold)
- [x] Incompatible-flag guards: --diff / --save-snapshot / --exit-code-on-verdict / --trend rejected with "single-shot semantics" hint

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

cargo check -p termlink 2>&1 | grep -q "Finished"
grep -q "cmd_fleet_doctor_watch" crates/termlink-cli/src/commands/remote.rs
grep -q "watch: Option<u64>" crates/termlink-cli/src/cli.rs
bash -c './target/debug/termlink fleet doctor --watch 3 2>&1 || true' | grep -q "must be 5..=3600"
bash -c './target/debug/termlink fleet doctor --watch 10 --diff /tmp/nope.json 2>&1 || true' | grep -q "incompatible with"

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

### 2026-05-17T19:36:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1667-fleet-doctor---watch-mode-for-continuous.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-2e3368c8
- **Timestamp:** 2026-05-17T19:53:30Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 1

**Per-AC findings:**

- **AC#7 (Agent)** — Live smoke against the 5-hub fleet: 2 cycles, no false positives, plain output — `/tmp/t1667-smoke2.out` 2026-05-17T19:49:43Z baseline, 19:50:08Z stop, 0 spurious lines
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tmp/t1667-smoke2.out in: Live smoke against the 5-hub fleet: 2 cycles, no false positives, plain output — `/tmp/t1667-smoke2.out` 2026-05-17T19:49:43Z baseline, 19:50:08Z stop`

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-17T19:53:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
