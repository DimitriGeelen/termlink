---
id: T-1689
name: "MCP exposure: termlink_fleet_bootstrap_check — agent-callable anchor preflight"
description: >
  MCP exposure: termlink_fleet_bootstrap_check — agent-callable anchor preflight

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-18T07:10:39Z
last_update: 2026-05-18T07:23:43Z
date_finished: 2026-05-18T07:23:43Z
---

# T-1689: MCP exposure: termlink_fleet_bootstrap_check — agent-callable anchor preflight

## Context

T-1688 shipped `termlink fleet bootstrap-check` so operators can validate declared `bootstrap_from` anchors at declaration time. T-1687 shipped `termlink_fleet_history` (MCP) so agents can investigate hub flap history. The missing piece in the agent-callable surface: validation. An agent that's diagnosing a flapping hub via `termlink_fleet_history` can't currently ask "would the declared heal actually work?" without shelling out to the CLI.

This task adds `termlink_fleet_bootstrap_check` (MCP) — agent-callable preflight. The CLI helpers `fetch_bootstrap_secret` + `normalize_and_validate_secret_hex` are private to termlink-cli, so the MCP tool either (a) re-implements them or (b) subprocesses the CLI. Subprocessing is simpler and avoids drift; the MCP body becomes a thin JSON-translating wrapper around `termlink fleet bootstrap-check --json` with a tokio timeout.

## Acceptance Criteria

### Agent
- [x] `termlink_fleet_bootstrap_check` MCP tool exists with `FleetBootstrapCheckParams { profile, all, timeout_secs }`; timeout clamped to 1..=120 via `clamp(1, 120)`
- [x] Validates exactly one of `profile`/`all` set; structured error JSON for both/neither — covered by `rejects_both_profile_and_all` + `rejects_neither_profile_nor_all` tests
- [x] Subprocesses `current_exe() fleet bootstrap-check --json [profile|--all]` via `tokio::process::Command` under `tokio::time::timeout`; `kill_on_drop=true` + stdin=null so a hanging interactive ssh anchor can't wedge the MCP server
- [x] Timeout returns structured `{ok: false, verdict: "timeout", error: "timeout after Ns", hint}` JSON (no panic, child killed via `kill_on_drop`)
- [x] 5 unit tests (3 tokio, 2 sync) pin: validation guards (both/neither), valid-input passthrough, params default parsing, clamp bounds
- [x] CLAUDE.md row added immediately under the CLI `fleet bootstrap-check` row, matching the T-1687 MCP-companion pattern
- [x] `cargo check -p termlink-mcp` clean; new tests pass (5/5); full mcp suite green (119/119)

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
cargo check -p termlink-mcp 2>&1 | tail -3 | grep -q "Finished"
cargo test -p termlink-mcp --lib fleet_bootstrap_check 2>&1 | tail -3 | grep -qE "ok\.|test result: ok"

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

### 2026-05-18T07:10:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1689-mcp-exposure-termlinkfleetbootstrapcheck.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-bf8281f7
- **Timestamp:** 2026-05-18T07:23:48Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T07:23:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
