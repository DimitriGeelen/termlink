---
id: T-2826
name: "canary-status reports 0 firing while two canaries are firing — strict > on same-second mtimes"
description: >
  canary-status.sh classifies FIRING as `log_mtime -gt heartbeat_mtime`. A canary touches its heartbeat BEFORE doing work and the cron appends the log after, so a fast canary lands both in the SAME second and strict `>` fails. Measured: charter-drift and waker-liveness both delta=0, both actively firing, both reported HEALTHY.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, canary, directive-2, pl-168]
components: []
related_tasks: [T-2172, T-2180, T-1723, T-2810]
created: 2026-08-21
last_update: 2026-08-21
date_finished: null
---

# T-2826: the verb that answers "are my canaries firing?" cannot say yes

## Context

Found while refreshing evidence for T-2389 (rail-dark relaunch). Running
`check-waker-liveness-freshness.sh` by hand:

```
waker-liveness canary: FIRING — 0 unwakeable LIVE agent(s), 4 dead waker(s), RAIL DARK
```

Minutes earlier, `canary-status.sh --quiet` had printed nothing at all — its convention for
"everything healthy". Run in full from the live checkout:

```
canary-status: 25 canary(ies) — 23 healthy, 0 firing, 2 stale
```

**Zero firing**, while the same output displays firing content under several entries:

```
[HEALTHY]  charter-drift-canary       hb=2026-08-20 08:17  log=2026-08-20 08:17
     ↳ check-charter-drift: FIRING — 40 live tool(s) drift from the charter
[HEALTHY]  waker-liveness-canary      ...
     ↳ best, forever at worst. Arm agents via the T-2388 launcher
```

It is printing the word FIRING inside a row it has classified HEALTHY, and totalling zero.

## Root cause

`scripts/canary-status.sh:153`:

```bash
elif [ "$log_mtime" -gt "$heartbeat_mtime" ]; then
    status="FIRING"
else
    # "prior firings are now resolved (healthy current state)"
    status="HEALTHY"
fi
```

The reasoning is sound — a log that has not grown since the last heartbeat holds only
historical entries — but the comparison is **strictly greater**, and the two timestamps are
routinely **equal**.

Every canary touches its heartbeat near the TOP of the run (T-1723: "prove this canary ran,
even on healthy/error cycles" — `check-charter-drift-freshness.sh:69`), then the crontab
appends the canary's stdout to the log when it finishes. A canary whose work takes under a
second — `charter-drift` only reads `termlink help --json` — writes both in the same second.

Measured on the live checkout, epoch mtimes:

| canary | log − heartbeat | classified | actually |
|---|---|---|---|
| `charter-drift` | **0** | HEALTHY | firing — 40 off-charter tools |
| `waker-liveness` | **0** | HEALTHY | firing — rail dark, 4 dead wakers |
| `stale-waker-code` | −604800 (7d) | HEALTHY | correct — historical |
| `stuck-claims` | −345599 (4d) | HEALTHY | correct — historical |

So the defect bites **exactly the canaries that fired most recently** — the ones an operator
most needs to see — and leaves the correct classification on the stale ones. A slower canary
gets a delta of 1s and is reported correctly, which is why this has survived: it looks like it
works, intermittently, for reasons unrelated to whether anything is wrong.

## Approach

`-gt` → `-ge`. Equality means the log was appended in the same second the heartbeat was
touched, i.e. **this run wrote to the log**, i.e. firing.

The healthy path is unaffected and that is worth stating: a canary that finds nothing writes
nothing, so its log keeps an OLD mtime while the heartbeat advances — `log < heartbeat`, still
HEALTHY. Only a run that actually appended can produce equality.

Add a fixture that pins the same-second case, because the existing suite cannot have covered it
— it is the case the bug is made of.

## Scope boundary

One comparison operator and its fixture. Does **not** change the taxonomy, the signal-bearing
line heuristic (T-2180), the stale threshold, or the discovery glob. Does **not** touch the
canaries themselves. Does **not** act on what the now-visible canaries report — `charter-drift`
(40 off-charter tools) and `waker-liveness` (rail dark) are separate pieces of work, and
T-2389 already records the rail-dark relaunch as an operator-timing decision.

## Acceptance Criteria

### Agent
- [x] `canary-status.sh` classifies a canary as FIRING when its log mtime **equals** its
      heartbeat mtime
- [x] A canary whose log predates its heartbeat is still HEALTHY — the historical-entries case
      the original reasoning was protecting
- [x] Fixtures pin both, including the same-second case that the bug consists of
- [x] Run against the live checkout, `charter-drift` and `waker-liveness` are reported FIRING
- [x] Exit code becomes 1 when a canary fires, so `--quiet` in cron stops being silent
- [x] No change to the taxonomy, signal-line heuristic, stale threshold or discovery

## Verification

bash tests/canary-status-firing-fixtures.sh
# The fix is present: equality counts as firing.
f=$(mktemp); grep -n 'log_mtime" -ge "\$heartbeat_mtime' scripts/canary-status.sh > "$f" 2>/dev/null; n=$(wc -l < "$f"); rm -f "$f"; test "$n" -ge 1

## Decisions

### 2026-08-21 — `-ge`, not a tolerance window

- **Chose:** change the comparison to `-ge`.
- **Why:** the obvious alternative — "treat a log within N seconds of the heartbeat as firing"
  — invents a threshold that would itself need justifying and would misfire on a canary that
  legitimately took N seconds. Equality is not an approximation here: a log can only share the
  heartbeat's second if this run appended to it, because a healthy run does not touch the log
  at all. The exact comparison is the correct one.

### 2026-08-21 — Fix it here even though both branches carry the bug

- **Context:** `worktree-governance-canary-signal` has the identical `-gt` at its line 229.
  Earlier in this session the canary-status work was deliberately yielded to that branch.
- **Chose:** fix here anyway.
- **Why:** it is a one-character defect in a non-vendored script that currently reports two
  actively-firing canaries as healthy. Yielding ownership of a file is not a reason to leave a
  known lie in it, and both branches need the same change — so this is a shared fix rather than
  a competing implementation. If their version wins at merge, the fix is one character to
  re-apply and this task's fixture will catch its absence.

## Outcome — 2026-08-21

Live checkout, same canary state, before and after the one-character change:

```
before:  25 canary(ies) — 23 healthy, 0 firing, 2 stale
after:   25 canary(ies) — 20 healthy, 3 firing, 2 stale
```

**Three canaries were invisible**, not two — the third only became apparent once the
classification worked:

| canary | was | signal |
|---|---|---|
| `charter-drift` | HEALTHY | **40 live tools drift from the charter** (off-charter social-analytics, not deprecated) |
| `waker-liveness` | HEALTHY | the G-069 **"0 wakers"** state — every DM waits on the ~15s poll floor at best, forever at worst |
| `substrate-preflight` | HEALTHY | 5 pass, **1 warn** |

`charter-drift` is the one worth reading twice. T-2483 built that canary specifically so
off-charter breadth could not re-accrete after P4 pruned 52 tools, and CLAUDE.md records its
state as "214 live tools scanned, 0 off-charter". It has been reporting **40** — and the verb
an operator runs to check on it has been answering "0 firing".

Mutation-tested: reverting `-ge` to `-gt` reddens 2 of the 6 fixture legs; restoring returns
6/6. The over-correction guard (a canary that fired last week and has been quiet since must
stay HEALTHY) passes both before and after, which is the leg that matters for not turning every
historical firing into a permanent red light.

**Not acted on here.** The three findings are separate work: `charter-drift`'s 40 tools is a
charter-review question, `waker-liveness` is T-2389's operator-timing decision, and
`substrate-preflight`'s warn is a one-line environment check. Making them visible was the
deliverable.
