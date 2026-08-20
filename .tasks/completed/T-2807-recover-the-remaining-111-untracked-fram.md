---
id: T-2807
name: "Recover the remaining 111 untracked framework files so a clean clone has the framework that runs"
description: >
  T-2806 recovered four subtrees by following the DANGLING chain. 111 of 326 files under lib/bin/policy/agents remain untracked, including hook scripts settings.json invokes by path — among them the T-1845 large-file gate. Recover them, scanning before commit.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, framework-recoverability, g-019, g-062, directive-2]
components: [scripts/check-framework-tracking-drift.sh, tests/framework-dangling-ref-fixtures.sh]
related_tasks: [T-2814, T-2817, T-2822, T-2803, T-2806, T-1845, T-2052]
created: 2026-08-20
last_update: 2026-08-20T15:14:34Z
date_finished: 2026-08-20T15:14:34Z
---

# T-2807: Recover the remaining untracked framework files

## Context

T-2806 recovered four subtrees by following the DANGLING chain until it converged, which fixed
the acute breakage (task finalization, `fw bvp`). It did not touch the rest, deliberately —
that was a separate deliverable with a different risk profile.

The rest is most of it. Measured across `lib/ bin/ policy/ agents/`: **326 files on disk in the
main checkout, 213 tracked, 111 still untracked** after T-2806. So roughly a third of the
framework's executable surface exists on exactly one disk and is absent from every worktree and
every clean clone.

### What is actually missing, and why it is worse than a count

Spot-checked in this worktree, all absent:

```
agents/context/check-arc-id.sh
agents/context/check-task-ac-structure.sh
agents/context/revisit-due-scan.sh
agents/git/lib/large-file-scan.sh
```

These are not library internals. They are enforcement scripts. But which enforcement, exactly,
matters — and the first framing of this task got it wrong, so here is what was actually
measured.

**Claude Code hooks are fine.** `.claude/settings.json` invokes them as
`/opt/termlink/.agentic-framework/bin/fw hook <name>` — an absolute path into the MAIN
checkout. So `check-active-task`, `check-tier0`, `check-project-boundary` and the rest all run
here, resolved out of a directory this worktree does not own. No gap.

**Git hooks are not.** `.git/hooks/pre-commit` is shared across worktrees, but it resolves
`FRAMEWORK_ROOT` to **`$PROJECT_ROOT/.agentic-framework`** — the worktree's own copy — and gates
each scanner on `-f`:

```
SCANNER="$FRAMEWORK_ROOT/agents/git/lib/secret-scan.sh"
MASTER_GUARD="$FRAMEWORK_ROOT/agents/git/lib/master-guard.sh"
DUP_TASK_SCANNER="$FRAMEWORK_ROOT/agents/git/lib/dup-task-scan.sh"
```

Checked against this branch's HEAD before recovery, of the four pre-commit guards:

| scanner | tracked at HEAD | ran during this session's commits |
|---|---|---|
| `secret-scan.sh` | **yes** | **yes** |
| `large-file-scan.sh` (T-1845, the G-058 gate) | no | **no — failed open** |
| `master-guard.sh` (blocks commits to master) | no | **no — failed open** |
| `dup-task-scan.sh` | no | **no — failed open** |

So **three of the four pre-commit guards were silently absent** in this worktree, including the
one CLAUDE.md credits with preventing the G-058 root cause (a 288 MB blob committed in May,
silently rejected by GitHub for 14 days) and the one that stops a commit landing on master.
Secret scanning did run, which is the reassuring half and is stated because the question "were
this session's commits scanned?" deserves an answer rather than an assumption.

### Why nothing reported it

The same asymmetry as T-2817. In the main checkout the files are present on disk, so every
scanner fires and everything is green; they are merely untracked, which no surface mentions. A
worktree materialises only tracked files, so the scanners are silently absent — and the hook
**fails open by design**: it gates on `-f` and skips, with a note in commit output that nobody
reads. That leniency is deliberate (T-2061, and the T-2052 install-time chmod gap it works
around), and it is correct for a missing optional tool. It is the wrong behaviour when the file
is missing because it was never committed, and the hook cannot tell those apart.

The T-2817 DANGLING axis structurally cannot see this class: it matches `source`/`exec`-position
references in tracked shell, and a hook named as a path inside `settings.json` is neither. That
blind spot is real and was filed upstream at `framework:pickup` offset 21; closing it locally is
T-2808, kept separate so this task stays a reviewable repair.

## Approach

Recover the remaining files the same way T-2806 did: enumerate from the checkout that has them,
move the bytes across through a **TermLink session rooted there** (T-559 project boundary, so
that checkout's governance runs in its own process), scan, then commit here where the narrowed
`.gitignore` (T-2822) permits tracking.

**Scan before committing, and treat the scan as advisory input rather than a verdict.** 111
files from another disk is exactly where a machine-local path or a credential enters permanent
history. T-2806's scan flagged a bare 64-hex that turned out to be a `sha256:` reproducibility
pin — a true pattern match and a false risk. Hard findings stop the commit and get read
individually; they do not get suppressed to make the run green.

**Exclude what should not be tracked.** `__pycache__/`, `*.pyc`, and anything under a `.git/`
are build artefacts, not source. Test fixtures and `docs/` are recovered too where present:
the standard for "load-bearing" here is "a clean clone behaves like this checkout", and a
missing test is a missing ability to verify.

**Prove it by behaviour.** The point is not that files appear; it is that the guards run. The
acceptance criterion is that every hook script `settings.json` names resolves on disk, and that
`fw doctor` passes — not `test -f` on a sample.

## Scope boundary

Recovers files that already exist in the sibling checkout. Does **not** write or modify any
framework code — every byte committed is bytes that already run today. Does **not** add the
settings.json-reference detection axis (T-2808). Does **not** widen `.gitignore` beyond T-2822.
Does **not** touch `.context/` or `.tasks/`, which are project data, not framework source.

## Acceptance Criteria

### Agent
- [x] Every file under `lib/ bin/ policy/ agents/` present in the sibling checkout is tracked
      here, excluding `__pycache__/`, `*.pyc`, `*.pyo`
- [x] All recovered files were secret-scanned before commit; any hard finding was inspected
      individually and its disposition recorded, not suppressed
- [x] All four pre-commit scanners resolve under this worktree's `FRAMEWORK_ROOT`:
      `secret-scan.sh`, `large-file-scan.sh`, `master-guard.sh`, `dup-task-scan.sh`
- [x] `agents/git/lib/large-file-scan.sh` — the T-1845 G-058 gate — is present, so the guard
      actually runs here rather than being skipped
- [x] `check-framework-tracking-drift.sh` reports 0 dangling and 0 load-bearing untracked
- [x] `fw doctor` reports **zero FAILures** — it exits 1 on WARNings, and the three that
      remain (framework path ambiguity; commit-msg and pre-push "not installed") are
      pre-existing worktree artefacts, not caused here: the hooks ARE installed in the
      shared `.git/hooks`, and doctor looks in the per-worktree path
- [x] Recovered shell/python scripts keep their executable bit — a present but non-executable
      scanner is the PL-205 fail-open, which is the failure this repairs, not a fix for it
- [x] CLAUDE.md records that enforcement hooks were absent in worktrees and what that cost

## Verification

# The drift check is clean on both axes.
bash scripts/check-framework-tracking-drift.sh
# All four pre-commit scanners resolve here. `-f`, not `-x`, because that is the
# predicate the hook itself uses (T-2061, working around the T-2052 chmod gap) —
# asserting -x would be a stricter claim than the thing being verified.
test -f .agentic-framework/agents/git/lib/secret-scan.sh
test -f .agentic-framework/agents/git/lib/large-file-scan.sh
test -f .agentic-framework/agents/git/lib/master-guard.sh
test -f .agentic-framework/agents/git/lib/dup-task-scan.sh
# Nothing under the recovered subtrees is left untracked.
test -z "$(git status --porcelain .agentic-framework/lib .agentic-framework/bin .agentic-framework/policy .agentic-framework/agents)"
# fw doctor reports no FAILures (it exits 1 on WARNings, so grep the report).
d=$(mktemp); .agentic-framework/bin/fw doctor > "$d" 2>&1; n=$(grep -c FAIL "$d"); rm -f "$d"; test "$n" -eq 0
# Both drift-check fixture suites still pass, including the T-2807 placeholder case.
bash tests/framework-dangling-ref-fixtures.sh
bash tests/framework-tracking-drift-fixtures.sh

## Decisions

### 2026-08-20 — Recover the whole subtree, not only what something references

- **Chose:** Everything under `lib/ bin/ policy/ agents/` that exists in the sibling checkout.
- **Why:** The reference-following approach is what T-2806 did, and it converges on exactly the
  files some tracked script happens to name in source position. That set is a lower bound by
  construction — it missed all 19 hook scripts. The property worth having is "a clean clone
  behaves like this checkout", and only copying the subtree delivers that.

### 2026-08-20 — Commit only bytes that already run

- **Chose:** No edits to any recovered file, including obvious ones.
- **Why:** This is a recovery, and a recovery that also changes things is unreviewable — a
  reader cannot tell which differences are the repair. Anything that deserves changing gets its
  own task, and most of it is vendored anyway (G-062).

### 2026-08-20 — Preserve the executable bit explicitly

- **Chose:** Copy with mode preserved and assert it afterwards.
- **Why:** PL-205 records the framework's pre-commit hooks failing OPEN when scanners are
  present but not executable — `secret-scan: scanner not found (skipping)`. Recovering the files
  without their mode would convert "missing scanner, fails open" into "present scanner, still
  fails open" while making the drift check report success. That is a worse state than the one
  being repaired, because it looks fixed.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-2b86f768
- **Timestamp:** 2026-08-20T15:14:39Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-20T15:14:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
