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
last_update: 2026-08-20T00:31:33Z
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

Axis B scans framework sources under `bin/`, `lib/` and `agents/` and tests each resolved
path with `-e`. References still carrying a shell interpolation in the tail cannot be
resolved statically, so they are skipped and counted rather than guessed.

**The anchor is narrow by necessity.** The first implementation matched *every*
`"$FRAMEWORK_ROOT/..."` occurrence and reported **47** dangling references, of which ~44
were noise: bare `$VAR` interpolations, `path/to/script.sh` usage examples in help text,
and the framework's own `tests/`, `tools/`, `.git/` and `.context/` — all of which a
vendored copy legitimately omits. A check that is wrong 44 times out of 47 is a check
nobody reads.

Narrowing to references in **source-or-execute position** (`.` / `source` / `bash` / `sh` /
`python3` / `python` immediately preceding the quoted path), and dropping comment lines,
took it to **2** — `lib/bvp.sh` and `lib/arc_membership.sh`, both genuinely missing, zero
false positives. A sourced or executed path is load-bearing by construction: if it is
absent the command fails at runtime. This mirrors the sibling static checks, which also buy
precision with a narrow anchor (T-2666 keys on an exact preceding-line shape, T-2672 on
specific RPC method strings).

## Acceptance Criteria

### Agent
- [x] Axis B detects a `$FRAMEWORK_ROOT/...` reference whose target is missing
- [x] Axis B fires (exit 1) in this worktree, where `lib/bvp.sh` is genuinely absent
- [x] Axis A behaviour is unchanged — T-2689's fixtures still pass unmodified
- [x] Dynamic references (a `$` remaining in the tail) are skipped and counted, not guessed
- [x] References in a comment line are ignored
- [x] References NOT in source-or-execute position (assignment, `echo`, `ls`) are ignored —
      the anchor-narrowness property that took the false-positive count 47 → 0
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

**Static-only resolution.** References carrying an interpolation in the tail are skipped
rather than guessed. A false positive in a check that gates nothing would train people to
ignore it; skipping is honest and the skipped count is reported.

**Precision over recall.** The narrow source-or-execute anchor will miss a genuinely
missing file that is only referenced indirectly (built into a variable, then invoked). That
is an accepted trade: this check gates nothing and is read by humans, so its value is
entirely in being trusted. The broad version was *more complete and less useful* — 47 hits
that an operator would learn to scroll past. Two hits they will act on beats forty-seven
they will not.
