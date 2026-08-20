---
id: T-2801
name: "Auto-deferred pickup envelopes can stick forever with no breadcrumb"
description: >
  P-043, a detailed two-bug report against the framework, has sat in
  .context/pickup/auto-deferred/ since 2026-06-08 with no breadcrumb — so
  `fw pickup promote-deferred` cannot evaluate it and `fw pickup auto-deferred list`
  prints `blocked-by=?` with no warning. Nothing surfaces a stuck envelope. One of its two
  bugs was independently re-discovered and fixed as T-2304; the other is still live today.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, pickup, g-063, observability]
components: []
related_tasks: [T-1425, T-2072, T-2304, T-2231, T-2691]
created: 2026-08-20
last_update: 2026-08-20T09:35:31Z
date_finished: null
---

# T-2801: Auto-deferred pickup envelopes can stick forever with no breadcrumb

## Context

The pickup pipeline routes an inbound envelope to `.context/pickup/auto-deferred/` when it is
blocked on a local inception task (G-059). T-1425 added `pickup_write_breadcrumb()`, which
drops a `<envelope>.breadcrumb.yaml` naming the blocking task, the reason, and the timestamp —
explicitly so that "operators (and `fw pickup auto-deferred list`) [can] trace why the
envelope was deferred instead of processed". T-2072 then added
`fw pickup promote-deferred`, which walks the directory and promotes any envelope whose
blocker has shipped, re-evaluated before every `process` tick.

Both halves depend on the breadcrumb. Neither checks that it exists.

```
$ fw pickup status
  Auto-deferred: 1

$ fw pickup auto-deferred list
  P-043-bug-report.yaml    blocked-by=?   reason=?   at=?

$ ls .context/pickup/auto-deferred/
  P-043-bug-report.yaml            # no .breadcrumb.yaml sibling
```

With no breadcrumb there is no blocking task to re-evaluate, so `promote-deferred` can never
promote it. The envelope is not *delayed*; it is **permanently stranded**, and the only
surface that shows it prints three question marks and no warning. `fw pickup status` counts
it as one more deferred item, indistinguishable from one deferred yesterday for a good reason.

### What was in it

P-043 is dated **2026-06-08** — two and a half months ago. It is not a stub; it is a careful
report of two framework bugs blocking every inception decision made through Watchtower or the
CLI, with file:line references, root-cause analysis, recommended one-line fixes, an impact
assessment and a reproducer.

Checking both against the framework as it stands today:

| | Reported | Status now |
|---|---|---|
| **BUG 1** — disposition gate accepts only `answered\|deferred\|dissolved`, rejecting the `resolved\|partial\|open` vocabulary the task template's own conventions produce | `update-task.sh:787` | **STILL LIVE** at `update-task.sh:791` |
| **BUG 2** — `python3 - <<HEREDOC` makes `__file__ == "<stdin>"`, so the dirname chain climbs to `/` and `lib.inception_decisions` is unimportable | `update-task.sh:578-588` | **FIXED** — by T-2304, independently |

The T-2304 fix is the recommended one, almost verbatim:
`sys.path.insert(0, os.environ.get("FRAMEWORK_ROOT") or ...)`. Nobody read P-043; somebody hit
the same wall months later, diagnosed it again, and fixed it again. That is the cost of a
stranded envelope stated precisely — not "a report was late" but "the same bug was solved
twice and its sibling is still open".

This is the G-063 class (a write-only sink nobody notices) turned inward. T-2231 built a
canary for the *inbound* `framework:pickup` topic and T-2691 stopped it firing on our own
filings; both guard the rail. Nothing guards the queue behind it.

## Approach

Two parts, matching the repo's habit of fixing the instance and then the blindness.

**1. Surface stuck envelopes.** `scripts/check-pickup-deferred-freshness.sh` walks
`.context/pickup/auto-deferred/*.yaml` and classifies each envelope:

- **STRANDED** — no `.breadcrumb.yaml` sibling. Un-promotable by construction, since
  `promote-deferred` has no blocking task to evaluate. FIRES.
- **STALE** — a breadcrumb exists but the envelope has been deferred longer than the
  threshold (default 30 days). The blocker may have shipped without the promotion path
  noticing, or may simply never ship. FIRES.
- **deferred** — breadcrumb present, within the threshold. Healthy; the mechanism is working.

Both firing classes print the envelope's `summary:` line, because the whole failure mode is
that nobody knows what is in the file.

**2. File BUG 1 upstream.** It is live, it is framework-owned (`update-task.sh` is vendored,
so a local edit is erased on the next re-vendor — G-062), and it has been waiting since June.
Posted to `framework:pickup` with the T-2304 outcome noted, so the framework can see that half
of this report was already validated by an independent re-discovery.

## Scope boundary

This detects and reports; it does not modify the queue. Promoting or discarding a stranded
envelope is a judgement about work — the framework's `promote-deferred` owns promotion, and a
human owns "this is obsolete, drop it". A checker that silently drained the queue would
replace one invisible loss with a faster one.

Fixing BUG 1 itself is likewise out of scope here: it is vendored code, and the last time a
framework fix was routed upstream from this repo without a local detector (G-007, T-229) it
was still unfixed five months later. The detector is the part that survives a re-vendor.

## Acceptance Criteria

### Agent
- [x] The check classifies an envelope with no breadcrumb as STRANDED and fires
- [x] The check classifies an envelope older than the threshold as STALE and fires
- [x] An envelope with a fresh breadcrumb is healthy and does not fire
- [x] Firing output prints each envelope's `summary:` so the operator sees what is stuck
- [x] The threshold is tunable
- [x] `--json` carries per-envelope class plus counts
- [x] An absent or empty `auto-deferred/` directory is healthy, not an error
- [x] Exit codes: 0 healthy, 1 firing, 2 tooling — a missing pickup dir is never a false clean
- [x] Fixtures build a scratch pickup directory and prove each class, host-independent (PL-213)
- [x] Run against this repo, it reports P-043 as STRANDED
- [x] BUG 1 is filed to `framework:pickup` with the T-2304 outcome noted

## Verification

bash tests/pickup-deferred-freshness-fixtures.sh

## RCA

**Symptom:** `fw pickup auto-deferred list` reports `P-043-bug-report.yaml blocked-by=?
reason=? at=?` and has done since 2026-06-08. The envelope contains a live framework bug
report that was never filed.

**Root cause:** the envelope has no `.breadcrumb.yaml` sibling. `promote-deferred` (T-2072)
resolves an envelope's blocking task from that file, so with no breadcrumb there is nothing to
re-evaluate and the envelope can never be promoted. Neither the deferral path nor the listing
path verifies the breadcrumb was written.

**Why structurally allowed:** the breadcrumb was introduced (T-1425) as an operator
convenience — "lets operators trace why the envelope was deferred" — and then quietly became
load-bearing when T-2072 made promotion depend on it. Nothing was added to check the
dependency it had acquired. And the one surface that would reveal the problem degrades
silently: `auto-deferred list` prints `?` for every missing field rather than saying the
breadcrumb is absent, so a stranded envelope and a well-formed one differ by three characters
in a list nobody runs.

**Prevention:** the check makes STRANDED a firing class, so an envelope that cannot be
promoted is loud rather than merely present, and STALE catches the case where the breadcrumb
exists but the blocker never ships.

**Not prevented:** the deferral path can still write an envelope without a breadcrumb — this
detects the result, it does not close the hole. `lib/pickup.sh` is vendored, so the fix is
upstream's; filed with BUG 1.

## Decisions

### 2026-08-20 — Detect, do not drain

- **Chose:** Report stranded and stale envelopes; never move or delete one.
- **Why:** Promotion is `promote-deferred`'s job and discarding is a human judgement about
  whether the work still matters. A checker that auto-drained the queue would turn a visible
  backlog into a silent one — the same trade this task exists to reverse.

### 2026-08-20 — File BUG 1 rather than patch it

- **Chose:** Post upstream; do not edit `update-task.sh` locally.
- **Why:** G-062 — vendored framework code, so a local edit is erased on the next re-vendor,
  and the edit would then be invisible in exactly the way T-2696 documented for the canary
  crontabs.
- **Noted:** the last framework fix routed upstream from this repo without a local detector
  (G-007, via T-229) was still unfixed five months later. That is the argument for building
  the detector alongside the filing rather than trusting the filing alone.
