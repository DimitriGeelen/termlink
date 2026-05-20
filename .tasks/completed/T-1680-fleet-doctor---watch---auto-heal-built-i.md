---
id: T-1680
name: "fleet doctor --watch --auto-heal: built-in cert-drift auto-heal via declared bootstrap_from (T-1669 + T-1679 capstone)"
description: >
  fleet doctor --watch --auto-heal: built-in cert-drift auto-heal via declared bootstrap_from (T-1669 + T-1679 capstone)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-17T22:20:01Z
last_update: 2026-05-17T22:31:32Z
date_finished: 2026-05-17T22:31:32Z
---

# T-1680: fleet doctor --watch --auto-heal: built-in cert-drift auto-heal via declared bootstrap_from (T-1669 + T-1679 capstone)

## Context

CLAUDE.md's auto-heal recipe (T-1670 section) tells operators to write a 3-line shell script that gates on `TERMLINK_WATCH_NEW_PIN=drift` and execs `termlink fleet reauth $TERMLINK_WATCH_HUB --bootstrap-from auto`. That works, but every operator writes the same script. Promote it to a built-in flag.

`--auto-heal` (requires `--watch`): on every per-hub change event, if `new_pin == "drift"` AND the profile declares `bootstrap_from`, spawn the heal subprocess inline (fire-and-forget, same pattern as `fire_notify`). Per CLAUDE.md R2, the trust anchor must be out-of-band; this gates on a declared anchor only.

## Acceptance Criteria

### Agent
- [x] `fleet doctor --watch --auto-heal` parses; `--auto-heal` requires `--watch` (clap `requires = "watch"` + runtime guard)
- [x] In the change-detection loop, after fire_notify, when change kind is `transition` AND `new_pin == "drift"` AND profile has declared `bootstrap_from`, an internal heal subprocess is spawned (fire-and-forget — does NOT block the next watch cycle)
- [x] Profile without declared `bootstrap_from`: NO heal attempted (R2 — anchor must be operator-declared); a one-line stderr hint explains why it was skipped
- [x] Each heal logs one line to stderr at fire time
- [x] `cargo check --workspace` passes
- [x] CLAUDE.md detection-verbs table mentions `--auto-heal`; auto-heal recipe section promoted to recommend the built-in flag with the script form kept for custom-logic cases
- [x] Live smoke: --help shows new flag; clap rejects `--auto-heal` without `--watch`; --watch --auto-heal runs without panic on local fleet

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

### 2026-05-17T22:20:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1680-fleet-doctor---watch---auto-heal-built-i.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-db91ed9d
- **Timestamp:** 2026-05-17T22:31:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T22:31:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
