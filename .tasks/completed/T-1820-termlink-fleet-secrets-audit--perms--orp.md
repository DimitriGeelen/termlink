---
id: T-1820
name: "termlink fleet secrets-audit — perms + orphan check on ~/.termlink/secrets (closes G-011 item 4)"
description: >
  termlink fleet secrets-audit — perms + orphan check on ~/.termlink/secrets (closes G-011 item 4)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/aggregator.rs, crates/termlink-hub/src/channel.rs, crates/termlink-protocol/src/events.rs]
related_tasks: []
created: 2026-05-28T06:32:50Z
last_update: 2026-05-28T06:43:09Z
date_finished: 2026-05-28T06:43:09Z
---

# T-1820: termlink fleet secrets-audit — perms + orphan check on ~/.termlink/secrets (closes G-011 item 4)

## Context

G-011 medium-term mitigation item 4: `~/.termlink/secrets/*.hex` files can drift to insecure
perms (the 2026-04 incident observed `proxmox4.hex` at 0o644). The existing `secret_file_perms_warning`
helper in `cmd_fleet_status` only inspects perms for hex files **referenced by a hubs.toml profile**;
orphaned cache files (left behind after profile removal, IP renumbering, or legacy heal flows)
are never inspected. This task adds a standalone `fleet secrets-audit` verb that walks the
directory directly and flags both insecure perms AND orphan status.

Read-only; never writes; never authenticates; suitable for cron/CI surveillance.

## Acceptance Criteria

### Agent
- [x] `FleetAction::SecretsAudit { json: bool, dir: Option<String> }` variant added to `cli.rs` (default dir = `~/.termlink/secrets`)
- [x] `main.rs` routes to a new `commands::remote::cmd_fleet_secrets_audit` function
- [x] Implementation walks the dir, stats every `*.hex` file, classifies each as `ok` / `warn-perms` / `warn-format` (not 64-char hex) / `info-orphan` (perms ok but no profile in hubs.toml references it) — additive: a file can be both warn-perms AND info-orphan
- [x] Per-file output (text mode): one row `<status> 0o<mode> <path> [reason]`; summary line at end (e.g. "5 files: 4 ok, 1 warn-perms, 1 info-orphan")
- [x] JSON mode: `{ok, dir, files: [{path, mode, size, status, reasons[], referenced_by[]}], summary: {total, ok, warn_perms, warn_format, info_orphan}}`
- [x] Exit code: 0 if every file is `ok` OR only `info-orphan`; 1 if any `warn-perms` or `warn-format` (cron-friendly: orphans informational, perms actionable)
- [x] Pure-function unit tests for the classifier (perms 0o600 = ok; 0o644 = warn-perms; 0o400 = ok per existing helper; 32-byte payload = ok-format; 17-byte = warn-format) — 6/6 pass (exceeds "at least 4" target)
- [x] `cargo check -p termlink` clean
- [x] Release build succeeds; `target/release/termlink fleet secrets-audit --help` shows the new subcommand. Live run on .107 found 4 files (3 ok, 1 info-orphan: proxmox4.hex — the G-011 incident file, now correctly chmod'd 0o600 but profile removed)

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
cargo check -p termlink
cargo test -p termlink --bin termlink secrets_audit

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

## Recommendation

[GO] `termlink fleet secrets-audit` shipped. Closes G-011 medium-term mitigation item 4
(orphan-and-perms audit on `~/.termlink/secrets/*.hex`). Read-only, no auth, no network — safe
for cron. Exit code 1 only on actionable problems (perms/format), so `fleet secrets-audit ||
mail -s ALERT` is a clean cron one-liner. Orphans are informational (operator decides whether
to remove the file or restore the profile).

Live run on .107 already produced direct operator value: surfaced `proxmox4.hex` as orphan
(the G-011 incident file — was originally at 0o644, now correctly 0o600, but the profile was
removed during cleanup). The orphan flag is the structural reminder to either restore-and-use
or delete.

Follow-up candidates (file as needed): (1) Wire a cron entry on .107 (similar to release-mirror
canary). (2) MCP parity `termlink_fleet_secrets_audit` for agent-callable surveillance. (3)
Item 1 of G-011 (mtime drift comparison against self-hub `<runtime_dir>/hub.secret`) — needs
self-hub IP detection, a separate scoped unit. (4) `--prune-orphans` write mode — deferred
until policy decided (delete vs. quarantine; Tier-2 gate).

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

### 2026-05-28T06:32:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1820-termlink-fleet-secrets-audit--perms--orp.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a133b950
- **Timestamp:** 2026-05-28T06:43:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T06:43:09Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
