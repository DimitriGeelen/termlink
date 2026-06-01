---
id: T-1908
name: "check-active-task Gate 2 grace-period: tolerate completed-task ref within 30min (T-1906 defence-in-depth)"
description: >
  check-active-task Gate 2 grace-period: tolerate completed-task ref within 30min (T-1906 defence-in-depth)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T10:12:14Z
last_update: 2026-06-01T10:12:14Z
date_finished: null
---

# T-1908: check-active-task Gate 2 grace-period: tolerate completed-task ref within 30min (T-1906 defence-in-depth)

## Context

T-1906 GO-COMPOSED defence-in-depth fix. Complements T-1907 (primary
auto-commit fix). Catches edge cases where the primary fix doesn't apply:
parallel agent sessions, batch-decide paths, `fw inception decide` invoked
from CLI without a clean tree, race conditions.

`check-active-task.sh` Gate 2 (G-013, line 308-321) currently rejects ANY
commit whose focused task is not in `.tasks/active/`. This task adds a
bounded carveout: if the task is in `completed/` AND its frontmatter
`date_finished` is within the last 30 minutes, allow the commit (with a
stderr NOTE).

Full evaluation: `docs/reports/T-1906-watchtower-decide-stranded-artifacts.md`
Composition-analysis section.

## Acceptance Criteria

### Agent
- [ ] `check-active-task.sh` Gate 2 (G-013, line 308-321) reads `date_finished`
      from the frontmatter when `find_task_file "$CURRENT_TASK" active` returns
      empty, BEFORE blocking
- [ ] If `date_finished` is within the last 30 minutes (1800 seconds), allow
      the commit and emit one stderr NOTE: `NOTE: T-XXX completed Ns ago — grace-period carveout (T-1908). Tier-3-pre-approved.`
- [ ] Otherwise (older than 30 min OR `date_finished` malformed OR missing):
      continue blocking with the existing G-013 error message
- [ ] Grace-period window is a tunable env var `FW_COMPLETED_TASK_GRACE_SECS`
      (default 1800). Allows adjustment without code change.
- [ ] Live replay: with T-1907 disabled, re-execute T-1904 scenario; confirm
      that within 30 min of close, `git commit -m "T-1904: ..."` lands; after
      30 min, blocking resumes.
- [ ] Audit log: each grace-period activation appends one line to
      `.context/working/.grace-period-bypass.log` (NDJSON: ts, task, secs_ago,
      command-preview)
- [ ] No regression of P-002 traceability for tasks NOT recently completed
      (steady-state behaviour stays identical)

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

bash -n .agentic-framework/agents/context/check-active-task.sh
grep -q "FW_COMPLETED_TASK_GRACE_SECS\|grace-period" .agentic-framework/agents/context/check-active-task.sh
grep -q "date_finished" .agentic-framework/agents/context/check-active-task.sh

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

### 2026-06-01T10:12:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1908-check-active-task-gate-2-grace-period-to.md
- **Context:** Initial task creation
