---
id: T-2802
name: "Declare guard-layer membership on this branch's new source checks"
description: >
  charter-review's `run-guard-layer.sh` (T-2684) discovers members via a
  `# guard-layer: source` header marker. The three run-anywhere checks added on this branch
  carry no marker, so on merge they would join the tree as SKIP(unclassified) — visible but
  never executed by the one command meant to run the layer. Declare membership, and fix a
  stale reference to a conf file removed during the T-2698 reshaping.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, guard-layer, interop, pl-168]
components: []
related_tasks: [T-2684, T-2800, T-2801, T-2689, T-2692, T-2698]
created: 2026-08-20
last_update: 2026-08-20T09:42:28Z
date_finished: null
---

# T-2802: Declare guard-layer membership on this branch's new source checks

## Context

Every check written on this branch has carried the same admission in its own task record —
"this is an ad-hoc check, so it only helps someone who runs it" (PL-168: a check without a
trigger is dormant tooling). I logged it four times and did nothing about it, because the
obvious fix — building an aggregate runner — is **already claimed**: `worktree-charter-review`
has T-2684 in flight shipping `scripts/run-guard-layer.sh`.

Reading their runner rather than writing my own turned out to be the whole task. Membership
is not a list inside the runner; it is **declared by each check in its own header**:

```
# guard-layer: source [extra args...]
```

with `source` meaning *"safe to run anywhere: no live hub, no network, no host state"*. And
critically:

> A `scripts/check-*.sh` with NO marker is reported as SKIP(unclassified) rather than silently
> ignored — a forgotten marker is exactly the shipped-but-dark condition this script exists to
> surface, so it stays visible.

So the three run-anywhere checks added here would merge into a tree that has a runner, be
listed by it, and never be executed by it. Not broken, not silent — just permanently skipped.
One comment line each is the difference.

Fixture suites need nothing: `tests/*fixtures*.sh` are members by naming convention, so all
the suites written on this branch already join automatically.

## Approach

Add `# guard-layer: source` to the three checks that genuinely qualify, and deliberately not
to the two that do not.

| Check | Marker | Why |
|---|---|---|
| `check-task-id-collisions.sh` (T-2800) | **yes** | pure `git ls-tree` / `git diff` reads; no hub, no network, no host state |
| `check-pickup-deferred-freshness.sh` (T-2801) | **yes** | reads in-repo `.context/pickup/` only |
| `check-framework-tracking-drift.sh` (T-2689/T-2692) | **yes** | git plus the vendored framework dir, both in-tree |
| `check-cron-install-drift.sh` (T-2697) | **no** | reads `/etc/cron.d` — host state by definition, so not `source`. Their own copy carries no marker either; this agrees with them rather than diverging |
| `check-verification-pipefail.sh` (T-2693) | **no** | duplicate of charter-review's T-2775 implementation, which already carries the marker. Theirs wins at merge; marking mine would just create a second marked copy of the same check |

None of the three write a cron heartbeat, so no `--no-heartbeat` argument is needed — that
flag exists so a check invoked from the runner cannot mask a dead cron from the T-1723
meta-canary, and none of these have a cron to mask.

**Two of the three currently FIRE**, and that is intended rather than an oversight. The
collisions (10) and the stranded envelope (1) are real findings; a runner that goes red on
them is reporting the truth. The alternative — withholding the marker so the layer stays
green — would be scoring the metric instead of fixing the problem, which is the failure this
whole session has been about.

Also fixes a stale pointer: `check-framework-tracking-drift.sh` instructed the reader to
"Register it in `.context/cron/ondemand-checks.conf`" — a registry removed in the T-2698
reshaping, when the canary-status work was yielded to governance-canary-signal (their
`crontab_declares` derives the same fact from the crontabs themselves). The instruction is
replaced by the guard-layer marker that now actually governs this check, with a line saying
where the old registry went — deleting the mention outright would leave the next reader of
the episodic record wondering what happened to it.

## Acceptance Criteria

### Agent
- [x] The three run-anywhere checks carry `# guard-layer: source` in their header
- [x] The marker is placed where the runner looks for it (header, matching the convention on
      charter-review's own marked checks)
- [x] `check-cron-install-drift.sh` is deliberately NOT marked — it reads host state
- [x] `check-verification-pipefail.sh` is deliberately NOT marked — superseded at merge
- [x] The dangling `Register it in` sentence is gone — the removed conf is now referenced
      only as history, explaining why it went, rather than as a live instruction
- [x] Every marked check still runs correctly and its fixtures still pass
- [x] A marked check invoked with no arguments still behaves as before (the marker is a
      comment and changes no behaviour)

## Verification

bash tests/task-id-collision-fixtures.sh
bash tests/pickup-deferred-freshness-fixtures.sh
bash tests/framework-tracking-drift-fixtures.sh
bash tests/framework-dangling-ref-fixtures.sh
test "$(grep -c '^# guard-layer: source' scripts/check-task-id-collisions.sh scripts/check-pickup-deferred-freshness.sh scripts/check-framework-tracking-drift.sh | grep -c ':1')" = "3"
! grep -q 'Register it in' scripts/check-framework-tracking-drift.sh

## Decisions

### 2026-08-20 — Join their runner rather than build one

- **Chose:** Read `run-guard-layer.sh` on charter-review and conform to its marker contract.
- **Why:** The residual risk I had logged four times (PL-168, dormant tooling) already had an
  owner. Building a second runner would have been the fifth duplicate of this session, and
  the cheapest possible fix — one comment line per check — was available by reading first.
  This is what the T-2800 detector is for, used on myself.
- **Rejected:** Wiring my checks into a session-start hook independently. It would work, and
  it would guarantee a conflict with T-2684 over the same responsibility.

### 2026-08-20 — Mark the checks that fire, anyway

- **Chose:** Mark `check-task-id-collisions` and `check-pickup-deferred-freshness` even though
  both currently fire on this tree.
- **Why:** They fire because there are 10 real ID collisions and 1 real stranded envelope. A
  guard layer that is green because its members were withheld is worth less than one that is
  red for a stated reason.
- **Noted:** whoever sequences the merge should expect the layer to go red on these two, and
  the remedy is to resolve the findings, not to unmark the checks.
