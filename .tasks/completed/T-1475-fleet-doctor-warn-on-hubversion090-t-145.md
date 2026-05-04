---
id: T-1475
name: "fleet doctor: warn on hub_version=0.9.0 (T-1458 follow-up)"
description: >
  fleet doctor: warn on hub_version=0.9.0 (T-1458 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs]
related_tasks: []
created: 2026-05-04T11:04:46Z
last_update: 2026-05-04T11:07:37Z
date_finished: 2026-05-04T11:07:37Z
---

# T-1475: fleet doctor: warn on hub_version=0.9.0 (T-1458 follow-up)

## Context

T-1458 follow-up. T-1458 added a `build.rs` to the termlink-hub crate so
`env!("CARGO_PKG_VERSION")` resolves to the git-derived version (e.g.
`0.9.1859`) instead of the workspace-static `0.9.0`. Fix verified in an
isolated test hub. But every long-lived production hub is still running
its pre-T-1458 binary and continues to return `0.9.0` via `hub.version`.
Operators running `fleet doctor` see "version: 0.9.0" across the fleet
and can't distinguish "binary is fresh, just didn't restart" from
"binary is genuinely old".

T-1458's own RCA recommended this follow-up. This task implements it:
surface a `[WARN] hub_version=0.9.0` line under each hub's [PASS] block
and add `version_stale: true` to the JSON shape.

## Acceptance Criteria

### Agent
- [x] `cmd_fleet_doctor` flags hubs reporting `hub_version="0.9.0"` with `version_stale: true` in JSON output (verified: 5 production hubs flagged)
- [x] Human-readable output renders a `[WARN] hub_version=0.9.0 — running binary predates T-1458` line under the affected hub's [PASS] block (verified live)
- [x] Warning text names the date watermark (2026-05-03) and the actionable next step ("Restart with a newer binary")
- [x] Hubs with git-derived versions emit no warning — code path: `let version_stale = hub_version == "0.9.0";` is exact-match, anything else (e.g. `0.9.1701`, `unknown`) is non-stale
- [x] Hubs reporting `unknown` emit no warning (verified by exact-match condition)
- [x] `cargo build -p termlink` succeeds (7.60s)
- [x] Smoke: live local hub returns 5 stale hubs in JSON (sample: laptop-141, version_stale=true)
- [x] JSON shape: `version_stale` only inserted when true; non-stale hubs do NOT carry the field (verified — 1 hub without it)
- [x] No regressions in fleet_doctor unit tests (0 fleet_doctor tests existed, 633 other tests still filter out cleanly)

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

cargo build -p termlink
target/debug/termlink fleet doctor --json 2>/dev/null > /tmp/t1475-fd.json && python3 -c "import json; d=json.load(open('/tmp/t1475-fd.json')); assert any(h.get('version_stale') for h in d.get('hubs',[])), 'expected at least one stale hub against pre-T-1458 prod fleet'"
target/debug/termlink fleet doctor > /tmp/t1475-fd.txt 2>&1; grep -q 'hub_version=0.9.0' /tmp/t1475-fd.txt

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

### 2026-05-04T11:04:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1475-fleet-doctor-warn-on-hubversion090-t-145.md
- **Context:** Initial task creation

### 2026-05-04T11:07:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
