# T-1449 Inception — Date-Triggered Revisit Mechanism for DEFER Inceptions

**Status:** Inception, Recommendation GO Phase 1, awaiting human decision
**Created:** 2026-05-02
**Source gap:** G-053 (concerns.yaml)
**Related:** T-1425 (DEFER source), T-1428 (sentinel testbed), T-1448 (auto-finalize bug)

## TL;DR

The framework captures DEFER decisions and creates sentinel tasks to revisit them, but the revisit *date* lives only in description prose — nothing scans for it, nothing surfaces ripe revisits. T-1428's audit fires 2026-05-14 with empty placeholder ACs and no surfacing mechanism. **Recommendation: GO Phase 1** — `revisit_at` frontmatter field + daily cron + `fw task revisit-due` CLI + T-1428 AC backfill (~120 LOC + 1 task-file edit). Defer optional separate register to Phase 2 unless Spike 1 reveals cross-task patterns.

## Dialogue Log

### 2026-05-02 — User asks "are we wired to capture the date we need to make a sensible analysis to support a later decision?"

Triggered after T-1425 48h soak synthesis decided DEFER with sentinel T-1428 firing 2026-05-14. Inspection revealed:

- T-1428 created 2026-04-30 with empty placeholder ACs (`[First criterion]`, `[Second criterion]`)
- The audit recipe lives only in T-1428's description prose
- Cron registry has 11 active jobs, none date-triggered per-task
- `assumptions.yaml` and `decisions.yaml` exist but have no date-trigger semantics
- Tasks have `horizon: now/next/later` but no `revisit_at: <date>` field

**Conclusion:** *Capture* mechanism exists (sentinel tasks). *Surface* mechanism does not.

### 2026-05-02 — User authorized capturing as G-053 + scoping inception

User: "yes" to "capture as a separate gap (G-053) and/or scope a build task to fix it." G-053 written to concerns.yaml. T-1449 inception captured with placeholder body.

### 2026-05-02 — User noticed T-1449 not in `/approvals`

User correctly observed that T-1449 wouldn't surface in Watchtower's approval queue because (a) status was `captured` not `started-work`, and (b) inception body was empty placeholders — no Recommendation for the human to decide on. Filled in problem statement, assumptions, exploration plan, scope fence, GO/NO-GO/DEFER criteria, and Recommendation. Flipped to `started-work`. Now visible in /approvals (7 mentions).

### 2026-05-02 — Recommendation gate parser quirk

`fw inception decide` rejected the Recommendation line `**Recommendation: GO — Phase 1 ...**` because `audit_inception_recommendation` (lib/task-audit.sh:117) expects the literal pattern `**Recommendation:**` (closing bold immediately after the colon) followed by content in plain text. Fixed: closed the bold after `Recommendation:` so the content is plain text.

## Problem Framing

When an inception decides DEFER (or any task names a future revisit date), the date lives only in description prose. If the date passes silently, the deferred decision stays open indefinitely — the framework cannot proactively surface "this is ripe for re-verdict."

The same shape applies to:

- **Foundation soak audits** (T-1428 fires 2026-05-14)
- **Amend windows on solo syntheses** (T-1425's 14d window ends 2026-05-14)
- **Post-fix audits** (G-053 itself, once a fix lands, when do we audit it worked?)
- **"2-week check" patterns** scattered across the project

## Audit of Recent Revisit-Pattern Instances (pre-Spike 1)

Quick grep for revisit-pattern vocabulary turns up:

| Source | Revisit trigger | Captured how? |
|---|---|---|
| T-1425 (DEFER) | 2026-05-14 (T-1428 fires) | Prose in T-1428 description |
| T-1425 (amend window) | 2026-05-14 (14d from 04-30T21:18Z) | Prose in T-1425 Recommendation |
| T-1428 (sentinel) | 2026-05-14 | Prose in description, ACs empty |
| T-1448 (G-052 auto-finalize) | After framework fix | No date — needs spec |
| G-053 (this gap) | After Phase 1 ships | No date — needs spec |
| T-1438 (.121 launcher unblock) | When operator identifies launcher | Event-driven, not date |

That's already 4 date-triggered instances and 2 event-triggered ones. Spike 1 will refine but the floor is established: this is a real recurring pattern, not a one-off.

## Recommendation Detail

**GO Phase 1** scope (~120 LOC + 1 task-file edit):

1. **Frontmatter field** — `revisit_at: <ISO-date>` and optional `revisit_evidence_needed: <one-line>` on tasks. Backward-compatible; existing tasks have no field and continue working.
2. **Daily cron** — `revisit-due-scan.sh` runs ~07:00, scans `.tasks/active/*.md` for `revisit_at <= today`, writes `.context/working/.revisits-due.txt` consumed by handover banner + Watchtower home page.
3. **CLI verb** — `fw task revisit-due` lists ripe revisits on demand (no cron dependency).
4. **T-1428 AC backfill** — extract the audit recipe from T-1428's description prose into checkbox ACs, so the 2026-05-14 fire is mechanically checkable.

**Phase 2 deferred:** `deferred-decisions.yaml` register (only if Spike 1 reveals cross-task patterns); multi-criterion expressions; event-driven triggers. These are different problems with different mechanisms.

**Why not DEFER this inception?** The DEFER criterion ("T-1428 fires successfully first") creates a chicken-and-egg — the mechanism that needs to surface T-1428 has to land *before* T-1428 fires. Going Phase 1 now and using T-1428 as the soak test inverts that correctly.

**Why not NO-GO?** The NO-GO criterion ("only 1-2 instances found") is unlikely — pre-spike audit already shows ≥4 date-triggered instances.

## Downstream Build Tasks (provisional, scope after Spike completion)

| # | Deliverable | Estimated size | Sequencing |
|---|---|---|---|
| 1 | `revisit_at` frontmatter field + template update | ~30 LOC | First (others depend on it) |
| 2 | `revisit-due-scan.sh` cron + handover banner | ~50 LOC | Second |
| 3 | `fw task revisit-due` CLI | ~40 LOC | Third (independent of #2) |
| 4 | T-1428 AC backfill | ~10 lines markdown | Anytime; can land first as a no-code wedge |

Total Phase 1: ~120 LOC, 4 small tasks. Each fits one session.

## Sentinel for This Inception

Once Phase 1 ships, T-1428 itself becomes the soak test: if T-1428 fires correctly on 2026-05-14 (surfaces in handover banner + Watchtower), Phase 1 is validated. If T-1428 fires silently or doesn't fire, Phase 1 has a bug to fix.

This is recursive — using the mechanism on its own first instance is the cheapest possible end-to-end validation.

## References

- `concerns.yaml` G-053 — the gap this inception fixes
- `lib/inception.sh:353-369` — the gate that requires this Recommendation section
- `lib/task-audit.sh:117-158` — the regex that the Recommendation line must match
- `.agentic-framework/.tasks/templates/zzz-default.md` — task template that needs `revisit_at` field added
- T-1425 RFC artifact `docs/reports/T-1425-agent-contact-pattern-rfc.md` — the DEFER instance that triggered this
