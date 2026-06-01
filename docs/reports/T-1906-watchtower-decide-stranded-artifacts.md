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

## Findings (investigation executed 2026-06-01)

### Spike 1 — Watchtower decide route
`/inception/<task_id>/decide` POST handler at
`.agentic-framework/web/blueprints/inception.py:478` shells out to
`fw inception decide T-XXX <decision> --rationale "..." --from-watchtower`
with a 30s timeout. **No git operations in this route.** A1 partially confirmed.

### Spike 2 — `fw inception decide` implementation
`fw inception decide` → `do_inception_decide` in
`.agentic-framework/lib/inception.sh:384`. It in turn shells to
`update-task.sh T-XXX --status work-completed` (line 696). The move from
`active/` to `completed/` happens at
`.agentic-framework/agents/task-create/update-task.sh:1244`:

```
git -C "$PROJECT_ROOT" mv "$TASK_FILE" "$DEST" 2>/dev/null || mv "$TASK_FILE" "$DEST"
```

T-1523 noted: `git mv` is used so both rename sides stage atomically. **But
neither `update-task.sh` nor `inception.sh` does any `git commit`.** Zero
hits for `git commit` in either file. A1 + A2 fully confirmed.

Side note: `update-task.sh:1260` clears `focus.yaml` on completion
(`current_task: null`). This is why the agent's first post-decide commit
attempt hit a "focus is null" path before reaching the active/ gate.

### Spike 3 — `check-active-task` hook gates
`.agentic-framework/agents/context/check-active-task.sh` has two relevant gates:

- **Gate 1 (line 247–306): Focus-drift detector (T-1730).** Regex pattern 3
  (line 258–259) `(T-[0-9]+):` extracts the task ID from a git commit message.
  If it differs from `$CURRENT_TASK` (focus.yaml), blocks unless `--switch-focus`
  is passed (Tier 2 logged).

- **Gate 2 (line 308–321): Active-task validator (G-013).** Checks
  `find_task_file "$CURRENT_TASK" active` — if the focused task is not in
  `active/`, hard-block with no override flag.

A3 confirmed. The hook does NOT special-case "task is in completed/ but
date_finished is recent."

### Spike 4 — Grace-period / recently-completed concept
```
grep -rEn "grace_period|recently.?completed|completion.?window|just.?completed|date_finished.*ago" \
  .agentic-framework/{lib,agents/context,agents/task-create}
```
→ zero hits. A4 confirmed.

### Live-incident replay (matrix-of-blockers)

Given the 2026-06-01 state immediately post-decide (T-1904 in completed/, focus
nulled, staged Edits referencing T-1904):

| Attempted commit | Gate 1 (focus-drift) | Gate 2 (active task) | Outcome |
|---|---|---|---|
| `git commit -m "T-1904: ..."` with focus=null | passes (no focus to drift from) | hits "focus null" earlier path | blocked |
| `git commit -m "T-1904: ..."` with focus=T-1904 | passes (target==focus) | **BLOCKS** — T-1904 not in active/ | blocked |
| `git commit -m "T-1904: ..."` with focus=T-1166 active | **BLOCKS** — drift T-1904≠T-1166 | (n/a) | blocked |
| `git commit -m "T-1166: housekeeping ..."` with focus=T-1166 | passes | passes | **lands** |

This explains why the wrap-up-task workaround was the minimum-viable escape:
the agent had to invent a NEW active-task ID that the message could reference.

## Candidate fixes — evaluation

**(a) auto-commit-while-clean.** Modify `do_inception_decide` to:
1. `git add` scope-related uncommitted edits before update-task.sh move.
   Scope = `.tasks/active/T-XXX-*.md` + `docs/reports/T-XXX-*`.
2. Let update-task.sh do its `git mv` (already staged).
3. After update-task.sh completes (including episodic write), do one
   `git commit -m "T-XXX: decision recorded — <DECISION> (<rationale-first-line>)"`.

Pros:
- Eliminates stranding by definition. One commit captures decision + evidence.
- No hook changes needed.
- Aligns with operator's mental model: "click GO → it lands."

Cons:
- `do_inception_decide` becomes responsible for git operations (new concern).
- If git fails mid-flow, needs careful rollback (the existing `primary_landed` logic
  in the Watchtower route already wrestles with side-effect failure separation —
  this stays in-scope of that pattern).
- Auto-staging must be scoped to T-XXX-related paths to avoid sweeping in
  unrelated WIP. The glob is bounded so this is a code hygiene concern, not a
  correctness blocker.

**(b) refuse-while-dirty.** Modify Watchtower decide route to:
1. Before shelling to `fw inception decide`, run
   `git diff --quiet HEAD -- .tasks/active/T-XXX-* docs/reports/T-XXX-*`
2. If exit non-zero (dirty): refuse with operator-actionable error.
3. Operator commits first, then re-clicks decide.

Pros:
- Conservative — no git operations in decide path itself.
- Failure mode visible at decide time, not after.

Cons:
- Adds friction the operator's mental model doesn't want.
- **Composes badly with inception 2-commit cap.** If the agent has already used
  both commits and now has staged Edits, the operator can't commit *because of
  the cap*, but also can't decide *because of dirty state*. Genuine deadlock,
  requires manual override. The live T-1904 incident would have hit this exactly.
- Operator has to context-switch between "review and decide" mode and
  "shell-and-commit" mode.

**(c) tolerate-completed-task-ref-on-followup-commit.** Modify
`check-active-task.sh` Gate 2 to:
1. If task is in `completed/`, read `date_finished` from frontmatter.
2. If within last 30 min: allow (emit stderr NOTE).
3. Otherwise: continue blocking.

Pros:
- Smallest code change — one branch in one hook.
- No coupling between decide path and git operations.
- Helps with edge cases beyond the inception incident: parallel sessions,
  batch-close paths, future decide-paths we haven't designed.

Cons:
- K=30 is arbitrary. Too short → still blocks legitimate cases.
  Too long → defeats P-002's "active task required" intent.
- Reactive, not preventive. Allows the bad pattern to happen.

## Composition analysis

(a) and (b) are mutually exclusive at the structural level — both try to
ensure no dirty state at decide-completion, just from opposite directions.
(c) is independent of both — it's about recovery, not prevention.

## Recommendation

**GO-COMPOSED — ship (a) as primary + (c) as defence-in-depth.**

Why (a) over (b):
- Operator UX is significantly better. The decide click "just works" — no
  friction trips.
- The whole *point* of using Watchtower is to compress decision-with-evidence
  into one step.
- The auto-add scope concern is bounded — paths derivable from task ID.
- (b) preserves the 2-commit cap deadlock: agent caps out → operator clicks
  decide → refused → "but I can't commit because I'm capped!".

Why (c) as defence-in-depth:
- Even with (a), edge cases will appear (race conditions, batch-decide paths,
  `fw inception decide` called from CLI without a clean tree, parallel-agent
  scenarios). (c) catches residue without trapping the agent.
- K=30 min is conservative; the natural window for "agent landing supporting
  evidence" is seconds-to-minutes.

### Follow-up build tasks (file post-GO)

1. **T-XXXX (primary, ~6 hr):** `do_inception_decide` auto-commits scope-related
   edits before the active→completed move. Scope =
   `.tasks/active/T-XXX-*.md` + `docs/reports/T-XXX-*` glob. Single commit
   `T-XXX: decision recorded — <DECISION> (<rationale-first-line>)`. Failure
   handling: if git operations fail, abort decide BEFORE the move (no
   half-applied state). Test against T-1904 incident replay.

2. **T-XXXX (defence-in-depth, ~2 hr):** check-active-task.sh Gate 2 allows
   commits referencing a completed task whose `date_finished` is within last
   30 min. Read `date_finished` from frontmatter. Emit stderr NOTE explaining
   the carveout. No flag needed — automatic + bounded.

3. **(optional, ~1 hr):** `do_inception_decide` emits an audit-trail entry
   to `.context/working/.inception-auto-commit.log` so future RCAs can trace
   which commits were synthesized by the decide path.

### Stale-assumption updates

- A1 ✓ confirmed in full.
- A2 ✓ confirmed in full.
- A3 ✓ confirmed — Gate 2 is the load-bearing blocker.
- A4 ✓ confirmed — zero grace-period concept exists.

### File a G-class concern?

Recommended: **YES — file G-XXX "Watchtower decide path leaves staged
working-tree evidence stranded" as medium severity, status=watching**, with
the live T-1904 incident as the source observation. Close the G-class entry
when the primary fix lands and stays green for ≥7 days.

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
