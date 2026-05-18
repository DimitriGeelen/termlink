---
id: T-1688
name: "fleet bootstrap-check — preflight validate declared bootstrap_from anchors"
description: >
  fleet bootstrap-check — preflight validate declared bootstrap_from anchors

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-18T07:00:47Z
last_update: 2026-05-18T07:08:42Z
date_finished: 2026-05-18T07:08:42Z
---

# T-1688: fleet bootstrap-check — preflight validate declared bootstrap_from anchors

## Context

Operators declare `bootstrap_from = "ssh:<host>"` or `"file:<path>"` on hub profiles (T-1291) so `--auto-heal` (T-1680/T-1683) can fire `fleet reauth <profile> --bootstrap-from auto`. The declared channel is only exercised when an actual heal fires — if the OOB channel is broken (host down, ssh key missing, file moved, secret no longer 64-hex), the operator finds out under pressure during a rotation event, not at declaration time.

`fleet bootstrap-check` runs steps 1-2 of the reauth bootstrap path — `fetch_bootstrap_secret(source)` + `normalize_and_validate_secret_hex(raw)` — without ever writing the secret file. Operator gets an early answer: "will this anchor work when I need it?"

## Acceptance Criteria

### Agent
- [x] New `FleetSub::BootstrapCheck { profile?, all, json }` clap variant in `crates/termlink-cli/src/cli.rs`; either `profile` or `--all` must be present (validated in command body — clap mutex via `conflicts_with`)
- [x] Implementation `cmd_fleet_bootstrap_check` in `crates/termlink-cli/src/commands/remote.rs` reuses existing `fetch_bootstrap_secret` + `normalize_and_validate_secret_hex`; **no `write_secret_file` call**, **no `std::fs::copy` to back up**, no mutation of any secret file
- [x] Per-profile status taxonomy: `ok`, `no-anchor`, `fetch-fail`, `invalid-format` — all four classify paths covered by `classify_bootstrap_check` unit tests
- [x] Exit codes 0/1/2 implemented via `bootstrap_check_exit_code` helper; covered by 5 unit tests; live-smoked exit 2 with `--all` no-anchor case
- [x] `--json` flag emits `{verdict, profiles: [{name, address, bootstrap_from, status, error?}]}` — shape confirmed via live smoke
- [x] 11 unit tests in `tests` module cover classify mapping, exit-code rollup, and verdict words; all pass
- [x] CLAUDE.md row added under the auto-heal block matching the `fleet history` entry style
- [x] `cargo check -p termlink` clean; `cargo test -p termlink --bin termlink bootstrap_check` shows 11/11 ok

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
cargo check -p termlink 2>&1 | tail -3 | grep -q "Finished"
cargo test -p termlink --bin termlink bootstrap_check 2>&1 | tail -3 | grep -qE "ok\.|test result: ok"

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

### 2026-05-18T07:00:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1688-fleet-bootstrap-check--preflight-validat.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d5e93cbe
- **Timestamp:** 2026-05-18T07:08:45Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#1 (Agent)** — New `FleetSub::BootstrapCheck { profile?, all, json }` clap variant in `crates/termlink-cli/src/cli.rs`; either `profile` or `--all` must be present (validated in command body — clap mutex via `confli
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/cli.rs in: New `FleetSub::BootstrapCheck { profile?, all, json }` clap variant in `crates/termlink-cli/src/cli.rs`; either `profile` or `--all` must be present (`
- **AC#2 (Agent)** — Implementation `cmd_fleet_bootstrap_check` in `crates/termlink-cli/src/commands/remote.rs` reuses existing `fetch_bootstrap_secret` + `normalize_and_validate_secret_hex`; **no `write_secret_file` call
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-cli/src/commands/remote.rs in: Implementation `cmd_fleet_bootstrap_check` in `crates/termlink-cli/src/commands/remote.rs` reuses existing `fetch_bootstrap_secret` + `normalize_and_v`

### 2026-05-18T07:08:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
