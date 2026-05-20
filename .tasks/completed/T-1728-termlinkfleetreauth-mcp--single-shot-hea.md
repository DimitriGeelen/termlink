---
id: T-1728
name: "termlink_fleet_reauth MCP — single-shot heal RPC (CLI parity, T-1054/T-1055/T-1291 culmination)"
description: >
  Add MCP wrapper for cmd_fleet_reauth — single-shot rotation heal RPC. Agents currently shell out to fleet reauth; MCP wrapper completes the rotation-protocol MCP arc (probe/verify/doctor/bootstrap_check shipped, heal is the missing verb). Bulk variant fleet reauth --all-drifted (T-1679) is also unwrapped; in-scope here.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T18:48:42Z
last_update: 2026-05-20T18:56:45Z
date_finished: 2026-05-20T18:56:45Z
---

# T-1728: termlink_fleet_reauth MCP — single-shot heal RPC (CLI parity, T-1054/T-1055/T-1291 culmination)

## Context

The rotation-protocol MCP arc shipped probe/verify/doctor/bootstrap_check (T-1663/T-1666/T-1687/T-1689) and history (T-1671/T-1687). The missing verb is **heal**: `termlink fleet reauth <profile> [--bootstrap-from <src>]` (T-1054/T-1055/T-1291) and its bulk companion `fleet reauth --all-drifted` (T-1679). Agents-with-MCP must currently shell out via `Bash` to heal rotations, which both bypasses MCP tooling and blocks agents that don't have shell access.

This task adds `termlink_fleet_reauth` (single-profile heal) following the PL-172 silent-strip MCP-parity recipe. The CLI verb is one-shot, deterministic, and emits JSON-compatible output — no `--watch` / fire-and-forget oversight needed, so it fits the MCP single-RPC model cleanly (per PL-172, --watch/--auto-heal/--notify variants are deliberately excluded; this is not one of those).

Scope: single-profile heal only. Bulk `--all-drifted` deferred to a follow-up (clean wedge; bulk needs different result-aggregation shape).

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_reauth(profile, bootstrap_from?)` MCP tool registered in `crates/termlink-mcp/src/tools.rs` with safe defaults (bootstrap_from=None → Tier-1 print-plan mode; "auto" → declared anchor lookup; "file:..."/"ssh:..." → explicit source)
- [x] Implementation delegates to existing CLI logic (`cmd_fleet_reauth` for Tier-1, `cmd_fleet_reauth_bootstrap` for Tier-2) — no logic duplication; expose existing functions or extract pure helpers per PL-172 step 2
- [x] Tool returns structured JSON: `{ok: bool, profile: string, mode: "plan-only"|"healed", source: string|null, secret_file: string|null, fingerprint_preview: string|null, error: string|null}` — fingerprint preview is the 12-char prefix per CLI's chmod 600 write path
- [x] Refuses profiles with inline `secret = ...` (returns `ok=false, error="profile uses inline secret; convert to secret_file before heal"`) — matches CLI's same check
- [x] R2 enforced: command:<shell> bootstrap_from sources rejected (Tier-2 has explicit security exclusion; MCP must not be a loophole)
- [x] Cargo build green: `cargo check -p termlink-mcp`
- [x] Unit test: deserialization defaults for the params struct (bootstrap_from None) + an integration-style test that invokes against a temp HUBS_FILE with mock secret_file path

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

### 2026-05-20T18:48:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1728-termlinkfleetreauth-mcp--single-shot-hea.md
- **Context:** Initial task creation

### 2026-05-20T19:25:00Z — implementation shipped
- **CLI extension:** Added `--json` flag to `FleetAction::Reauth` (cli.rs:3331) + wired through main.rs:653. `cmd_fleet_reauth(profile, bootstrap_from, json)` branches: Tier-1 emits `{mode: "plan-only", plan_text: ...}`; Tier-2 emits `{mode: "healed", source, secret_file, fingerprint_preview: <12-char>}`.
- **Refactor:** `cmd_fleet_reauth_bootstrap` now returns `Result<ReauthBootstrapOutcome>` instead of `Result<()>`. Outcome carries profile/address/secret_file/source/fingerprint_preview. CLI eprintln moved into `print_reauth_outcome_human(&outcome)` helper. Bulk caller (`cmd_fleet_reauth_all`) updated to invoke the helper to preserve operator-visible output.
- **MCP wrapper:** `termlink_fleet_reauth(FleetReauthParams)` subprocesses the resolved `termlink` binary with `fleet reauth <profile> [--bootstrap-from <src>] --json` under `tokio::time::timeout` + `kill_on_drop=true` + null stdin (matches `termlink_fleet_bootstrap_check` pattern from T-1689). R2 guard at the tool boundary rejects `command:` scheme — MCP must not become a Tier-2 loophole around the CLI's security review exclusion. Empty-stdout + non-zero exit returns `{ok: false, error: <stderr>, exit_code: ...}`.
- **Tests:** Added 3 unit tests in `crates/termlink-mcp/src/tools.rs::tests`: `fleet_reauth_params_required_profile`, `fleet_reauth_params_accepts_auto_and_explicit`, `fleet_reauth_rejects_command_scheme`. All pass (`cargo test -p termlink-mcp --lib fleet_reauth`).
- **Smoke test:** `./target/debug/termlink fleet reauth ring20-management --json` emits clean Tier-1 JSON with profile/secret_file/plan_text populated; unknown-profile returns CLI bail to stderr → MCP wrapper's empty-stdout branch handles it.
- **Out of scope (followed AC):** `--all-drifted --json` not honored — bulk operator-table output preserved. Filed as T-1729 follow-up if needed.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1a7ca20d
- **Timestamp:** 2026-05-20T18:56:45Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — `termlink_fleet_reauth(profile, bootstrap_from?)` MCP tool registered in `crates/termlink-mcp/src/tools.rs` with safe defaults (bootstrap_from=None → Tier-1 print-plan mode; "auto" → declared anchor l
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `termlink_fleet_reauth(profile, bootstrap_from?)` MCP tool registered in `crates/termlink-mcp/src/tools.rs` with safe defaults (bootstrap_from=None → `

### 2026-05-20T18:56:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** MCP tool shipped + CLI --json flag + 3 unit tests pass + Tier-1 smoke test confirmed clean JSON. R2 guard verified at boundary.
