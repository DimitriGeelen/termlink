---
id: T-1825
name: "termlink_fleet_secrets_audit MCP — add target_cache param (T-1824 follow-up)"
description: >
  termlink_fleet_secrets_audit MCP — add target_cache param (T-1824 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T08:26:40Z
last_update: 2026-05-28T08:29:30Z
date_finished: 2026-05-28T08:29:30Z
---

# T-1825: termlink_fleet_secrets_audit MCP — add target_cache param (T-1824 follow-up)

## Context

T-1824 added `--target-cache <PATH>` to CLI. PL-167 silent-strip rule:
MCP must mirror the CLI flag set or agent callers can't reach the new
narrowing. Mechanical follow-up — add one optional param, forward one
optional `--target-cache <PATH>` subprocess arg.

## Acceptance Criteria

### Agent
- [x] `FleetSecretsAuditParams` gains `target_cache: Option<String>` field
- [x] Subprocess args append `--target-cache <PATH>` when set
- [x] Tool description mentions narrowing capability + new envelope field
- [x] 1 new unit test verifies target_cache deserialization; existing all-fields test extended to assert the new field too
- [x] `cargo check -p termlink-mcp` passes
- [x] `cargo test -p termlink-mcp fleet_secrets_audit` passes (6 existing + 1 new = 7/7)

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

Completes the T-1820→T-1821→T-1822→T-1823→T-1824→T-1825 arc. CLI and MCP
surfaces are in lockstep across all flags:

| CLI flag | MCP param | Effect |
|---|---|---|
| `--dir <PATH>` | `dir` | scan-dir override |
| `--check-drift <PATH>` | `check_drift` | enable broad-mode drift detection (T-1822/T-1823) |
| `--target-cache <PATH>` | `target_cache` | narrow drift to one cache (T-1824/T-1825) |
| `--json` | (always JSON) | structured output |

7/7 MCP tests pass. The arc closes G-011 items 1 + 4 at the surface
level (operator + agent surveillance) — long-term item (IP-keyed cache
deprecation) remains as separate G-011 work.

## Updates

### 2026-05-28T08:26:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1825-termlinkfleetsecretsaudit-mcp--add-targe.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-92829332
- **Timestamp:** 2026-05-28T08:29:46Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T08:29:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
