# T-3166 — arc-008's open backlog vs the 8 live audit warnings

**Phase:** inception (exploration). **Recommendation at filing:** DEFER — no evidence yet.
**Question (one):** of arc-008's open tasks, which are actionable from a code-only session here,
and which of the 8 warnings `fw audit` currently emits are not covered by any task at all?

## Why this arc, and why now

arc-008 is *"Audit and doctor finding remediation"*. Its headline mechanic is specific and
falsifiable:

> An operator runs `fw audit` and `fw doctor` and reads a clean report: zero failures, and every
> remaining warning either fixed or explicitly acknowledged with a cited reason, instead of the
> same findings being re-reported unread every session.

So the arc has a real finish line, and today's audit says how far away it is: **Pass 43 / Warn 8
/ Fail 0**. Zero FAILs means half the bar is already met. The remaining work is entirely the 8
warnings, and the bar per warning is *fixed **OR** acknowledged with a cited reason* — not
necessarily fixed. That distinction is what makes the arc closeable rather than open-ended.

The trend data is why this is worth doing now. The audit's own 14-day trend analysis reports five
of these warnings recurring **10 times each** — every audit for ten consecutive runs. A warning
re-reported unread ten times is exactly the alarm-fatigue failure T-2818 and T-2833 document, and
it is measurably happening in the surface whose job is to notice things.

## Method

Read-only. For each of the 8 warnings in `.context/audits/2026-09-26.yaml`, find whether an
arc-008 task claims it; for each open arc-008 task, classify actionable / blocked / human-owned.
**No fixes under this task** — this is the examination. Anything fixable becomes its own build
task, per "one task = one deliverable".

## The 8 warnings (from 2026-09-26), and who claims each

| # | warning | claimed by | state |
|---|---|---|---|
| 1 | onboarding-seed corpus refs — NOT EVALUATED | T-3097 | completed; filed upstream offset 178 |
| 2 | free driver F-ORCH `retire_when` appears met | T-3126 | captured, **owner: human** |
| 3 | PROJECT_ROOT resolution of framework assets (OBS-097) — NOT EVALUATED | T-3098 | completed; same filing |
| 4 | stale-slice-references (L-417) — NOT EVALUATED | T-3099 | completed; same filing |
| 5 | 93 GO-scope-not-propagated inceptions of 176 | T-3100 | captured, **owner: human** |
| 6 | Fabric: 297/535 cards have no edges | **T-3009 — tagged `arc:arc-009`, not arc-008** | captured; premise stale |
| 7 | Fabric drift: 7 source files have no card | T-3131 | started-work, agent |
| 8 | Fabric: 10 cards point at files no watch pattern covers | T-3103 | captured, agent |

**IW-1 answered:** no warning is entirely unclaimed. But #6 is claimed from **outside the arc**
(T-3009 sits in arc-009), so arc-008's membership under-represents the scope of its own headline
mechanic. Anyone closing arc-008 on the strength of its member list would leave a live audit
warning behind.

## arc-008's actual size

I came into this expecting ~17 open tasks. That estimate was wrong by roughly 4x.

Replicating the audit's own membership predicate exactly — `arc:arc-008` present on the `tags:`
line, OR an anchored `arc_id:` line, across `active/` + `completed/`:

| predicate | count |
|---|---|
| `arc:arc-008` on the `tags:` line | 57 |
| anchored `arc_id: arc-008` line | 15 |
| **audit predicate (union)** | **72** |
| of those, `status: work-completed` | 40 |
| **ratio** | **0.5556** |

Active breakdown: 46 files mention the arc — 3 `work-completed` (all three carry `date_finished`,
so T-193 partial-complete **by design**, confirmed by `check-stranded-finalized-tasks.sh`: 0
stranded, 72 partial-complete), 9 `started-work`, 34 `captured`. 27 of the 46 are `owner: human`.

A hypothesis worth recording as refuted: I expected the audit's `^tags:\s*(.*?)$` regex to miss
block-style YAML tag lists and so undercount membership. Measured: **0 tasks** carry the tag off
the `tags:` line. No blindness there.

## Findings

**F1 — T-3117's premise has expired.** It was filed as *"Arc arc-008 hit 80% closure threshold
(25/30) without explicit fw arc close"*. Today the same computation gives **40/72 = 0.556**, well
below the 0.80 threshold, and today's audit emits **no** arc-closure warning for arc-008. The task
is not wrong about what it saw; the condition it describes no longer holds.

**F2 — and it expired for a structural reason, not a random one. arc-008 is self-diluting.** Its
stated job is that *"every individual FAIL and WARNING becomes its own governed task"* — so the
arc is a permanent intake queue, and every new audit finding grows its denominator. Between
T-3117's snapshot and now, membership went 30 → 72 while completions went 25 → 40: the numerator
grew 60%, the denominator 140%. **A closure-percentage threshold is category-inappropriate for an
arc whose purpose is to keep absorbing work.** It can only cross 80% during a quiet spell, and
when it does the check pressures an operator to `fw arc close` an arc that is by design never
finished. The check is not measuring what its name suggests for this arc.

**F3 — the other four closure tasks are NOT stale.** Worth stating explicitly so nobody
batch-closes them alongside T-3117:

| task | arc (as reported) | file | claimed | today | still fires? |
|---|---|---|---|---|---|
| T-3117 | arc-008 | `arc-008` | 25/30 = .833 | 40/72 = .556 | **no** |
| T-3118 | arc-010 | `arc-011` | 17/19 = .895 | 20/21 = .952 | yes |
| T-3119 | arc-001 | `arc-parallel-substrate` | 42/45 = .933 | 42/45 = .933 | yes |
| T-3120 | arc-002 | `arc-substrate-fitness` | 11/12 = .917 | 11/12 = .917 | yes |
| T-3121 | arc-005 | `mcp-slimming` | 2/2 = 1.0 | 3/3 = 1.0 | yes |

**F4 — `arc-011.yaml` declares `id: arc-010`.** A filename/id off-by-one. It *functions*, because
the membership predicate unions slug and id, but every audit line and task title says "arc-010"
while every human-facing surface — the handover banner, `fw arc focus` — says arc-011. That cost
me a wrong turn during this examination and will cost the next reader the same one.

**F5 (the sharpest) — arc-008's success condition is unreachable with the current instrument.**
The headline mechanic requires every remaining warning to be *"fixed **or explicitly acknowledged
with a cited reason**"*. There is no acknowledgement mechanism for audit warnings. The only
silencing available is a per-check global kill switch — `FW_RETIRE_WHEN_ADVISORY=0` is the one
example in the file — which suppresses a whole check for everyone with no reason recorded
anywhere. That is the opposite of an acknowledgement: it removes the finding *and* the record.

This repo solved exactly this problem twelve times over on the other side of the guard layer. Every
static check carries a git-tracked `.context/checks/<name>-allowlist` whose entries are **counted
and reported but do not fire**, each with a cited reason and a statement of what would let the
entry be deleted (T-2681, T-2483, T-2680). The audit — the oldest and most-read surface — never got
one. So three warnings on today's list (#1/#3/#4) *are* acknowledged in substance, with the reason
filed upstream at offset 178 and printed in the audit's own Mitigation line, and they still land in
the operator's "Warn 8" count indistinguishably from unexamined ones.

**F6 — nothing performs the verification the arc prescribes.** arc-008's description says
outright: *"Verification of a task is the re-run of the audit in the next cycle, not
self-assertion."* Nothing re-checks a filed task against the current audit. T-3117 and T-3009 are
both evidence: each carries hard numbers in its own title (25/30, 326/491) that the next audit run
already contradicted (40/72, 297/535), and neither noticed. A task filed from a measurement has no
way to learn its measurement expired, so the backlog accumulates tasks whose premises are gone —
which is itself a species of the alarm-fatigue the arc exists to end.

## Recommendation

**GO, narrowly, on two things; and do not work the backlog as filed.**

1. **Close T-3117 as dissolved** — premise expired (F1), and the metric is category-inappropriate
   for an intake arc (F2). Refresh T-3009's title/body to the current 297/535 rather than closing
   it; the underlying warning is live.
2. **Build the audit-warning acknowledgement ledger** — the missing half of arc-008's own success
   condition (F5), modelled directly on the twelve existing `.context/checks/*-allowlist` files:
   git-tracked, cited reasons, entries counted and reported on the clean path so a green never
   conflates "clean" with "acknowledged". **Constraint: `audit.sh` is vendored (G-062)**, so the
   audit cannot be taught to read a ledger locally. The half that survives a re-vendor is a local
   post-processor over `.context/audits/<date>.yaml` that classifies each warning as
   acknowledged / unexamined and renders the honest two-number summary. File the in-audit support
   upstream; build the wrapper here.
3. **Do NOT bulk-work the 34 `captured` tasks.** 27 of the 46 active are `owner: human`, and the
   two largest (T-3100's 93 GO-scope inceptions, T-3126's F-ORCH driver retirement) are human
   judgement — value drivers are §ACD sovereignty-gated, so F-ORCH is not agent work at any
   confidence level.
4. **The three Fabric warnings (#6/#7/#8) are the only mechanically-actionable ones from here**,
   and they are genuinely small (`fw fabric scan`, `fw fabric enrich`, widening
   `.fabric/watch-patterns.yaml`). They belong in one build task, not three, since all three are
   symptoms of the registry and the watch patterns having drifted apart.

**What I am NOT recommending:** working arc-008 toward a closure percentage. F2 says that number
is meaningless for this arc. The arc closes when the audit's *warning list* is empty-or-acknowledged
— which is item 2 — not when a ratio crosses 0.8.
