# T-3006 — Task completion-rate drop investigation (C-42)

**Task:** T-3006 (inception, arc-009, S-29d/C-42)
**Source finding:** consolidated value review C-42 — "Task completion fell ~8× (609/mo → ~75/mo)
while commits held at 447/30d."
**Measured:** 2026-09-20, over `.tasks/completed/` `date_finished` (0 missing post-T-2203) and
`git log` monthly buckets since 2026-02.

## Monthly series

| month | completed | commits | distinct T-IDs in commits | commits per completion |
|---|---|---|---|---|
| 2026-03 | 562 | 1,054 | 570 | 1.9 |
| 2026-04 | 609 | 2,114 | 623 | 3.5 |
| 2026-05 | 412 | 1,090 | 482 | 2.6 |
| 2026-06 | 359 | 1,077 | 433 | 3.0 |
| 2026-07 | 165 | 594 | 198 | 3.6 |
| 2026-08 | 279 | 1,055 | 356 | 3.8 |
| 2026-09 (20d) | 51 | 206 | 89 | 4.0 |

Cross-checks: `git rev-list --count --since="30 days ago"` = **450** (matches C-42's 447);
August alone = 1,062.

## Findings

**F1 — The drop reproduces, but so does a commit drop C-42 missed.** Sep run-rate ≈ 77
completions/mo vs the April peak 609 — the ~8×. But commits fell from 2,114 (Apr) to a ~310/mo Sep
run-rate — **~6.8×, nearly proportional**. C-42's premise "commits held at 447/30d" is a window
artifact: the review compared the *current* 30-day commit window against the *April* completion
peak without checking April's commit volume. Work volume and completions fell **together**.

**F2 — Task size grew, but modestly in the drop window.** Commits-per-completion doubled across
the whole span (1.9 in March → 4.0 in September) — tasks are ~2× bigger than in March. But from the
April peak to September it moved only 3.5 → 4.0 (+14%). Size growth explains the small residual
between the 6.8× commit drop and the 8× completion drop, not the drop itself.

**F3 — Gate friction is a real stock, not the flow driver.** 29 active tasks have every Agent AC
ticked with `owner: human` (crude parse; the review-queue surface reports 75 with unchecked Human
ACs, 57 >30 days — T-2940/D2). That stock suppresses the completed/ count by at most ~1–1.5 months
of Sep-rate completions in total, accumulated over months — material for ledger truthfulness,
nowhere near an 8× monthly flow change.

**F4 — Abandonment is negligible.** Exactly 1 started-work task has gone >30 days without update
(T-2486, an inception awaiting its own review).

## Attribution

The completion-rate drop is **~85–90% total-throughput decline** (fewer commits, fewer distinct
tasks touched per month — the project simply did less per month after April, and monthly volume is
non-monotonic: Jul 165 < Aug 279), **~10% task-size growth**, **small residual** review-queue
accumulation, **~0%** abandonment. April was also the historic peak — measuring the drop from the
peak maximizes it; Mar–Jun average (~485/mo) → Sep gives ~6×.

## Recommendation: NO-GO on process change

C-42's implicit premise (completions decoupled from work volume ⇒ something in the process is
eating closes) is **falsified**: the two series fell together. No gate/process change is warranted
by this data. What remains actionable is already owned elsewhere: the review-queue stock is
T-2940/D2's finding, and per-month throughput is an operator capacity fact, not a framework defect.
Optional cheap follow-on (not required): a completions-vs-commits monthly panel in `fw metrics` so
the next reviewer sees both series at once instead of re-deriving them.
