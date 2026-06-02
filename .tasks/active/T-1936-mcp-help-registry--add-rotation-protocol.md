---
id: T-1936
name: "MCP help registry — add rotation-protocol family (tofu_*, hub_*, fleet_*)"
description: >
  Add the rotation-protocol tool family to termlink_help registry so LLM consumers can discover them. Currently tofu_clear/list/verify, hub_probe/fingerprint/export_secret/restart, and the entire fleet_* family (verify/doctor/history/status/bootstrap_check/reauth/secrets_audit/adoption_snapshot) are registered as MCP tools but invisible via termlink_help. LLM agents cannot help with auth-rotation recovery (documented at length in CLAUDE.md) without discovery. Out of scope: agent_* family (100+ tools, separate slice).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T22:55:51Z
last_update: 2026-06-02T22:55:51Z
date_finished: null
---

# T-1936: MCP help registry — add rotation-protocol family (tofu_*, hub_*, fleet_*)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `tofu` category exists in `termlink_help` registry with entries for `termlink_tofu_list`, `termlink_tofu_verify`, `termlink_tofu_clear`
- [ ] `fleet` category exists with entries for `termlink_fleet_verify`, `termlink_fleet_doctor`, `termlink_fleet_history`, `termlink_fleet_status`, `termlink_fleet_bootstrap_check`, `termlink_fleet_reauth`, `termlink_fleet_secrets_audit`, `termlink_fleet_adoption_snapshot`
- [ ] `hub` category extended with `termlink_hub_probe`, `termlink_hub_fingerprint`, `termlink_hub_export_secret`, `termlink_hub_restart`
- [ ] Unknown-category error message lists `tofu` and `fleet` in its enumeration
- [ ] `cargo build --release -p termlink-mcp` is warning-free

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
! cargo build --release -p termlink-mcp 2>&1 | grep -q "warning:"
grep -q '"tofu", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"fleet", vec!' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_hub_probe"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_fleet_doctor"' crates/termlink-mcp/src/tools.rs
grep -q '"termlink_tofu_clear"' crates/termlink-mcp/src/tools.rs

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

### 2026-06-02T22:55:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1936-mcp-help-registry--add-rotation-protocol.md
- **Context:** Initial task creation
