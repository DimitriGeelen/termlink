---
id: T-1652
name: "fleet doctor + status — secret_file perms warning (G-011)"
description: >
  fleet doctor + status — secret_file perms warning (G-011)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T22:48:26Z
last_update: 2026-05-16T22:48:26Z
date_finished: null
---

# T-1652: fleet doctor + status — secret_file perms warning (G-011)

## Context

G-011 sub-point #4: at least one `~/.termlink/secrets/*.hex` observed at chmod 644 (world-readable) — a 32-byte HMAC secret that authenticates every fleet operation, sitting world-readable in a home directory is a security smell that no current code path catches. The mitigation candidate names this as a `fw doctor` freshness/perms check.

Highest-severity, smallest-surface piece: warn at every operator touchpoint where the file is consulted (`fleet doctor` per-hub line, `fleet status` per-hub line) when the `secret_file`'s Unix perms grant group or world read/write access. Catches the leak risk before a peer-share or shoulder-surf incident.

Sibling work in `what_remains` (mtime freshness check, IP-keyed cache deprecation) is out of scope for this task — those need a separate cross-host comparison story.

## Acceptance Criteria

### Agent
- [x] New helper `secret_file_perms_warning(path: &Path) -> Option<String>` in `crates/termlink-cli/src/commands/remote.rs` — returns `Some("...")` when the file's mode bits include group or world read/write/execute (`& 0o077 != 0`), `None` otherwise (or when the file doesn't exist / metadata read fails — silent for missing, those have their own messages elsewhere). **Implemented at remote.rs near format_hmac_mismatch_diagnosis; #[cfg(unix)] gated with non-unix stub returning None.**
- [x] Helper formats with chmod-octal-display + remediation hint: `"secret_file perms 0o{mode:03o} expose secret to group/world — run: chmod 600 {path}"`. **Verified live: `secret_file perms 0o644 expose secret to group/world — run: chmod 600 /tmp/T-1652-test-secret.hex`**
- [x] `cmd_fleet_doctor` per-hub block: when `entry.secret_file.is_some()`, expand `~` and call helper; if Some, emit as a warning line (text and JSON). **Wired at hub_obj construction for PASS branch + both Err branches (Ok(Err(e)) and timeout Err(_)) so warning fires even when probe fails. JSON: `secret_perms_warning` field. Text: `[WARN] ...` line under each per-hub status.**
- [x] `cmd_fleet_status` per-hub block: same wire-in, emit as an action_item-style line so the existing "Reauth needed" pattern is mirrored. **Pushed to `actions` Vec early in the loop, before the connect attempt — fires before reachability is known.**
- [x] Unit tests in `mod tests`: (a) helper returns None for 0o600, (b) helper returns Some for 0o644, (c) helper returns Some for 0o660, (d) helper returns None for non-existent path — using `tempfile` + `std::os::unix::fs::PermissionsExt`. **All 4 added; plus a 5th `expand_secret_file_path_substitutes_home_for_tilde` for the helper's HOME-expansion contract.**
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated warning in termlink-mcp)
- [x] `cargo test -p termlink --bin termlink -- secret_file_perms expand_secret_file` passes — 5/5 PASS (note: cli crate name is `termlink` not `termlink-cli`, and target is `--bin termlink` since cli has no lib target)
- [x] Live verification: built debug binary, staged fake HOME with `/tmp/T-1652-test-secret.hex` at chmod 644 + a test hubs.toml profile pointing at it. Ran `fleet status` → ACTIONS line "test-bad-perms: secret_file perms 0o644 expose secret to group/world — run: chmod 600 /tmp/T-1652-test-secret.hex" surfaced as item #1, alongside the existing Reauth action. Ran `fleet doctor` → `[WARN] secret_file perms 0o644 expose...` rendered under the per-hub FAIL block. Chmod 600 → both warnings silent. Fixtures cleaned up.

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
grep -q "secret_file_perms_warning" crates/termlink-cli/src/commands/remote.rs

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

### 2026-05-16T22:48:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1652-fleet-doctor--status--secretfile-perms-w.md
- **Context:** Initial task creation
