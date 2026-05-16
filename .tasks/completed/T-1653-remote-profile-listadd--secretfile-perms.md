---
id: T-1653
name: "remote profile list/add — secret_file perms surfacing (PL-159 mirror)"
description: >
  remote profile list/add — secret_file perms surfacing (PL-159 mirror)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T22:58:31Z
last_update: 2026-05-16T22:58:31Z
date_finished: null
---

# T-1653: remote profile list/add — secret_file perms surfacing (PL-159 mirror)

## Context

PL-159 says: when shipping a config-driven mechanism, audit every operator-facing CLI surface (incident-time, idle-time, add-time, inspection) for whether it should reflect the new state. T-1652 just shipped `secret_file_perms_warning` at the incident-time + idle-time surfaces (`fleet status` actions, `fleet doctor` per-hub). This task fills the remaining two surfaces predicted by PL-159: add-time (`remote profile add`) and inspection (`remote profile list`).

The same warning helper from T-1652 is reused — no new detection logic, just wire-in to the two remaining surfaces. Catches the world-readable secret_file footgun at the moment of configuration (`profile add`) and during routine inspection (`profile list`) rather than waiting for the next `fleet status` / `fleet doctor` cycle.

## Acceptance Criteria

### Agent
- [x] `cmd_profile_list` (`ProfileAction::List` branch in `remote.rs`) reads each profile's `secret_file` (when present), expands `~/`, calls `secret_file_perms_warning`, and renders a warning row under the affected profile in text mode + a `secret_perms_warning` field in JSON output. **Text: `  Warning: ...` indented under the row. JSON: `"secret_perms_warning": <string-or-null>` field.**
- [x] `cmd_profile_add` (`ProfileAction::Add` branch) computes the warning at insertion time (after the profile is saved) and prints it in non-JSON output as a `Warning:` line (mirroring the existing "Tip:" pattern from T-1651 for bootstrap-readiness). **Reads back from `config.hubs.get(&name).secret_file` after insertion to sidestep the secret_file → HubEntry move at line 933.**
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated warning in termlink-mcp)
- [x] `cargo test -p termlink --bin termlink -- secret_file_perms expand_secret_file` passes — 5/5 T-1652 tests still green; no new tests needed since the helper logic is unchanged
- [x] Live verification: built debug binary, staged fake HOME with two profiles (`bad-perms` → 0o644 file, `good-perms` → 0o600 file). `profile list` showed Warning indented under `bad-perms` row only; `good-perms` clean. JSON output: `"secret_perms_warning"` was the full warning string for bad-perms and `null` for good-perms. `profile add new-profile ... --secret-file <0o644-file>` printed the Tip + Warning in sequence. Chmod 600 → list silent. Fixtures cleaned up.

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
cargo test -p termlink --bin termlink -- secret_file_perms expand_secret_file

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

### 2026-05-16T22:58:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1653-remote-profile-listadd--secretfile-perms.md
- **Context:** Initial task creation
