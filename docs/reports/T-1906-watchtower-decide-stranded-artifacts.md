# T-1906 — Watchtower decide leaves staged work-tree artifacts stranded

**Status:** Inception, exploration phase.
**Filed:** 2026-06-01
**Owner:** human (decision authority); claude-code (advisory)
**Recommendation at filing:** PENDING-EVIDENCE (filing-state)

This is the live research artifact for T-1906. Updated incrementally as
spikes produce findings.

## The question (one sentence)

When Watchtower's inception-decision form moves a task from `active/` to
`completed/`, why are the operator's uncommitted working-tree edits (the
research artifact + task-file edits that produced the decision) left
stranded — and what is the right structural fix?

## Why it matters

Live incident on 2026-06-01 during T-1904 census:

1. Agent completed Steps 1-5; wrote final recommendation + matrix
   evidence to `.tasks/active/T-1904-*.md` (+52 lines) and
   `docs/reports/T-1904-mcp-vs-direct-session.md` (+258 lines). Staged.
2. Inception 2-commit cap reached. Cannot commit until decision.
3. Operator clicked **GO** in Watchtower's `/inception/T-1904` form.
4. Watchtower's decide handler recorded the decision, moved the task
   to `completed/`, generated `.context/episodic/T-1904.yaml`. Did NOT
   commit the agent's pre-existing staged Edits.
5. Agent tried to commit with `T-1904: ...` message. The
   `check-active-task` hook parsed the commit message, found `T-1904`,
   looked up its state, found it was no longer in `.tasks/active/`,
   and refused.
6. Workaround: file wrap-up build task T-1905, write ACs, commit under
   T-1905 referencing T-1904 in the body. Two extra commits for
   paperwork; episodic memory for T-1905 was created for content that
   really belongs to T-1904.

**Cost of this incident:** 3-4 extra agent turns + cognitive load + a
trip through the G-020 build readiness gate.

**Cost if recurring:** Every future Watchtower-driven inception decision
that lands while the agent has uncommitted supporting work hits the same
wall. The framework defaults push toward this pattern — research
artifacts updated incrementally + decision recorded at the end. Current
design **structurally penalises success**.

## Hypotheses (filed as assumptions A-032 .. A-035)

- **A1.** Watchtower's decide handler calls `fw inception decide` (or
  its Python equivalent) which performs the file-move + episodic-write
  but not a `git commit` of working-tree state.
- **A2.** Neither the decide handler nor `fw inception decide` checks
  for unstaged/staged edits before performing the move. No precondition
  enforcement.
- **A3.** The `check-active-task` PreToolUse hook parses the commit
  message for `T-XXXX` task IDs and verifies each is in
  `.tasks/active/`. It rejects commits whose message references a task
  that has just been moved to `completed/`, even when the diff IS the
  supporting evidence for that task's just-recorded decision.
- **A4.** No "tolerate completed-task ref for ≤ K minutes after close"
  carveout exists — the natural window in which supporting commits
  would land.

## Exploration Plan

**Methodology:** Static + dynamic trace of the decide path, then design
+ evaluation of the three candidate fixes. Time-box: ~60 min, one
session.

### Spikes

1. **Locate decide handler** (5 min). Find the Watchtower route in
   `.agentic-framework/web/app.py` that POSTs the GO/NO-GO decision.
   Identify what it delegates to.

2. **Locate file-move + episodic-write code** (10 min). Trace from
   `fw inception decide` into the sh/py implementation. Identify the
   `active → completed` move site and the episodic-YAML write.

3. **Locate commit-message validator** (10 min). Find the hook
   (`fw hook check-active-task` or equivalent) and read its task-ID
   extraction regex + state-check logic.

4. **Survey the three candidate fixes** (15 min):
   - **(a) auto-commit-while-clean.** Decide handler stages + commits
     working-tree edits to the task file and `docs/reports/T-XXX-*`
     before performing the move.
   - **(b) refuse-while-dirty.** Decide handler refuses to proceed if
     working tree has uncommitted edits to either; Watchtower UI shows
     "Cannot decide — you have N uncommitted edits to T-XXX. Click
     [Commit Now] / [Discard] / [Cancel Decide]."
   - **(c) tolerate-completed-task-ref-on-followup.** The
     `check-active-task` hook accepts commits referencing a task
     completed within the last K minutes. Preserves "nothing gets done
     without an active task" — the work IS for the just-completed task.

5. **Decide which fix(es) compose well** (10 min). Not mutually
   exclusive. (a) or (b) prevents stranding; (c) recovers when it
   happens anyway. Expected output: one primary + at most one as
   defence-in-depth.

6. **Distill** (10 min). Write Recommendation + propose follow-up
   build tasks.

## Technical Constraints

- Read-only investigation; no code changes during inception phase.
- No Watchtower restart. The running instance must continue to serve.
- No changes to `fw` global state; don't touch focus.yaml or re-open
  T-1904/T-1905.

## Scope Fence

**IN scope:**
- Watchtower → fw inception decide → file-move pipeline
- check-active-task hook's commit-message parsing
- The three candidate fixes
- Recommendation

**OUT of scope:**
- Re-litigating the inception 2-commit cap (separate concern)
- Re-litigating G-020 build readiness (T-1905 having to write ACs was
  a consequence, not the root cause)
- Touching CLAUDE.md or framework docs in inception phase
- Building the fix (separate build task after GO decision)

## Go/No-Go Criteria

**GO — ship the primary fix as a build task** if:
- One candidate is proportional to recurrence likelihood + incident cost
- Cost of building + testing is bounded (<1 day)
- Fix does not regress P-002 traceability, G-020 build readiness, or
  the inception 2-commit cap

**NO-GO — leave the workaround in place** if:
- Wrap-up-task workaround is judged proportional to recurrence
- All three candidates carry unacceptable side effects

**DEFER — partial findings** if:
- Investigation reveals additional decide-paths beyond inception
  (e.g. `/review` path, batch-close) that share the same gap and
  require broader scope

## Recommendation

**PENDING-EVIDENCE (filing-state)** — Investigation not yet executed.

Execution awaits operator sign-off on the inception scope. The plan is
self-contained enough that another session can execute it without
re-litigating scope.

## Dialogue Log

### 2026-06-01T — Operator: file inception
- **Question posed:** "please incept investigation and fix [framework
  gap]: Watchtower's decide path moved T-1904 to completed/ but did
  not auto-commit the staged supporting artifacts. The hook then
  refused to accept commits referencing the now-completed task ID,
  forcing the wrap-up-task workaround. Should either
  auto-commit-while-clean or refuse-while-dirty."
- **Agent action:** Filed T-1906, drafted Problem Statement /
  Assumptions / Exploration Plan / Scope Fence / Go-No-Go from the
  live 2026-06-01 incident as empirical baseline. Created this
  research artifact (C-001).
