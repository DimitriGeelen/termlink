---
id: T-1476
name: "fleet doctor: annotate (unknown) caller as pre-T-1409 residue"
description: >
  fleet doctor: annotate (unknown) caller as pre-T-1409 residue

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T11:08:47Z
last_update: 2026-05-04T11:08:47Z
date_finished: null
---

# T-1476: fleet doctor: annotate (unknown) caller as pre-T-1409 residue

## Context

The hub-side audit log records `from`/`peer_pid`/`peer_addr` per call.
Before T-1409 (peer_addr fix, landed 2026-04-29), TCP/TLS calls without
an explicit `from` parameter ended up bucketed as `(unknown)` because
the peer address wasn't being threaded through. Post-T-1409, every TCP
call carries `addr:<ip>` and Unix calls carry `pid:<n>`.

`fw fleet doctor --legacy-usage` aggregates audit-log counters into a
"Top callers" list. On the local fleet right now, `(unknown)` accounts
for >99% of historical residue (e.g. 20360× of 20381× fleet-wide). It
all predates 2026-04-29 — the verdict logic correctly says "no live
legacy callers" — but the operator still sees a giant `(unknown)` line
without context about WHY it's there or what to do with it.

This task adds a one-line annotation under the top_callers block when
`(unknown)` appears: "(unknown) entries are pre-T-1409 residue; the
attribution gap was closed 2026-04-29 — these counts cannot be acted
on individually." Annotation fires once per hub block and once for the
fleet-wide aggregate, never repeated per occurrence.

## Acceptance Criteria

### Agent
- [x] Per-hub `top_callers` annotation fires when `(unknown)` is among that hub's top callers — verified live: annotation appears under local-test, ring20-management, workstation-107-public hubs (3 hits)
- [x] Fleet-wide aggregate annotation fires once after `Top callers (fleet-wide)` block — verified live (after `20360× (unknown)`)
- [x] Annotation text names watermark (2026-04-29) + action ("track recent traffic via 'ACTIVE' tag instead")
- [x] No annotation printed when no `(unknown)` appears — implementation gates on `seen_unknown` flag set inside the loop
- [x] `--json` mode suppressed — annotation is `eprintln!` inside the `!json` branch (the existing render block); JSON output verified clean (no "pre-T-1409" string in JSON dump)
- [x] `cargo build -p termlink` succeeds (7.50s)
- [x] Smoke confirms annotation lines visible in human-readable fleet doctor output (3 per-hub + 1 fleet-wide)

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
target/debug/termlink fleet doctor --legacy-usage > /tmp/t1476-fd.txt 2>&1; grep -q "pre-T-1409" /tmp/t1476-fd.txt

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

### 2026-05-04T11:08:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1476-fleet-doctor-annotate-unknown-caller-as-.md
- **Context:** Initial task creation
