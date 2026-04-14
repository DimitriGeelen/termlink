---
id: T-1055
name: "termlink fleet reauth --bootstrap-from autonomous heal (R2)"
description: >
  termlink fleet reauth --bootstrap-from autonomous heal (R2)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T20:01:20Z
last_update: 2026-04-14T20:01:20Z
date_finished: null
---

# T-1055: termlink fleet reauth --bootstrap-from autonomous heal (R2)

## Context

Fourth build task from T-1051 inception (Option D, Tier-2, R2 compliance).
Extends `termlink fleet reauth` with an optional `--bootstrap-from <SOURCE>` flag
that actually *does* the heal: fetches the new secret via the named out-of-band
channel, validates it's 64-char hex, backs up the current file to `.bak`, and
atomically writes the new value at chmod 600.

The `<SOURCE>` must be explicit — there is no default. The operator picks the
trust anchor per incident. This is R2 compliance: the bootstrap channel is
out-of-band by construction because its identity is `file:` / `ssh:` — neither
of which depends on the termlink auth we're trying to heal.

Supported sources for this task:
- `file:<path>`  — read local file (the most portable — delivered via git, USB, etc.)
- `ssh:<host>`  — run `ssh <host> -- sudo cat /var/lib/termlink/hub.secret`

Deliberately out of scope for T-1055:
- `command:<cmd>` (arbitrary shell) — reserved for a later task with explicit security review
- `stdin` — trivial to add later, low payoff now
- Automatic discovery of the hub's secret path on non-default runtime_dirs

## Acceptance Criteria

### Agent
- [x] `--bootstrap-from <SOURCE>` flag added to `FleetAction::Reauth`
- [x] Source parser accepts `file:<path>` and `ssh:<host>`; rejects unknown prefixes with actionable error
- [x] Fetched value is trimmed + validated as 64-char ASCII hex; rejected otherwise
- [x] Profile must use `secret_file` — inline-secret profiles error out with migration hint
- [x] Existing secret_file is backed up to `<path>.bak` before being overwritten
- [x] New secret written at chmod 600 (atomic via `.hex.tmp` → rename)
- [x] Success message includes the new fingerprint preview (first 12 hex chars) so the operator can confirm without dumping the secret
- [x] When no `--bootstrap-from` provided, command falls back to Tier-1 printer (T-1054 behavior preserved)
- [x] 8 unit tests: hex validator accept/reject-length/reject-non-hex, unknown-prefix, empty-prefix, file-source happy path (incl. .bak), invalid-hex-rejection-preserves-file, inline-secret profile refusal
- [x] `cargo build -p termlink` clean, zero new clippy warnings
- [x] `cargo test -p termlink --bin termlink -- fleet_reauth` passes (13 tests — Tier-1 + Tier-2)
- [x] Full termlink test suite: 189/189 pass (stable across 3 runs); fixed pre-existing `dispatch::isolate_rejects_non_git_dir` CWD leak and `config::tests::save_and_load_hubs_config` HOME race via new crate-wide `test_env_lock::ENV_LOCK`

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

### 2026-04-14T20:01:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1055-termlink-fleet-reauth---bootstrap-from-a.md
- **Context:** Initial task creation
