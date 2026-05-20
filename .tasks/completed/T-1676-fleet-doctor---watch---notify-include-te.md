---
id: T-1676
name: "fleet doctor --watch --notify: include TERMLINK_WATCH_TS in env-var contract (T-1669 follow-up)"
description: >
  fleet doctor --watch --notify: include TERMLINK_WATCH_TS in env-var contract (T-1669 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-17T21:20:37Z
last_update: 2026-05-17T21:42:32Z
date_finished: 2026-05-17T21:42:32Z
---

# T-1676: fleet doctor --watch --notify: include TERMLINK_WATCH_TS in env-var contract (T-1669 follow-up)

## Context

T-1669 ships the `--notify <cmd>` event hook with 8 env vars (HUB, CHANGE_KIND, OLD/NEW × CONN/PIN/LEGACY). Conspicuously missing: **when** the change was detected. Operator auto-heal scripts that log "drift detected at X" can only use `date` inside the script — which captures the script's launch time, not the watch loop's diff-cycle time. Drift between the two grows with notify-script duration and matters for forensic log correlation (e.g. "did the rotation happen before or after the X event we're investigating?").

Add `TERMLINK_WATCH_TS` to the env-var contract — RFC3339 timestamp of when the watch loop detected the change. Free to produce (we already format `now_rfc3339()` inside `cmd_fleet_doctor_watch`'s logging path) and consumed once per change event. Adds zero hot-path overhead and zero new dependencies.

## Acceptance Criteria

### Agent
- [x] `fire_notify` gains a `ts: &str` parameter and passes it via `.env("TERMLINK_WATCH_TS", ts)`
- [x] All 3 fire_notify call sites pass `&crate::manifest::now_rfc3339()` (or equivalent already-computed RFC3339 timestamp from the same diff cycle)
- [x] `cargo check --workspace` passes
- [x] CLAUDE.md auto-heal recipe (T-1670 section) mentions `TERMLINK_WATCH_TS` in the env-var listing
- [x] Live smoke: `--notify` script reads `TERMLINK_WATCH_TS` and the value is RFC3339-formatted, NOT "(unset)"

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

cargo check --workspace
bash -c "grep -q 'TERMLINK_WATCH_TS' crates/termlink-cli/src/commands/remote.rs"
bash -c "grep -q 'TERMLINK_WATCH_TS' CLAUDE.md"


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

### 2026-05-17T21:20:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1676-fleet-doctor---watch---notify-include-te.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-710250b0
- **Timestamp:** 2026-05-17T21:42:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T21:42:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
