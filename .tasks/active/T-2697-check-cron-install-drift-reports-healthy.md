---
id: T-2697
name: "check-cron-install-drift reports healthy while crontabs diverge from git"
description: >
  DRIFT is a non-firing WARNING by default, so the check exits 0 and prints "healthy" while
  installed crontabs differ from their git source. It reported 3 matching / 21 drifting —
  and said "healthy" — while that drift concealed a real uncommitted fix (T-2696) for an
  unknown period. Make DRIFT fire by default, with an allowlist for deliberate host-local
  variation, matching the four existing static-check allowlist precedents.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, cron, drift, reliability]
components: []
related_tasks: [T-2561, T-2696, T-2527, T-2531, T-2666, T-2672]
created: 2026-08-20
last_update: 2026-08-20
date_finished: null
---

# T-2697: `check-cron-install-drift` reports healthy while crontabs diverge from git

## Context

`scripts/check-cron-install-drift.sh` (T-2561) exists because a canary crontab committed to
git is not thereby installed to `/etc/cron.d` — it can ship dark and never fire (the G-069
class, recorded as a learning: *"shipped != live recurs at the CANARY layer"*). It classifies
each git-tracked crontab as MISSING (fires), DRIFT (**non-firing warning** unless `--strict`),
or OK.

That default is what this task changes, on evidence.

While investigating T-2696, the check reported:

```
check-cron-install-drift: healthy (3 installed + matching, 21 drift-warning, 0 skipped)
  → exit 0
```

**Twenty-one of twenty-four installed crontabs differed from their source of truth, and the
verdict word was "healthy".** The divergence was not cosmetic: every one of those files
carried a real, correct fix — routing canary stderr to a `.log.stderr` companion instead of
merging it into the findings log — that had been applied to the deployed copies and never
committed. So the live behaviour and the reviewable source had been telling different stories
for an unknown length of time, and the check whose entire purpose is to notice that said
"healthy" every time it ran.

The failure mode is worth naming precisely, because it is not "the check was wrong". The
check saw the drift and printed it. It simply declined to insist — and a warning nobody must
act on is indistinguishable from no warning at all once it has scrolled past. Twenty-one
files is what that looks like after a while.

### Why the original default was reasonable, and why it no longer is

T-2561 made DRIFT non-firing deliberately: MISSING is unambiguous (the canary is dark), while
DRIFT could be a benign local edit — a host-specific path or schedule — and firing on it
would create permanent red on legitimately-varied hosts.

That reasoning is sound but the repo has since settled the general question in the opposite
direction, four times over. The alloc-sink (T-2527), drain-sink (T-2531), silent-exit
(T-2666) and busy-spin (T-2672) checks all **fire by default** and carry an
`.context/working/.*-allowlist` where a confirmed-safe instance is acknowledged with a cited
reason. That shape gets both properties: nothing is silently tolerated, and a deliberate
exception is cheap to record once. PL-219 states the underlying bias directly — *"a false
fire is cheap while a false silence is the whole point"*.

Applying the established pattern here is a smaller change than inventing one.

## Approach

Adopt the sibling checks' shape exactly:

1. **DRIFT fires by default** (exit 1), joining MISSING.
2. **`.context/working/.cron-drift-allowlist`** acknowledges deliberate host-local variation,
   one crontab basename per line with a `#` reason, same file format and spirit as the four
   existing allowlists. An allowlisted file is reported as ACKNOWLEDGED, not hidden — the
   count stays visible so an allowlist that has quietly grown is still readable.
3. **`--lenient`** restores the old non-firing behaviour for anyone who depended on it.
4. **The verdict word stops lying.** "healthy" is emitted only when nothing is missing and
   nothing is drifting unacknowledged.

`--strict` is retained as an accepted no-op alias so existing invocations keep working.

The check has **no automated caller** — `grep` across the repo finds only documentation
references, and `/canaries` explicitly delegates to it rather than running it. So changing
the default exit code cannot cascade into a cron or a gate.

## Acceptance Criteria

### Agent
- [x] DRIFT fires (exit 1) by default, without `--strict`
- [x] `--lenient` restores the pre-change non-firing DRIFT behaviour
- [x] `--strict` is still accepted and behaves the same as the new default (no breakage)
- [x] An allowlisted crontab is reported as ACKNOWLEDGED and does not fire
- [x] Allowlisted entries remain visible in the output and counts — acknowledged, not hidden
- [x] The verdict prints "healthy" only when nothing is missing and nothing is unacknowledged
- [x] `--json` carries the allowlist state and an `acknowledged` count
- [x] MISSING still fires regardless of allowlist or `--lenient` — a dark canary is never
      acknowledgeable
- [x] Fixtures prove each class from a scratch tree, host-independent (PL-213)
- [x] Fixture proves the load-bearing property: drift with an empty allowlist fires, and the
      same drift with the file acknowledged does not
- [x] The check passes on the current tree (T-2696 already reconciled all 21)

## Verification

bash tests/cron-drift-firing-fixtures.sh
bash scripts/check-cron-install-drift.sh

## RCA

**Symptom:** `check-cron-install-drift.sh` printed `healthy (3 installed + matching, 21
drift-warning, 0 skipped)` and exited 0 while 21 of 24 installed crontabs differed from their
git source.

**Root cause:** DRIFT was classified as a non-firing WARNING by default, and the summary line
used the word "healthy" whenever nothing was MISSING — regardless of the drift count. Exit 0
plus "healthy" is indistinguishable from a clean run for any caller and for a human skimming.

**Why structurally allowed:** the check was built to answer "is this canary installed at
all?", where a binary present/absent answer is unambiguous. "Is the installed copy the one we
reviewed?" is the harder question, and it was given a softer verdict to avoid false alarms on
host-local edits — before the repo had established the allowlist pattern that resolves that
tension. Nothing then re-examined the default as four sibling checks converged on the
opposite convention.

**Prevention:** drift now fires, so divergence has to be either fixed or explicitly
acknowledged with a reason. Fixtures pin that an empty allowlist fires on the same input an
acknowledged one tolerates, so the firing path cannot be quietly disabled again.

## Decisions

### 2026-08-20 — Fire on drift, allowlist deliberate variation

- **Chose:** The four-sibling allowlist pattern (fire by default, acknowledge with a cited
  reason).
- **Why:** It resolves the exact tension that motivated the soft default — legitimate
  host-local edits — without buying it at the price of silence. It is also the convention a
  reader of this repo already expects.
- **Rejected:** Keeping DRIFT non-firing but changing only the verdict word. Cheaper and it
  fixes the literal lie, but exit 0 is what automation reads, and a check that is right only
  for humans who read prose is half a check.
- **Rejected:** Firing with no allowlist. Would put permanent red on any host with a
  legitimate local variation, which is how a check gets ignored — the failure this task is
  correcting.

### 2026-08-20 — Keep `--strict` working

- **Chose:** Accept `--strict` as a no-op alias of the new default.
- **Why:** It appears in documentation and possibly in operator muscle memory; making it an
  unknown-flag error would turn a documentation lag into a hard failure for no benefit.
