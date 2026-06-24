---
id: T-1823
name: "termlink_fleet_secrets_audit MCP — add check_drift param (T-1822 + T-1821 follow-up)"
description: >
  termlink_fleet_secrets_audit MCP — add check_drift param (T-1822 + T-1821 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T08:07:10Z
last_update: 2026-05-28T08:09:34Z
date_finished: 2026-05-28T08:09:34Z
---

# T-1823: termlink_fleet_secrets_audit MCP — add check_drift param (T-1822 + T-1821 follow-up)

## Context

T-1822 added `--check-drift <PATH>` to the CLI `fleet secrets-audit` verb.
T-1821 shipped the MCP parity for the original T-1820 surface. This task
closes the silent-strip gap (PL-167 / PL-172): MCP-side surface must mirror
the CLI's flag set, otherwise agent callers can't reach the new drift
detection — and the next G-011 rotation incident goes undetected via the
agent surveillance path that's supposed to catch exactly this class of bug.

Mechanical: add one optional param, forward as one optional `--check-drift
<PATH>` subprocess arg, ship.

## Acceptance Criteria

### Agent
- [x] `FleetSecretsAuditParams` gains `check_drift: Option<String>` field with doc-comment pointing at T-1822
- [x] `termlink_fleet_secrets_audit` tool body appends `--check-drift <PATH>` to subprocess args when set
- [x] Tool description updated to mention the drift-check capability + the `authoritative` and `authoritative_error` JSON envelope fields
- [x] 2 new unit tests: accepts check_drift, accepts all fields together (proves field is optional + no field-ordering regression)
- [x] `cargo check -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp fleet_secrets_audit` passes (4 existing + 2 new = 6/6)

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
cd /opt/termlink && cargo check -p termlink-mcp 2>&1 | tail -5
cd /opt/termlink && cargo test -p termlink-mcp fleet_secrets_audit 2>&1 | tail -10

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

## Recommendation

**GO** — ship as-is.

Closes the PL-167 silent-strip gap between T-1822 CLI and T-1821 MCP.
Without this, agents calling `termlink_fleet_secrets_audit` could not
reach the drift-detection path that closes G-011 item 1 — the next
PL-041-style silent rotation would land back in the "no automated
detection" hole.

6/6 unit tests pass. Subprocess argument forwarding is the smallest
possible change; the description string is updated so MCP catalog
inspection (used by agent toolchain discovery) advertises the new
shape including `authoritative` envelope fields.

This rounds out the T-1820 → T-1821 → T-1822 → T-1823 arc:
- T-1820 ships CLI verb
- T-1821 ships MCP parity for the verb
- T-1822 extends CLI with `--check-drift`
- T-1823 extends MCP with the same parameter

CLI and MCP surfaces are now in lockstep.

## Updates

### 2026-05-28T08:07:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1823-termlinkfleetsecretsaudit-mcp--add-check.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-a1e11546
- **Timestamp:** 2026-05-28T08:09:51Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T08:09:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
