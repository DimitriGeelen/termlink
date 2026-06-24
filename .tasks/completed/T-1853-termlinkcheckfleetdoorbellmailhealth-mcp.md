---
id: T-1853
name: "termlink_check_fleet_doorbell_mail_health MCP wrapper (T-1845/T-1831 follow-on)"
description: >
  termlink_check_fleet_doorbell_mail_health MCP wrapper (T-1845/T-1831 follow-on)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-28T21:26:08Z
last_update: 2026-05-28T21:28:19Z
date_finished: 2026-05-28T21:28:19Z
---

# T-1853: termlink_check_fleet_doorbell_mail_health MCP wrapper (T-1845/T-1831 follow-on)

## Context

T-1831 ships the loopback canary as a shell script, T-1845 added per-hub
timeout wrap. The discovery-triangle MCP layer has wrappers for "who's
there?" (T-1839), "is rail real-used?" (T-1847), "what's been said?"
(T-1852), but no wrapper for the rail-can-carry-a-turn canary. Agents
investigating a fleet need to ask "would a real doorbell+mail turn work
right now?" without shelling out.

## Acceptance Criteria

### Agent
- [x] `CheckFleetDoorbellMailHealthParams` struct added to `crates/termlink-mcp/src/tools.rs` with fields `hubs_file: Option<String>`, `no_heartbeat: Option<bool>`, `timeout_secs: Option<u64>` and `#[derive(Deserialize, JsonSchema)]`
- [x] `#[tool] async fn termlink_check_fleet_doorbell_mail_health` method added on the existing impl, named exactly `termlink_check_fleet_doorbell_mail_health`, reusing `resolve_t1836_script("check-fleet-doorbell-mail-health.sh")` and `run_t1836_subprocess` per the T-1847/T-1852 pattern
- [x] Tool description references T-1831 + T-1845 lineage and distinguishes it from `termlink_fleet_doctor` and `termlink_fleet_adoption_snapshot`
- [x] `cargo check -p termlink-mcp` exits 0
- [x] `cargo build -p termlink-mcp` exits 0

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
grep -q "termlink_check_fleet_doorbell_mail_health" crates/termlink-mcp/src/tools.rs
grep -q "CheckFleetDoorbellMailHealthParams" crates/termlink-mcp/src/tools.rs
cargo check -p termlink-mcp 2>&1 | tail -5 | grep -qE "Finished|warning"
cargo build -p termlink-mcp 2>&1 | tail -5 | grep -qE "Finished|warning"

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

### 2026-05-28T21:26:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1853-termlinkcheckfleetdoorbellmailhealth-mcp.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f0c8372f
- **Timestamp:** 2026-05-28T21:28:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T21:28:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
