---
id: T-1627
name: "T-1166 cut-flip projection report (2026-05-10) — operator authorization aid"
description: >
  T-1166 cut-flip projection report (2026-05-10) — operator authorization aid

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T19:00:16Z
last_update: 2026-05-06T19:02:11Z
date_finished: 2026-05-06T19:02:11Z
---

# T-1627: T-1166 cut-flip projection report (2026-05-10) — operator authorization aid

## Context

T-1166 cut is gated on `legacy_pct < 1.0%` over a 7-day window. Today
(2026-05-06) cut_ready=False with legacy_pct=2.904% (4552 legacy / 156714
total events). The prior session's "0.018%" framing was wrong — the gate
is about percentage of total RPC traffic, not absolute count, and it's
currently 3× above threshold.

Per-day analysis (from `/var/lib/termlink/rpc-audit.jsonl`) shows legacy
emissions essentially stopped on 2026-05-04 (post-T-1418 deploy). Only 5
total emissions across May 4-6 (vs 1389 on May 2). The remaining 4451
in-window are historical residue that rolls out as the 7d window slides.

Goal: produce a one-shot operator-readable artifact with: (a) current
state, (b) per-day decay, (c) projection table, (d) verification command,
(e) authorization runbook. Saves the operator from redoing this math
manually on each check-in.

## Acceptance Criteria

### Agent
- [x] `docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md` exists
- [x] Report contains the per-day decay table (8 rows: Apr 29 - May 6)
- [x] Report contains the projection table (today + 7 days forward) with cut-flip date highlighted
- [x] Report contains the operator's re-verification command (`fw metrics api-usage --cut-ready --json`)
- [x] T-1166 task body has a 1-2 line pointer to the report under a `## Cut Projection (2026-05-06)` heading

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

test -f docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md
grep -q '2026-05-04' docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md
grep -q '2026-05-10' docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md
grep -q -- '--cut-ready' docs/reports/T-1627-t1166-cut-flip-projection-2026-05-06.md
grep -q 'Cut Projection' .tasks/active/T-1166-t-11559-retire-legacy-eventbroadcast--in.md

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

## Updates

### 2026-05-06T19:00:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1627-t-1166-cut-flip-projection-report-2026-0.md
- **Context:** Initial task creation

### 2026-05-06T19:02:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
