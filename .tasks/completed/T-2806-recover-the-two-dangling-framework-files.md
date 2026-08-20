---
id: T-2806
name: "Recover the two dangling framework files: task finalization dies on every build task in a worktree"
description: >
  arc_membership.sh and bvp.sh are untracked (T-2814 blanket gitignore) so they are absent from every worktree and clean clone. evolution_log.sh:52 sources arc_membership.sh with no existence guard, so `fw task update --status work-completed` exits 1 AFTER passing every gate. 24 active build tasks are fully ticked and stuck, including every task this session completed. Also unblocks `fw bvp`.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, framework-recoverability, g-019, g-062, directive-2]
components: []
related_tasks: [T-2814, T-2817, T-2822, T-2803, T-2804, T-2805]
created: 2026-08-20
last_update: 2026-08-20T15:05:06Z
date_finished: 2026-08-20T15:05:06Z
---

# T-2806: Recover the two dangling framework files

## Context

T-2817 built the DANGLING axis to detect tracked framework code that sources a path which is
not present, and reported two: `lib/arc_membership.sh` and `lib/bvp.sh`. At the time the known
cost was that `fw bvp` fails in a clean clone.

The cost is larger than that, and it was found by tripping over it. Completing T-2805 printed:

```
lib/evolution_log.sh: line 52: .../lib/arc_membership.sh: No such file or directory
```

**11/11 acceptance criteria, 4/4 verification commands passed — and the task was not
finalized.** It stayed `started-work` in `active/`. Reproduced deliberately: exit 1, file not
moved, every time.

### The mechanism is a guard that covers the wrong file

`update-task.sh::check_evolution_log` is careful:

```bash
local lib_path="$FRAMEWORK_ROOT/lib/evolution_log.sh"
[ -f "$lib_path" ] || return 0      # guards THIS file
source "$lib_path"
```

But `evolution_log.sh:52` then does, unguarded:

```bash
. "$__el_lib_dir/arc_membership.sh"
```

The comment two lines above says *"guard the path resolution so we don't fail if FRAMEWORK_ROOT
isn't yet set"* — and the guard it describes protects against an **unset variable**, not a
**missing file**. So the caller's existence check is real, and it protects exactly one link of a
two-link chain. The sourced file's own dependency was never covered by anything.

The gate is scoped to `workflow_type: build`, which is why this is not universally visible: an
inception or refactor task finalizes fine, and a build task dies.

### Blast radius, measured

**24 active build tasks have every acceptance criterion ticked and are still `started-work`** —
including **every task completed in this session and the previous one** (T-2813, T-2814, T-2816,
T-2817, T-2818, T-2820, T-2821, T-2800 through T-2805). The work is committed; the task records
say it never finished.

This is a third, distinct mechanism for the half-finalized class T-2804 repaired 13 instances
of. It is not the same signature — T-2804's were in `completed/` with `status: work-completed`
and a missing `date_finished`, whereas these never leave `active/` at all — but it produces the
same end state: work that is done and a register that disagrees.

### Why it is invisible in the checkout where it does not happen

The main checkout HAS both files on disk. They are merely untracked, so everything works there
and nothing looks wrong. A worktree materialises only *tracked* files, so the failure appears
exclusively in worktrees and clean clones — and the framework creates those worktrees itself.

### The deadlock this sits in

- **Main** has the files and cannot `git add` them: its `.gitignore` still carries the blanket
  rule, and `git add` on an ignored path fails (T-2803 measured this and now BLOCKs on it).
- **This branch** carries the narrowed rule (T-2822) and can add them — but does not have them,
  because they were never committed and there is nothing to pull.

Neither checkout can fix it alone. The break is to move the bytes from the checkout that has
them into the one that is allowed to track them.

## Approach

Copy both files from the main checkout into this worktree and commit them here, where the
narrowed `.gitignore` permits tracking. Cross-checkout access goes through **TermLink dispatch**
(T-559 project-boundary policy) rather than a direct read, so the main checkout's own governance
runs in its own process.

**Scan before committing.** These are framework source files from another checkout; a
machine-local path, an IP or a credential embedded in one would become permanent history. Reuse
the mechanised scan from `scripts/remediate-main-checkout.sh` (T-2803) rather than eyeballing.

**Verify by the behaviour, not by the file appearing.** The property that matters is that a
build task finalizes, so the proof is completing one and watching it move — not `test -f`.

## Scope boundary

Recovers these two files and files the unguarded-source defect upstream. Does **not** patch
`evolution_log.sh` — it is vendored (G-062) and a local edit is erased on the next re-vendor;
and with the file present the code path is correct anyway. Does **not** finalize the 24 stuck
tasks: most are not mine, several are `owner: human`, and completing a human-owned task is
outside autonomous authority. Does **not** widen the `.gitignore` further than T-2822 already did.

## Acceptance Criteria

### Agent
- [x] `lib/arc_membership.sh` and `lib/bvp.sh` are present in this worktree and **tracked**
- [x] Both were scanned for secrets and machine-local paths before being committed
- [x] `check-framework-tracking-drift.sh` reports **0 dangling references** (was 2)
- [x] `fw task update --status work-completed` finalizes a build task: it moves to
      `completed/` and its status changes — proven by behaviour, not by the file existing
- [x] `fw bvp` runs instead of failing on a missing library
- [x] The unguarded `source` at `evolution_log.sh:52` is filed upstream on `framework:pickup`
      with the reproduction and the blast-radius measurement
- [x] CLAUDE.md's T-2817 section records that DANGLING also breaks task finalization, not only
      `fw bvp`

## Verification

# Both files are tracked, not merely present.
test -n "$(git ls-files .agentic-framework/lib/arc_membership.sh)"
test -n "$(git ls-files .agentic-framework/lib/bvp.sh)"
# The DANGLING axis is clean — this is the check that reported the defect.
bash scripts/check-framework-tracking-drift.sh
# The finalize chain loads end to end. Sourcing the library that failed must now
# succeed AND actually define the function whose absence killed the run.
bash -c 'FRAMEWORK_ROOT=.agentic-framework; . .agentic-framework/lib/evolution_log.sh; declare -F task_has_arc_membership >/dev/null'

## Decisions

### 2026-08-20 — Move the bytes rather than widen the ignore rule again

- **Chose:** Copy from the checkout that has the files; commit here where tracking is allowed.
- **Why:** The two checkouts are each missing a different half of what a fix needs, and neither
  can complete it alone. Widening `.gitignore` further would not help — the rule is already
  narrowed on this branch; the missing ingredient is the file content, which exists on exactly
  one disk.

### 2026-08-20 — Do not patch the vendored guard, even though it is a one-line fix

- **Chose:** File it upstream; recover the file locally.
- **Why:** G-062 — a local edit to `evolution_log.sh` is erased on the next re-vendor, so it
  would fix this checkout and silently regress. And the guard is only load-bearing while the
  file is missing; recovering the file removes the failure for every consumer, whereas the guard
  fix removes only the crash and leaves `task_has_arc_membership` undefined.

### 2026-08-20 — Prove it by finalizing a task, not by `test -f`

- **Chose:** The acceptance criterion is that a build task actually moves to `completed/`.
- **Why:** The file being present is the input, not the outcome. This session has repeatedly
  found checks that assert something adjacent to the property they claim — the audit checking
  that an episodic file exists rather than parses (T-2805) is the same error, and it hid 29
  corrupt files for months.

### 2026-08-20 — Leave the 24 stuck tasks alone

- **Chose:** Fix the mechanism; do not bulk-finalize the backlog.
- **Why:** Several are `owner: human`, and completing a human-owned task is explicitly outside
  autonomous authority. A bulk sweep over tasks whose acceptance criteria I did not verify is
  also precisely how the G-066 finalization-bypass class was created in the first place.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-52b1b2cf
- **Timestamp:** 2026-08-20T15:05:08Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-20T15:05:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
