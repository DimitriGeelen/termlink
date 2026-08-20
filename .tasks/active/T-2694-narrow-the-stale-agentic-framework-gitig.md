---
id: T-2694
name: "Narrow the stale .agentic-framework gitignore rule so the vendored subset is trackable"
description: >
  .gitignore:21 blanket-ignores `.agentic-framework` under a label that predates vendoring.
  1565 files are already tracked so nothing looks broken, but every framework file added
  since is silently untrackable — `lib/bvp.sh` and all of `policy/` among them, which is why
  `fw bvp` fails in any clean clone. Replace the blanket rule with
  ignore-contents-plus-re-include-the-vendored-subset.
status: started-work
workflow_type: build
horizon: now
owner: claude-code
created: 2026-08-20
last_update: 2026-08-20
tags: [governance, gitignore, vendoring, clean-clone, bvp]
---

# T-2694: Narrow the stale `.agentic-framework` gitignore rule

## Context

`.gitignore:20-21`:

```
# Framework symlink (machine-specific)
.agentic-framework
```

The label is the bug. When it was written, `.agentic-framework` was a machine-specific
symlink and ignoring it was correct. The tree is now **vendored**, with 1565 files tracked
under it — including `bin/fw` itself.

Git ignore rules do not apply to already-tracked files, so everything committed before the
tree was vendored keeps working and nothing looks broken. But every framework file added
*after* the rule landed is silently untrackable: `git add -A` skips it and `git status`
never mentions it, because ignored files are not reported. The framework that RUNS and the
framework that is RECOVERABLE have been diverging invisibly.

### Confirmed casualties

`scripts/check-framework-tracking-drift.sh` axis B (T-2692) names them from the consumer
side — tracked code that sources a path which is not there:

```
DANGLING   $FRAMEWORK_ROOT/lib/bvp.sh
DANGLING   $FRAMEWORK_ROOT/lib/arc_membership.sh
```

Plus the whole of `policy/` (`value-drivers.yaml`, `bvp-scoring-rubric.md`) per T-2689.

Concretely: **`fw bvp` does not run in this worktree**, and would not run in a clean clone.
The §ACD sovereignty-gated BVP driver weights and the scoring rubric exist only on one
host's disk — one `rm -rf` from being unrecoverable. The BVP estimator has been requested
as routine tooling; it currently cannot be routine anywhere but one machine.

## Approach

Do **not** simply delete the rule. The vendored tree is a *subset* — the framework's own
`.context/`, `tests/`, `tools/` and `.git/` are deliberately not vendored here, and
un-ignoring everything would surface them as untracked noise and invite committing them.

Do **not** use `!` negations under the existing `.agentic-framework` rule either: git
cannot re-include a path whose parent directory is excluded, so those negations would
silently do nothing — the same class of quiet no-op this task exists to remove.

Correct form: exclude the directory's *contents* (`/*`, which leaves the directory itself
un-excluded so negations bind), then re-include exactly the vendored subset, then re-exclude
generated artefacts inside it.

The re-included set is derived from what is *already tracked* —

```
$ git ls-files .agentic-framework | sed 's|^\.agentic-framework/||' | cut -d/ -f1 | sort | uniq -c
   1197 docs      147 web       105 agents    103 lib       5 bin
      3 .tasks      1 VERSION     1 metrics.sh  1 .gitignore  1 .fw-not-a-project  1 FRAMEWORK.md
```

— plus `policy/`, which should have been tracked and is the live casualty.

## Scope boundary

This task changes the **rule**. It does not add the missing files: they live in the main
checkout, and T-559 project-boundary enforcement correctly stops a worktree session from
reaching them. Narrowing the rule is the structural half — it makes the files *visible to
`git status` and addable by a plain `git add`*, which is the actual defect. The one-time
catch-up `git add` is a main-checkout step, recorded as a Human AC below so the operator
reviews the list before committing rather than trusting a blind sweep from here.

## Acceptance Criteria

### Agent
- [x] The blanket `.agentic-framework` rule is replaced with a contents-plus-re-include form
- [x] Every currently-tracked top-level entry is re-included (no tracked path becomes ignored)
- [x] `policy/` is re-included (the live casualty)
- [x] The framework's own `.context/`, `tests/`, `tools/`, `.git/` remain ignored
- [x] `__pycache__/`, `*.pyc`, `*.pyo` remain ignored inside the re-included subtrees
- [x] Fixture proves a path under `lib/` is addable by a plain `git add` after the change
      and was NOT before it — the load-bearing property
- [x] Fixture proves the ignored-by-design paths are still ignored
- [x] Fixture is host-independent (scratch repo, no real framework)
- [x] No tracked file's status changes in this checkout (`git status` stays clean)

### Human
- [ ] Run the one-time catch-up in the main checkout and review before committing.
      **Steps:** 1. `cd /opt/termlink && git status --short .agentic-framework/ | head -50`
      2. Read that list — confirm nothing machine-local, host-specific or secret-bearing
      appears (host paths, tokens, per-machine config, `*.local.*`).
      3. `cd /opt/termlink && git add .agentic-framework/lib .agentic-framework/policy .agentic-framework/bin .agentic-framework/agents`
      4. `cd /opt/termlink && git commit -m "T-2694: track the vendored framework subset that the stale ignore rule hid"`
      5. `cd /opt/termlink && bash scripts/check-framework-tracking-drift.sh` → expect exit 0.
      **Expected:** step 5 reports no load-bearing drift and no dangling references, and
      `fw bvp --quadrant hv-lc` runs from a fresh worktree.
      **If not:** the check names what is still missing; re-run step 3 for that path.

## Verification

bash tests/gitignore-framework-scope-fixtures.sh
git status --porcelain .agentic-framework/ | wc -l | grep -qx 0

## Decisions

**Re-include the tracked subset, not everything.** Deleting the rule outright would surface
the framework's own `.context/`, `tests/` and `tools/` — not vendored here — as untracked
noise, and noise is what gets committed by accident. Enumerating the subset keeps the
signal.

**Enumerated from `git ls-files`, not from a guess.** The re-include list is exactly what is
already tracked plus `policy/`. That makes the change provably status-preserving for this
checkout: no tracked path can become ignored, so nothing can silently drop out of git.

**Rule here, files in main.** Splitting the fix is not a compromise, it is the correct
boundary: T-559 exists so a worktree session cannot reach into the parent checkout, and the
`git add` deserves a human reading the list first — which is precisely the review I could
not perform from here.
