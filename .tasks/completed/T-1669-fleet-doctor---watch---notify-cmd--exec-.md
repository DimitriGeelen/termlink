---
id: T-1669
name: "fleet doctor --watch --notify <cmd> — exec hook on state change"
description: >
  fleet doctor --watch --notify <cmd> — exec hook on state change

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T20:02:34Z
last_update: 2026-05-17T20:07:12Z
date_finished: 2026-05-17T20:07:12Z
---

# T-1669: fleet doctor --watch --notify <cmd> — exec hook on state change

## Context

T-1667 added `fleet doctor --watch` for continuous monitoring with state-diff
output. Operators currently have to read the terminal output to notice
rotations. A `--notify <cmd>` option invokes an operator-supplied shell
command whenever a hub's state changes (between cycles, not on baseline).
This is the right abstraction layer: termlink ships detection + change
events; the operator ships response policy (notify, page, auto-heal).

Closes the operator-workflow loop without forcing inline `--auto-heal`
semantics (which would carry write-side risk and require profile-level
`bootstrap_from` declarations). Operators who want auto-heal can write a
3-line bash script that calls `termlink fleet reauth $hub --bootstrap-from auto`.

Per-event environment passed to <cmd>:
- TERMLINK_WATCH_HUB: hub profile name
- TERMLINK_WATCH_OLD_CONN, TERMLINK_WATCH_NEW_CONN: connectivity status transition
- TERMLINK_WATCH_OLD_PIN, TERMLINK_WATCH_NEW_PIN: pin status transition (or "-" if not tracked)
- TERMLINK_WATCH_OLD_LEGACY, TERMLINK_WATCH_NEW_LEGACY: legacy count transition
- TERMLINK_WATCH_CHANGE_KIND: "transition" | "new" | "removed"

## Acceptance Criteria

### Agent
- [x] `--notify <cmd>` flag accepted on `fleet doctor` (only valid with `--watch`) — `--notify requires --watch` verified
- [x] When a hub's state changes during watch, the cmd is invoked once per affected hub — `synthetic-flap NEW` triggered notify in cycle 2
- [x] Environment passed to cmd: TERMLINK_WATCH_HUB + OLD/NEW for conn, pin, legacy + CHANGE_KIND — `synthetic-flap:new::error` (HUB:KIND:OLD_CONN:NEW_CONN) written via env-var template
- [x] Baseline cycle (cycle 1) does NOT trigger notifications — `/tmp/t1669-notify.tag` had 1 line for cycle-2 only event
- [x] Notify is fire-and-forget (don't await): cmd spawned via `std::process::Command::spawn()`, not `.output().await`
- [x] Notify failure (cmd not found) is logged to stderr but does NOT kill the watch — `sh: 1: /tmp/nonexistent-script.sh: not found` printed, watch continued to cycle 2 + clean SIGINT exit
- [x] Live smoke: notify writes a tag file when invoked; baseline produced no file, change cycle produced file containing `synthetic-flap:new::error`
- [x] `cargo check -p termlink` passes; no new clippy classes in touched files — only pre-existing tools.rs warning

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
grep -q "notify:" crates/termlink-cli/src/cli.rs
grep -q "TERMLINK_WATCH_HUB" crates/termlink-cli/src/commands/remote.rs
bash -c './target/debug/termlink fleet doctor --notify /tmp/nope.sh 2>&1 || true' | grep -q "requires --watch"

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

### 2026-05-17T20:02:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1669-fleet-doctor---watch---notify-cmd--exec-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e52538a8
- **Timestamp:** 2026-05-17T20:07:21Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** yes
- **Findings:** 2

**Per-AC findings:**

- **AC#4 (Agent)** — Baseline cycle (cycle 1) does NOT trigger notifications — `/tmp/t1669-notify.tag` had 1 line for cycle-2 only event
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tmp/t1669-notify.tag in: Baseline cycle (cycle 1) does NOT trigger notifications — `/tmp/t1669-notify.tag` had 1 line for cycle-2 only event`
- **AC#6 (Agent)** — Notify failure (cmd not found) is logged to stderr but does NOT kill the watch — `sh: 1: /tmp/nonexistent-script.sh: not found` printed, watch continued to cycle 2 + clean SIGINT exit
  - **AC-verify-mismatch** (narrow, heuristic) — `path=tmp/nonexistent-script.sh in: Notify failure (cmd not found) is logged to stderr but does NOT kill the watch — `sh: 1: /tmp/nonexistent-script.sh: not found` printed, watch continu`

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-17T20:07:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
