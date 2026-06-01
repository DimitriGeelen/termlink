---
id: T-1907
name: "Auto-commit scope-related edits in do_inception_decide (T-1906 primary fix)"
description: >
  Auto-commit scope-related edits in do_inception_decide (T-1906 primary fix)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T10:11:25Z
last_update: 2026-06-01T10:11:25Z
date_finished: null
---

# T-1907: Auto-commit scope-related edits in do_inception_decide (T-1906 primary fix)

## Context

T-1906 GO-COMPOSED primary fix. After `update-task.sh` moves the inception
task from `active/` to `completed/` (via `do_inception_decide`), the agent's
pre-existing staged edits to the task file and `docs/reports/T-XXX-*`
artifact are left stranded — subsequent commits referencing T-XXX are blocked
by `check-active-task.sh` Gate 2 (G-013). This fix makes
`do_inception_decide` auto-commit scope-related edits in one go.

Full trace + evaluation: `docs/reports/T-1906-watchtower-decide-stranded-artifacts.md`.

## Acceptance Criteria

### Agent
- [ ] `do_inception_decide` (in `.agentic-framework/lib/inception.sh`) stages
      uncommitted edits matching `.tasks/active/T-XXX-*.md` and
      `docs/reports/T-XXX-*` (glob-bounded to the inception's task ID) BEFORE
      calling `update-task.sh --status work-completed`
- [ ] After `update-task.sh` completes (which adds the active→completed rename
      to the stage), `do_inception_decide` runs ONE `git commit -m
      "T-XXX: decision recorded — <DECISION> (<rationale-first-line>)"`
- [ ] If `git add` or `git commit` fails: abort decide BEFORE the
      `update-task.sh` move (no half-applied state). Surface error via
      Watchtower's existing `?error=` query param path.
- [ ] Behavior is OFF-by-default with an opt-in env var
      `FW_INCEPTION_AUTO_COMMIT=1`, OR ON-by-default with an opt-out env var
      `FW_INCEPTION_AUTO_COMMIT=0` — decide which during build phase. Default
      preserves current behavior on day 1; flip the default after one-week
      soak.
- [ ] Live replay: re-execute the T-1904 scenario (uncommitted Edits +
      decide-via-Watchtower) end-to-end with the new code; confirm only ONE
      commit lands and references T-XXX with both the rename and the edits.
- [ ] No regression of P-002 traceability, G-020 build readiness, or
      inception 2-commit cap (build cap-aware test that confirms a 2-commit
      capped inception still decides correctly via auto-commit — the
      auto-commit is the 3rd inception commit, which is permitted post-decide).

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

bash -n .agentic-framework/lib/inception.sh
grep -q "git add.*\.tasks/active/T-.*\\|docs/reports/T-" .agentic-framework/lib/inception.sh
grep -q "git commit -m.*decision recorded" .agentic-framework/lib/inception.sh

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

### 2026-06-01T10:11:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1907-auto-commit-scope-related-edits-in-doinc.md
- **Context:** Initial task creation
