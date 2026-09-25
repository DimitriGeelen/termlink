# T-3153 — Is the human review queue mis-owned, or genuinely pending?

**Task:** T-3153 · **Filed for:** T-2940 (D2, `owner: human`) · **Measured:** 2026-09-26

## The question, and who asked it

T-2940 records the audit's D2 finding — tasks waiting over 30 days in the human review
queue — and its own remediation text names the alternative explanation explicitly:

> **If not:** if the 57 are mostly mis-owned rather than genuinely pending, that is a
> separate structural finding about ownership assignment at task creation — file it
> rather than bulk-reassigning.

This report answers that, and files rather than acts. **No Human AC was ticked, no task
was re-owned, no task was closed.**

The standard being measured against is not an opinion. CLAUDE.md (T-1811/T-1878) already
states the rule:

> default to `[REVIEWER]` if Expected is grep-able — If your Expected clause is grep-able
> / file-exists / structural (a deterministic shell check), prefer `[REVIEWER]`; that AC
> should be an Agent AC with the reviewer command in `## Verification` instead of a Human
> AC here. Only keep `[REVIEW]` if verification genuinely needs human taste.

So "mis-classified" means: **this AC's own `Expected:` clause is one a command could
settle, and the project's documented rule says it should not have been a Human AC.**

## Three counts of the same backlog, and they disagree

| source | figure |
|---|---|
| audit D2 finding (T-2940 title) | **57** tasks |
| `fw review-queue` | **141** Human ACs awaiting verification (+8 pending inception decisions) |
| this scan of `.tasks/active/` | **165** unticked Human ACs across **142** tasks |

The three do not agree, and the disagreement is itself worth recording: they count
different units (tasks vs. ACs) over different windows (>30d vs. all). A backlog with
three published sizes is hard to act on, because closing "57" and closing "165" are
different jobs. **The 142-task figure is the one that matches `fw review-queue`'s 141
within rounding; the audit's 57 is an age-filtered subset, not a smaller measurement of
the same thing.**

Oldest item: **148 days** (T-1417).

## Method, and the instrument that failed

**First attempt — a regex classifier over `**Expected:**` clauses. It does not work, and
saying so is part of the result.** Two vocabularies (mechanical vs. judgement) were
written down *before* looking at outcomes and applied uniformly, with ties scored against
the "mis-classified" claim. It returned MECHANICAL 39 / JUDGEMENT 6 / **UNCLEAR 120**.

UNCLEAR at 73% is not a finding, it is an instrument failure. Inspecting the bucket showed
why: 105 of the 120 had a perfectly checkable Expected clause that simply used vocabulary
the regex lacked — comparison and quantity forms like `/var/log < 50 %`, `Same hash`,
`--version reports 0.9.1591 or newer`, `channel --help lists 53 subcommands`,
`0 attributable legacy calls`. The regex **under-counted mechanical ACs badly.**

**The obvious next move — widen the vocabulary and re-run — was deliberately not taken.**
Tuning a classifier after seeing which way it errs fits it to the answer the author
already suspects. Instead:

**Second attempt — a hand-labelled random sample as ground truth.** 25 of the 165 drawn
with a fixed seed (31537, reproducible), each labelled by reading its Expected clause
against the rule above. Mixed clauses (one mechanical half, one taste half — e.g. *"exit
codes are distinct (0/6/7)… output is readable"*) were scored **JUDGEMENT**, again against
the claim.

The regex was then scored against those labels:

```
REGEX vs HAND: agree 8 / 25   → error rate 68%
  MECHANICAL -> UNCLEAR  5     (missed)
  JUDGEMENT  -> UNCLEAR 10     (missed)
  JUDGEMENT  -> MECHANICAL 2   (wrong direction)
```

**A 68% error rate means the regex supports no claim at all.** It is reported here so that
nobody later mistakes its tidy-looking `39 / 6 / 120` for a measurement. The numbers below
come from the hand-labelled sample only.

## Result — the hypothesis is NOT supported

Of 25 sampled ACs, 2 carried no `Expected:` clause at all (indeterminate), leaving 23
labelable:

| label | count | share |
|---|---|---|
| **MECHANICAL** — a command could settle it | **10** | **43%** |
| **JUDGEMENT** — genuinely needs a human | **13** | **57%** |

Wilson 95% CI on the mechanical share: **26% – 63%**.

Scaled to the 165 unticked Human ACs: **~72 mechanically checkable** (range ~42–104), and
**~93 genuinely needing a human**.

**So the answer to T-2940 is: no, they are not "mostly mis-owned".** A substantial
minority is — somewhere between a quarter and two-thirds, best estimate a little over
four in ten — but the majority of this queue is real human work that no amount of
reclassification removes. Bulk-reassigning would have been wrong.

Note the prefix data does not rescue the optimistic reading either: 130 of 165 are already
marked `[REVIEW]` (the "needs taste" prefix) and only 34 `[RUBBER-STAMP]`. Even if every
single RUBBER-STAMP were mechanical, that is 21% of the queue, not a majority.

## What this means in practice

1. **There is a real, bounded conversion opportunity: ~72 ACs.** Each is an AC that should
   move to `### Agent` with its check written into `## Verification`, per T-1811/T-1878.
   That is worth doing and would materially shrink the queue — but it is ~72 individual
   judgements, not a batch operation, and each one changes what gates a task's completion.
2. **It will not clear D2.** ~93 items would remain. If the audit's D2 threshold expects a
   clean queue, it will keep firing after the conversion, and the expectation — not the
   queue — is what needs revisiting.
3. **The real cost is age, not size.** A 148-day-old `[RUBBER-STAMP]` whose Expected
   condition has *already been satisfied by later work* is the worst case: it is neither
   pending nor done, and nothing tells the operator it has become free. Two were found
   incidentally while selecting this work — see below.

## Incidental finding: stamp-ready items nothing surfaces

While selecting work, **T-2939** and **T-2938** (both arc-008, both Q1) were checked. Both
are `[RUBBER-STAMP]` Human ACs whose Expected condition is **already met**:

- T-2939 expects `/etc/cron.d/termlink-substrate-smoke-canary` to exist and the audit to
  stop printing `[FAIL] cron(substrate-smoke-canary)`. Measured: the file exists,
  **byte-identical** to `.context/cron/substrate-smoke-canary.crontab` (`cmp` clean), and
  `check-cron-install-drift.sh` reports *healthy — 30 installed + matching, 0 drift*, rc 0.
- T-2938 expects the deployed crontab to match the registry. Same check, same result.

Both are stamp-ready and neither is surfaced as such. The audit's own
`unclosed-satisfied` scan **cannot** find them: its criterion requires *zero unticked
Human AC checkboxes*, so a task waiting on exactly this kind of now-satisfied stamp is
excluded by construction. That is a genuine gap between two adjacent reports — one lists
tasks with no outstanding human work, the other lists tasks with outstanding human work,
and neither lists **tasks whose outstanding human work has quietly become trivial.**

## Limits — read before acting on this

- **n=23 is a small sample.** The CI is wide (26–63%) on purpose; do not quote 43% as
  precise. What the data supports is "a substantial minority, not a majority".
- **The hand labels are mine**, and I am the producer of this report. They are reproducible
  (seed 31537) and every labelled clause is printed in the task record, so they can be
  re-labelled by someone else and this conclusion falsified. That is the intended check —
  not that I judged carefully.
- **This measures the AC text, not the work.** An Expected clause that *reads* mechanical
  may still sit on a task whose real remaining work is a judgement, and vice versa.
- **Nothing here authorises the conversion.** Moving ~72 ACs from Human to Agent changes
  what gates completion on 72 tasks. That is a scope decision for the operator.

## Sovereign question

**Should the ~72 mechanically-checkable Human ACs be converted to Agent ACs + Verification
commands?** It is the documented convention (T-1811/T-1878), it would shrink the queue by
roughly 40%, and it moves 72 completion gates from human sign-off to automated check —
which is the point, and also the risk. Surfaced, not decided.
