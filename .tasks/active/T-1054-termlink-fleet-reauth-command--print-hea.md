---
id: T-1054
name: "termlink fleet reauth command — print heal incantation for a profile (Tier-1)"
description: >
  termlink fleet reauth command — print heal incantation for a profile (Tier-1)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T19:54:41Z
last_update: 2026-04-14T19:54:41Z
date_finished: null
---

# T-1054: termlink fleet reauth command — print heal incantation for a profile (Tier-1)

## Context

Third build task from T-1051 inception (Option D, Tier-1 heal command).
Adds `termlink fleet reauth <profile>` that prints the exact copy-pasteable
incantation to refresh a cached secret for a hub profile. Tier-1 only:
no automation, no SSH, no `--bootstrap-from` flag (that's T-1055 per R2).

Output must be self-contained and safe — the command reads local config,
never writes, and never contacts the hub. It makes the operator heal path
a one-command lookup instead of a memory exercise.

## Acceptance Criteria

### Agent
- [x] New `Fleet Reauth { profile: String }` CLI subcommand registered in `cli.rs` and dispatched in `main.rs`
- [x] `cmd_fleet_reauth(profile)` in `remote.rs`:
  - [x] Looks up the profile from `~/.termlink/hubs.toml`
  - [x] Prints: profile name, hub address, where the local secret is cached (secret_file path or "inline secret" warning)
  - [x] Prints the refresh incantation: SSH read from hub + write to local secret file + chmod 600
  - [x] Explicitly tags the trust anchor as OUT-OF-BAND (R2 compliance: `--bootstrap-from` not yet available)
  - [x] Prints the verify command (`termlink fleet doctor`)
  - [x] Errors cleanly when the profile doesn't exist (actionable message, exit 1)
- [x] 5 unit tests: render-with-secret-file, render-with-inline-secret, render-with-no-secret, unknown-profile error, empty-hubs-config error
- [x] `cargo build -p termlink` clean, zero new clippy warnings
- [x] `cargo test -p termlink --bin termlink -- fleet_reauth` passes (5 tests)
- [x] Manual smoke test: live `termlink fleet reauth ring20-management` renders correctly; `termlink fleet reauth nonsense` exits 1 with actionable error

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

cargo build -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink -- fleet_reauth 2>&1 | grep -E "[0-9]+ passed"

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-14T19:54:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1054-termlink-fleet-reauth-command--print-hea.md
- **Context:** Initial task creation
