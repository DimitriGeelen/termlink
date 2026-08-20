---
id: T-2800
name: "Detect cross-branch task-ID collisions and duplicate work before it happens"
description: >
  T-229 renumbered colliding task IDs in March 2026 and noted the counter is not safe for
  concurrent work. Nothing was built. It recurred in August at 12x — and the expensive half
  was not the IDs but three agents independently solving the same two defects. Add a
  deploy-time check with two axes: IDs claimed by more than one branch, and near-duplicate
  task titles across branches.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, concurrency, task-system, duplicate-work]
components: []
related_tasks: [T-229, T-2696, T-2697, T-2698, T-2561]
created: 2026-08-20
last_update: 2026-08-20
date_finished: null
---

# T-2800: Detect cross-branch task-ID collisions and duplicate work before it happens

## Context

Every worktree allocates task IDs by scanning its **own** `.tasks/` directory for the highest
existing ID and incrementing. Worktrees materialise only their own branch's files, so no
branch can see any other branch's allocations. The framework creates those worktrees itself
(bgIsolation, and CLAUDE.md instructs background sessions to isolate before editing), so this
is a framework-made race, not an operator oversight.

### It already happened once, and the gap was recorded

T-229 (March 2026, `work-completed`) is titled *"Renumber colliding task IDs T-205–T-210 to
T-222–T-227"*, and its Context ends:

> Also register **G-007** for framework upstream (**task counter not safe for concurrent
> work**).

So this was understood five months ago. The symptom was fixed by hand — six task files, six
episodic files, fourteen fabric cards, one handover, all renumbered — and the underlying gap
was routed upstream. Nothing landed. Note also what T-229 left behind as its verification:

```
test "$(cat .tasks/active/*.md .tasks/completed/*.md | grep '^id:' | sort | uniq -d | wc -l)" = "0"
```

That command reads **one working tree**. It is structurally incapable of seeing the failure
it was written to close, and it has passed cleanly every day since while the collision count
went to twelve.

### The current state

Measured across all local branches against their common base (`main`, max T-2677):

```
worktree-charter-review-2026-0814      117 new  T-2678..T-2794
worktree-governance-canary-signal        3 new  T-2690..T-2692
worktree-t2687-pickup-failopen          12 new  T-2687..T-2698
                                colliding IDs:  12
```

Every collision holds a **different** task. T-2690 is simultaneously "termlink purpose review
4", "canary stderr sink severs detection", and "audit cron-drift slug uses worktree basename".

### The expensive half is not the IDs

Renumbering twelve tasks is mechanical. What is not recoverable is this:

| Defect | governance-canary-signal | charter-review | t2687-pickup-failopen |
|---|---|---|---|
| Canary stderr merged into findings log | T-2690 `work-completed` 08-18 | T-2685 started | T-2696 (this session) |
| Static-check allowlists untracked | T-2692 `work-completed` 08-18 | T-2681 started | T-2698 (this session) |

Three agents, two defects, six tasks. One pair finished two days before the third was started.
Nothing warned anyone — not `fw work-on`, not the handover, not the audit.

And the duplication is not random. All three agents were pointed at "framework governance",
and all three found the same two things because those were the most visible problems in the
tree. A shared work-surface is not just collision avoidance; it is the difference between
three agents covering three problems and three agents covering one.

This task builds the **detector**. The allocator fix belongs to the framework and is filed
upstream (`framework:pickup` offsets 15–16, G-062).

## Approach

One script, two axes — the shape `check-framework-tracking-drift.sh` already uses, because
each axis is blind exactly where the other fires.

**Axis A — colliding IDs.** For each local branch, the set of task IDs not present in the
merge base. Any ID claimed by two or more branches fires, and the differing filenames are
printed so "same task on two branches" (harmless — a cherry-pick) is distinguishable at a
glance from "two different tasks, one ID" (the real defect).

**Axis B — near-duplicate titles.** The axis that would actually have saved this session.
Task titles are compared across branches by normalised word-set overlap; a pair above the
similarity threshold on *different* branches is reported. Deliberately a WARNING, not a
firing condition: title similarity is a heuristic and a false positive must never block, only
prompt a look. Axis A fires; axis B advises.

Same-branch pairs are excluded (a branch is allowed to have related tasks), as are pairs
already sharing an ID (axis A owns those).

Deploy-time / ad-hoc, **not** a cron canary — the same tier as `check-cron-install-drift.sh`.
Run it before starting work, and before a merge.

## Acceptance Criteria

### Agent
- [x] Axis A reports every task ID claimed by more than one branch, with the differing
      filenames alongside
- [x] Axis A does NOT fire when two branches carry the same ID for the *same* task
      (cherry-pick / shared history), only when the filenames differ
- [x] Axis A fires (exit 1) on a genuine collision
- [x] Axis B reports near-duplicate task titles across different branches as a non-firing
      warning
- [x] Axis B excludes same-branch pairs and pairs already reported by axis A
- [x] Axis B's similarity threshold is tunable
- [x] `--json` carries both axes separately with their own counts
- [x] Exit codes: 0 clean, 1 collision, 2 tooling (not a git repo)
- [x] Fixtures build a scratch repo with real branches and prove each axis independently,
      host-independent (PL-213)
- [x] Fixture proves the same-ID-same-task case does NOT fire — the false-positive guard
- [x] Run against this repo, it reports the 12 known collisions and the two known duplicate
      pairs

## Verification

bash tests/task-id-collision-fixtures.sh

## RCA

**Symptom:** twelve task IDs are each claimed by two or three branches for different tasks,
and three agents independently implemented fixes for the same two defects.

**Root cause:** task-ID allocation scans the current working tree only. Worktrees are
isolated by construction, so concurrent branches allocate from the same starting point and
produce overlapping ranges. There is no cross-branch view of either IDs or work in progress.

**Why structurally allowed:** the gap was identified in March 2026 (T-229) and routed
upstream as G-007 rather than mitigated locally, and nothing tracked the handoff. The local
duplicate-ID check that T-229 left behind reads a single working tree, so it cannot observe
the cross-branch case at all — it has been passing throughout. The framework encourages
parallel isolated agents and provides no shared work-surface, so duplicated effort produces
no signal until someone happens to read another branch.

**Prevention:** axis A makes the collision visible before a merge; axis B makes the
duplicated *work* visible before it is done, which is the expensive half. Neither fixes
allocation — that is upstream's to take (filed) — but both convert a silent cost into a
loud one.

**Residual risk:** this is an ad-hoc check, so it only helps someone who runs it. It is
therefore worth wiring into the session-start path rather than leaving it as another
dormant script (PL-168). Left as a follow-up rather than assumed here.

## Decisions

### 2026-08-20 — Two axes, only one of them fires

- **Chose:** Axis A (IDs) fires; axis B (titles) warns.
- **Why:** An ID collision is a fact — two files, one identifier, provable. Title similarity
  is a judgement, and a heuristic that blocks on a judgement gets disabled the first time it
  is wrong. Axis B's value is entirely in being read, not in being obeyed.

### 2026-08-20 — Detector here, allocator upstream

- **Chose:** Build only the detection locally; file the allocation fix.
- **Why:** G-062 — `fw task create` is vendored framework code, so a local edit is erased on
  the next re-vendor. That is exactly what happened to G-007: the fix was correctly routed
  upstream and then nothing existed locally to notice it had not arrived. A local detector is
  the part that survives.

### 2026-08-20 — This task's own ID was picked by hand

- **Context:** `fw work-on` allocated T-2699 for this task. T-2699 is already claimed by
  charter-review, so the allocator was about to create a thirteenth collision while creating
  the task whose purpose is to detect them. `fw task create` has no `--id` flag, so the file
  was renamed to T-2800 afterwards.
- **Why it matters:** the framework offers no way to avoid a collision even once you have
  detected it. Recorded here because it is the sharpest available evidence for the upstream
  filing, and because the next person to hit this needs to know the workaround is a manual
  rename.
