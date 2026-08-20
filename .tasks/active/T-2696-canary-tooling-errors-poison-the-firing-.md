---
id: T-2696
name: "Canary tooling errors poison the firing log and misread as drift"
description: >
  Canary crontabs in git wired `2>&1`, merging the exit-2 tooling-error stream into the log
  whose only documented meaning is "drift detected" — so a transient network blip pins a
  canary FIRING forever. The installed /etc/cron.d copies already split stderr to a
  `.log.stderr` companion, but that fix was never committed and nothing ever read the files.
  Commit the wiring back to git and make canary-status.sh the reader.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, canary, observability, reliability]
components: []
related_tasks: [T-1696, T-2172, T-2180, T-2688, T-2557, T-2561, T-2689, T-2692]
created: 2026-08-20T06:45:22Z
last_update: 2026-08-20T07:20:00Z
date_finished: null
---

# T-2696: Canary tooling errors poison the firing log and misread as drift

## Context

Seventeen canaries share one documented contract, repeated verbatim in CLAUDE.md for each
of them: **"Empty log = healthy. Any entry = operator action required."**

Every canary crontab in git wires that contract like this:

```
13 7 * * * root cd /opt/termlink && bash scripts/check-mirror-freshness.sh --quiet \
    >> .context/working/.release-mirror-canary.log 2>&1
```

`2>&1` is the defect. The scripts distinguish three outcomes by exit code — 0 healthy,
1 drift/finding, 2 tooling error — and print findings on **stdout** but tooling errors on
**stderr**. The redirect collapses that distinction: an exit-2 tooling error is written into
the log whose only meaning is "a real problem was detected".

This was live at the start of the session:

```
$ bash scripts/canary-status.sh --quiet
canary-status: 1 canary(ies) need attention (1 firing, 0 stale, threshold 48h)
  FIRING      release-mirror-canary
             ↳ error: origin HEAD empty

$ bash scripts/check-mirror-freshness.sh
GitHub mirror: synced
  → exit 0
```

The canary reported a mirror problem. There was no mirror problem. Some run could not reach
OneDev, `die "origin HEAD empty"` fired, and because `die()` runs *after* the heartbeat
touch, the log ends up newer than the heartbeat — which is exactly `classify()`'s definition
of FIRING. It stays FIRING until a human truncates the log by hand, a remediation documented
nowhere.

### Why this matters more than one stale line

The harm is not the wrong line, it is what the wrong line teaches. An operator who checks a
FIRING canary, finds nothing wrong, and has no way to clear it learns that canaries cry
wolf. That is the same erosion T-2693 documented for the P-011 gate: *"a gate that blocks
incorrectly 150 times teaches the operator that P-011 failures are noise"*. A canary set
people have learned to skim is a canary set that is no longer load-bearing — and this one
guards G-058, the 16-day silent mirror failure that shipped three release tags to nowhere.

The convention is also **already settled in this repo, in the opposite direction**. T-2557's
session-control canary states it explicitly:

> selftest exit 2 (tooling — hub down / no tmux / dep missing) → canary exit 2
> (non-firing — that is a substrate/environment fault, NOT a verb-4 regression).
> **This split keeps a firing log meaningful:** it fills ONLY when session control genuinely
> broke, never on a transient hub-down.

T-2557 got it right at the *exit-code* layer. The redirect then undid it at the *logging*
layer, for all seventeen. PL-259 names the same rule generally — never collapse a structured
error into an unstructured one, which is what merging stderr into stdout does.

### What the investigation actually found

The intended fix was to change the redirect. Diffing one crontab against its installed copy
first showed that **someone had already done exactly that, on the deployed files only**:

```
$ diff <git HEAD> /etc/cron.d/termlink-release-mirror-canary
< ... >> .context/working/.release-mirror-canary.log 2>&1
> ... >> .context/working/.release-mirror-canary.log 2>> .context/working/.release-mirror-canary.log.stderr
```

Uniform across all 22 canary cron lines, `.log.stderr` every time, and only the two
genuinely non-canary jobs (presence-sweep, heartbeat) left merged — a deliberate, correct,
consistently-applied fix. It was never committed. So:

- **git carried the broken form.** The live fix existed on one host's disk and nowhere else —
  the same non-recoverability class as T-2689/T-2692 (`lib/bvp.sh` running but untracked).
  A reinstall from git would have silently regressed it.
- **`check-cron-install-drift.sh` (T-2561) saw it and said "healthy".** It classifies DRIFT
  as a non-firing WARNING by default, so it had been reporting 3 matching / **21 drifting**
  while exiting 0. Twenty-one crontabs diverged from their source of truth, surfaced only as
  a line nobody had to act on.
- **Nothing read the `.stderr` files.** `canary-status.sh` globs `.log` and `.heartbeat`
  only. So the errors were being carefully written to files with no reader — the G-063
  write-only-sink class, the very failure the unconfirmed-delivery canary (T-2295) exists to
  catch on a different rail.

Three separate blind spots stacked: the fix wasn't recoverable, the detector that saw it
didn't insist, and the output had no consumer. The canary reported drift that did not exist
while the machinery to report it correctly was already in place and unread.

## Approach

1. **Reconcile git to the installed wiring** rather than invent a competing convention.
   `.log.stderr` is what is actually running; introducing `.err` would have orphaned the
   existing files and added a 22nd drift. The transform is applied to the git source by
   regex and then **byte-compared against the installed file**, so this is a verified
   reconciliation, never a blind copy of whatever is on disk. Before writing anything, all
   21 were confirmed to differ *only* in the redirect — no other installed edit gets
   imported by accident.
2. **Make `canary-status.sh` the reader.** A **TOOLING** state: the findings log is clean,
   but the `.log.stderr` companion has content no older than the heartbeat.

TOOLING counts toward "needs attention" and still exits 1. It is not a way to make the
problem quiet: a *persistent* tooling error means the canary cannot run at all, which is the
canary going dark and is strictly worse than the drift it was watching for. What changes is
that the operator is told which of the two they are looking at, and how to clear it.

A canary with a real finding stays FIRING regardless of what is in `.log.stderr`, and a
stale heartbeat still reports STALE. TOOLING is reachable only from HEALTHY, so it can never
mask a live finding or a dead cron.

### Why no reinstall is needed

Because git is being brought to the deployed state rather than the other way round, there is
no operator step and no window where this is worse than the status quo. It also takes
`check-cron-install-drift.sh` from 3 matching / 21 drifting to **24 matching / 0 drifting** —
clearing a backlog that predates this task.

## Acceptance Criteria

### Agent
- [x] Every canary cron line in git routes stderr to its `.log.stderr` companion — no canary
      line still merges the two streams
- [x] The git source is byte-identical to the installed `/etc/cron.d` copy for every file
      touched, verified programmatically before writing
- [x] Only the redirect differs between git and installed — confirmed for all 21 before
      import, so no unrelated on-disk edit is pulled in
- [x] `classify()` returns a distinct `TOOLING` status when the findings log is clean but the
      `.log.stderr` companion has content newer than the heartbeat
- [x] A canary with a real finding stays FIRING even when `.log.stderr` also has content
      (findings dominate — the fail-safe direction)
- [x] A stale heartbeat still reports STALE even with `.log.stderr` content (a dead cron
      dominates)
- [x] TOOLING counts toward the non-zero exit — reported, never silenced
- [x] TOOLING renders its own remediation, distinct from the FIRING and STALE hints
- [x] TOOLING surfaces the `.log.stderr` text, not the empty findings log
- [x] `--json` carries the TOOLING state, a `tooling` summary count, and per-canary `err_size`
- [x] `--quiet` renders TOOLING rows alongside FIRING/STALE
- [x] Absent `.log.stderr` classifies exactly as before — no regression
- [x] Fixtures prove every classification from scratch logs, host-independent (PL-213)
- [x] `check-cron-install-drift.sh` reports 0 drift after the change

### Human
- [ ] Clear the one stale mirror-canary log line (optional hygiene — it is already
      classified HEALTHY, this just removes the misleading historical text).
      **Steps:**
      1. `cd /opt/termlink && bash scripts/check-mirror-freshness.sh`
      2. Only if that prints `GitHub mirror: synced` and exits 0:
         `cd /opt/termlink && : > .context/working/.release-mirror-canary.log`
      3. `cd /opt/termlink && bash scripts/canary-status.sh`
      **Expected:** step 3 shows release-mirror-canary HEALTHY with no `↳ error:` line.
      **If not:** step 1 failing means the mirror genuinely drifted since this was written —
      do NOT clear the log; follow the T-1695 mirror-restore playbook instead.

## Verification

bash tests/canary-tooling-class-fixtures.sh
bash scripts/canary-status.sh --json > /dev/null
bash scripts/check-cron-install-drift.sh
test -z "$(grep -lE '>> \.context/working/\.[a-z-]*(canary|aliveness)[a-z-]*\.log 2>&1' .context/cron/*.crontab)"

## RCA

**Symptom:** `canary-status.sh` reported `FIRING release-mirror-canary — error: origin HEAD
empty` while `check-mirror-freshness.sh` run ad-hoc reported `synced` and exited 0.

**Root cause:** The canary crontabs in git redirect `2>&1`, merging the exit-2 tooling-error
stream into the log file whose sole documented meaning is "exit-1 finding". `die()` touches
the heartbeat before writing the error, so the resulting log is newer than the heartbeat,
which is `classify()`'s FIRING predicate. One transient network failure therefore pins the
canary FIRING permanently.

**Why structurally allowed:** three independent blind spots, any one of which would have
caught it.

1. *The exit-code contract is discarded at the wiring.* Every script implements 0/1/2
   correctly; the shell redirect that runs them throws the distinction away. Crontabs are
   reviewed as cron syntax, not as part of the canary's semantics.
2. *The fix existed but was not recoverable.* All 22 installed cron lines already split
   stderr to `.log.stderr` — correct, uniform, and never committed. git kept the broken form,
   so the working behaviour lived on one host's disk and a reinstall from source would have
   regressed it. Same class as T-2689/T-2692.
3. *The detector that saw it did not insist.* `check-cron-install-drift.sh` (T-2561) grades
   DRIFT as a non-firing WARNING, so it exited 0 and printed "healthy" while reporting 21 of
   24 crontabs diverged from source. A warning nobody must act on is how a 21-file drift
   backlog accumulates unnoticed.

And downstream of all three: nothing read the `.stderr` files. `canary-status.sh` globs
`.log` and `.heartbeat` only, so the tooling errors were written to a sink with no reader
(G-063).

**Prevention:** git now matches the deployed wiring, so the split is recoverable and a
reinstall cannot regress it. `canary-status.sh` reads the companion, so the errors have a
consumer. The fixtures pin both directions of the classification — in particular that a real
finding always dominates — so a future edit cannot widen TOOLING into swallowing drift. The
`2>&1` shape is asserted absent from every canary cron line, so copy-pasting an old crontab
re-fires the suite.

**Not prevented (filed for follow-up):** `check-cron-install-drift.sh` reporting "healthy"
while 21 files drift is its own defect, and one this task only cleared rather than fixed —
the next divergence will accumulate just as quietly. Worth its own task on whether DRIFT
should fire by default.

## Decisions

### 2026-08-20 — Reconcile git to the installed wiring, not the reverse

- **Chose:** Bring the git source up to what `/etc/cron.d` already runs, using `.log.stderr`.
- **Why:** The deployed form is correct and uniform; the bug is that it was never committed.
  Inventing `.err` would have orphaned live files, added a 22nd drift, and left the real
  defect (an uncommitted fix) untouched.
- **Rejected:** A fresh `.err` convention. It was implemented first, then backed out on
  discovering the installed state — the diff was worth reading before writing.
- **Guard:** Verified up front that all 21 differ *only* in the redirect, and byte-compared
  each rewritten file against its installed copy before saving. "Match what is deployed" is
  only safe when you have checked what is deployed.

### 2026-08-20 — Split the streams rather than match prose

- **Chose:** Rely on stdout-is-findings / stderr-is-errors.
- **Why:** The distinction already exists structurally; preserving it needs no heuristic.
- **Rejected:** Classifying by log content. Attempted first, abandoned on evidence —
  `task-finalization canary:` prefixes both a healthy line and a parse error, so no anchor is
  simultaneously narrow enough to protect real findings and wide enough to catch tooling
  errors. Reconstructing by pattern a distinction that structure already provides is how
  detectors acquire false negatives.

### 2026-08-20 — TOOLING still exits non-zero

- **Chose:** TOOLING counts toward "needs attention".
- **Why:** A persistent tooling error means the canary cannot run — it is dark, which is
  worse than the drift it was watching for. Making it quiet would convert a loud wrong answer
  into a silent one, trading a Directive-#2 violation for a bigger one.
- **Rejected:** Treating TOOLING as healthy. Attractive because it clears the red, which is
  exactly why it is wrong: the goal is an honest signal, not a green one.

### 2026-08-20 — Findings dominate a tooling error on the same canary

- **Chose:** FIRING and STALE both beat TOOLING; TOOLING is reachable only from HEALTHY.
- **Why:** The predicate's failure direction has to be over-reporting. The alternative would
  let a transient error mask a real finding on the same canary — converting a false-positive
  fix into a false-negative, the one outcome worse than the bug being fixed.
- **Rejected:** Reporting both states, or letting the more recent stream win. Both are
  defensible and neither is worth a second axis of operator judgment on a status line whose
  entire job is to be skimmable.
