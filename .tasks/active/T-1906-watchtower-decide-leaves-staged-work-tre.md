---
id: T-1906
name: "Watchtower decide leaves staged work-tree artifacts stranded — investigate auto-commit vs refuse-while-dirty"
description: >
  Inception: Watchtower decide leaves staged work-tree artifacts stranded — investigate auto-commit vs refuse-while-dirty

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T09:53:36Z
last_update: 2026-06-01T09:54:45Z
date_finished: null
---

# T-1906: Watchtower decide leaves staged work-tree artifacts stranded — investigate auto-commit vs refuse-while-dirty

## Problem Statement

**The question:** When Watchtower's inception-decision form moves a task from
`active/` to `completed/`, why are the operator's uncommitted working-tree edits
(the research artifact + task-file edits that produced the decision) left
stranded — and what is the right structural fix?

**Live incident on 2026-06-01 (T-1904 census):**

1. Agent completed Steps 1-5 of the T-1904 census. Wrote final recommendation +
   matrix evidence to `.tasks/active/T-1904-*.md` (+52 lines, staged) and
   `docs/reports/T-1904-mcp-vs-direct-session.md` (+258 lines, staged).
2. Inception 2-commit cap reached — could not commit until decision recorded.
3. Operator clicked GO in Watchtower's `/inception/T-1904` form.
4. Watchtower's decide handler recorded the decision, moved the file from
   `active/` to `completed/`, generated `.context/episodic/T-1904.yaml`.
   **Did NOT commit** the agent's pre-existing staged Edits.
5. Agent tried to commit those staged edits with a `T-1904: ...` message. The
   `check-active-task` hook parsed the commit message, found `T-1904`, looked
   up its state, found it was no longer in `.tasks/active/`, and refused the commit.
6. Workaround: file wrap-up build task `T-1905`, write ACs, commit under T-1905
   referencing T-1904 in the body. Two extra commits for paperwork.

**Cost of this incident:** 3-4 extra agent turns + cognitive load + a trip
through the G-020 build readiness gate (which required writing ACs for a task
whose sole purpose was a `git commit`).

**Cost if recurring:** Every future Watchtower-driven inception decision that
lands while the agent has uncommitted supporting work hits the same wall. The
framework defaults push toward this pattern — research artifacts updated
incrementally during the census, decision recorded at the end. Current design
**structurally penalises success**.

**For whom:** TermLink agents (current + future) and operators who use
Watchtower to record inception decisions.

**Why now:** Defect observed live; workaround in `f6dac995` + `2c43eef3`.
Fixing now is cheaper than later — episodic memory of the live incident is
fresh and usable as the empirical baseline for the chosen fix.

## Assumptions

- **A1.** Watchtower's decide handler calls `fw inception decide` (or its
  Python equivalent in `.agentic-framework/web/...`) which performs the
  file-move + episodic-write but not a `git commit` of working-tree state.
- **A2.** Neither the decide handler nor `fw inception decide` checks for
  unstaged/staged edits to the task file or research artifact before
  performing the move. No precondition enforcement.
- **A3.** The `check-active-task` PreToolUse hook (likely `fw hook
  check-active-task`) parses commit messages for `T-XXXX` task IDs and
  verifies each is in `.tasks/active/`. It rejects commits whose message
  references a task that has just been moved to `completed/`, even when the
  diff IS the supporting evidence for that task's just-recorded decision.
- **A4.** No "tolerate completed-task ref for ≤ K minutes after close"
  carveout exists in the hook.

## Exploration Plan

**Methodology:** Static trace of the decide path, then design + evaluation of
the three candidate fixes against the 2026-06-01 incident as empirical baseline.
Time-box: ~60 min, one session.

### Spikes

1. **Locate decide handler (5 min).** Find the Watchtower route in
   `.agentic-framework/web/app.py` that POSTs the GO/NO-GO decision. Identify
   what it delegates to.

2. **Locate file-move + episodic-write code (10 min).** Trace from
   `fw inception decide` into the sh/py implementation. Identify the
   `active → completed` move site and the episodic-YAML write.

3. **Locate commit-message validator (10 min).** Find the hook
   (`fw hook check-active-task` or equivalent) and read its task-ID extraction
   regex + state-check logic.

4. **Survey the three candidate fixes (15 min):**
   - **(a) auto-commit-while-clean.** Decide handler stages + commits
     working-tree edits to the task file and `docs/reports/T-XXX-*` before
     performing the move. Closes the gap proactively.
   - **(b) refuse-while-dirty.** Decide handler refuses to proceed if working
     tree has uncommitted edits to either. Watchtower UI shows "Cannot decide —
     you have N uncommitted edits to T-XXX. Click [Commit Now] / [Discard] /
     [Cancel Decide]." Operator explicitly resolves the dirty state.
   - **(c) tolerate-completed-task-ref-on-followup.** The `check-active-task`
     hook accepts commits referencing a task completed within the last K
     minutes. Preserves "nothing gets done without an active task" because
     the work IS for the just-completed task — just landed late.

5. **Decide which fix(es) compose (10 min).** Not mutually exclusive. (a) or
   (b) prevents stranding; (c) recovers when it happens anyway. Expected
   output: one primary + at most one as defence-in-depth.

6. **Distill (10 min).** Write Recommendation + propose follow-up build tasks
   for the chosen fix(es).

## Technical Constraints

- **Read-only investigation.** No code changes during inception phase.
- **No Watchtower restart.** The running instance must continue to serve.
  Investigation reads source; modifications come in the build phase.
- **No changes to `fw` global state.** Don't touch focus.yaml; don't re-open
  T-1904 or T-1905.

## Scope Fence

**IN scope:**
- Watchtower → fw inception decide → file-move pipeline
- check-active-task hook's commit-message parsing
- The three candidate fixes above
- Recommendation of which fix(es) to ship

**OUT of scope:**
- Re-litigating the inception 2-commit cap (separate concern)
- Re-litigating G-020 build readiness (T-1905 having to write ACs was a
  consequence, not the root cause)
- Touching CLAUDE.md or framework docs in inception phase
- Building the fix (separate build task after GO decision)

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO — ship the primary fix as a build task** if:
- One of the three candidate fixes is proportional to recurrence likelihood
  + incident cost (3-4 turns + G-020 wrap-up paperwork per occurrence), AND
- Cost of building + testing is bounded (<1 day), AND
- The fix does not regress P-002 traceability, G-020 build readiness, or
  the inception 2-commit cap.

**NO-GO — leave the workaround in place** if:
- The wrap-up-task workaround is judged proportional to recurrence likelihood,
  AND
- All three candidate fixes carry unacceptable side effects (e.g. risk
  regressing the active-task gate broadly).

**GO-COMPOSED — ship two fixes (defence in depth)** if:
- (a)/(b) is right as the primary structural fix BUT
- (c) is cheap enough to ship as a safety net for race conditions / parallel
  sessions / future decide-paths we haven't anticipated.

**DEFER — partial findings** if:
- Investigation reveals additional decide-paths beyond inception (e.g.
  `/review` path, batch-close) that share the same gap and require broader
  scope.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:**

Filing-state PENDING-EVIDENCE. Need to investigate which path of Watchtower's decide handler moves the task file, why it does not stage/commit the operator's working-tree edits, what hooks then block the agent from landing those edits under the original task ID, and which of (a) auto-commit-while-clean (b) refuse-while-dirty (c) tolerate-completed-task-ref-on-followup-commit best closes the gap. Recommendation will be filed after investigation.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-01T09:54:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
