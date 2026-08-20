---
id: T-2692
name: "Tracking-drift check is blind in a clean clone — add dangling-reference detection"
description: >
  check-framework-tracking-drift.sh compares on-disk files against git, so it only sees
  drift in the checkout where the untracked files physically exist. In a worktree or a
  clean clone the files are simply absent and it reports clean — the exact environment
  where the breakage bites. `fw bvp` is broken in this worktree right now while the check
  says everything is fine.
status: started-work
workflow_type: build
horizon: now
owner: claude-code
created: 2026-08-20
last_update: 2026-08-20
tags: [governance, tooling, clean-clone, detection-gap]
---

# T-2692: Tracking-drift check is blind in a clean clone

## Context

T-2689 shipped `scripts/check-framework-tracking-drift.sh` to detect vendored-framework
files unreachable from git. It works by listing files on disk and testing each against
`git ls-files`.

That construction carries an inherent blind spot: **it can only report a file it can see.**
In the checkout where someone created an untracked file, the file is on disk and the check
fires. In a *clean clone* — or a git worktree, which materialises only tracked files — the
file is simply absent, so there is nothing to iterate over and the check reports clean.

The blind spot is not theoretical. Running the BVP estimator from this worktree:

```
$ fw bvp --quadrant hv-lc
.agentic-framework/bin/fw: line 3223: .../.agentic-framework/lib/bvp.sh: No such file or directory

$ bash scripts/check-framework-tracking-drift.sh
check-framework-tracking-drift: no load-bearing drift (1565 file(s) scanned, 0 informational)
```

The tool is broken and the detector says clean, in the same directory, seconds apart. This
is exactly the "clean clone has a tracked `bin/fw` routing to a library that is not there"
scenario T-2689's own header predicted — and T-2689's check cannot see it.

### Why this matters more than the original axis

The person who *created* the drift is the least likely to be hurt by it: their machine has
the files. The people hurt are everyone who clones, deploys, or spins up a worktree — and
for all of them, T-2689's axis is silent. The axis that matters for the consumer was the
one missing.

## Approach

Add a second, complementary detection axis to the same script:

- **Axis A (existing, T-2689)** — file on disk, absent from git. Fires where the drift was
  created.
- **Axis B (new)** — a `"$FRAMEWORK_ROOT/<path>"` reference in tracked framework code whose
  target does not resolve on disk. Fires where the file is missing.

Neither subsumes the other; each is blind exactly where the other fires. Both firing sets
feed the same exit code, so a single invocation is correct in both environments.

Axis B scans tracked shell sources under the framework root for the literal
`"$FRAMEWORK_ROOT/..."` interpolation shape and tests each resolved path with `-e`. Paths
ending in `/` are directory references and are tested as directories. References containing
a further shell interpolation (`${...}` beyond the leading `$FRAMEWORK_ROOT`) are skipped —
they cannot be resolved statically, and guessing would produce false positives.

## Acceptance Criteria

### Agent
- [x] Axis B detects a `$FRAMEWORK_ROOT/...` reference whose target is missing
- [x] Axis B fires (exit 1) in this worktree, where `lib/bvp.sh` is genuinely absent
- [x] Axis A behaviour is unchanged — T-2689's fixtures still pass unmodified
- [x] Dynamic references (containing a nested `${...}`) are skipped, not false-positived
- [x] Directory references (trailing `/`) are tested as directories
- [x] `--json` carries the axis-B findings separately from axis A
- [x] Regression fixtures cover: missing target fires; present target clears; dynamic
      reference skipped; both axes can fire together
- [x] Fixtures are host-independent (build their own scratch tree, no real framework)

## Verification

bash tests/framework-tracking-drift-fixtures.sh
bash tests/framework-dangling-ref-fixtures.sh

## Decisions

**Second axis in the same script, not a new script.** The question both axes answer is one
question — "is the framework on this machine actually complete and recoverable?" — and an
operator should not have to know which of two commands matches their environment. Splitting
them would reproduce the original failure: a check that is correct only where you are not.

**Static-only resolution.** References carrying a nested interpolation are skipped rather
than guessed. A false positive in a check that gates nothing would train people to ignore
it; skipping is honest and the skipped count is reported.
