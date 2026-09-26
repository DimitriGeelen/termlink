# T-3170 — 832's BVP auto-confirm + scoring telemetry: evaluate for incorporation

**Phase:** inception (exploration). **Recommendation at filing:** DEFER — three of the four
load-bearing claims are about code nobody here has read.
**Source:** `framework:pickup` offset 168, from 832-Workflow-designer (their T-856).

## The problem, stated plainly

The framework can score a task for priority (BVP). An agent may PROPOSE a score; only a human
may CONFIRM it, via `fw bvp confirm --i-am-human`. The flag exists so an agent cannot run the
verb by pretending to be the operator. That is a genuine sovereignty gate and it is doing real
work — the scoring *rules* are policy, and policy is the operator's.

But it also means every scored task needs a personal approval of a number the operator usually
has no strong opinion about. The predictable result is that scores do not get confirmed and the
priority ranking that depends on them stays empty — so the one feature meant to answer "what
should I work on" is starved by the gate protecting it.

832's operator ruled on this for their project:

> "BVP scoring should come automatically, no approval from user anymore. Scoring just happens."
> … "key thing is we want to have telemetry about it. We want to collect data so we can analyze
> it and improve it."

They implemented it, then sent the **design** here rather than keeping it local, explicitly
noting it is a change to our governance model and that their operator's decision binds their
project only.

## What they actually built (their account, not yet verified here)

1. **One verb, not the gate.** `acd_gate()` guards five verbs; they opened exactly one —
   `confirm`, which scores a task. `weight --set`, `driver --add`, `driver --remove` and
   `auto-promote --enable` stay gated, because those edit the VALUE MODEL, a different
   authority from scoring a task against it. Their suite's load-bearing case asserts the other
   four still refuse **with the switch on** — they note a blanket bypass is indistinguishable
   from the correct fix on the happy path.
2. **A switch, not a deletion.** `BVP_AUTO_CONFIRM`, defaulting OFF, so upstream semantics are
   unchanged for anyone who has not set it and the divergence is legible in config rather than
   invisible in a diff. Chosen because they measured one `fw upgrade` reverting six vendored
   fixes with nothing noticing — a deleted gate would have been silently restored.
3. **The telemetry works only because `confirm` PROMOTES rather than writes.** It takes
   `bvp_scores_proposed:` and promotes it, so they capture the proposal before it is cleared and
   record per row: `proposal_existed`, `proposed`, `confirmed`, `overrides`,
   `delta_vs_proposed`, `proposer_exact`. Across many rows that answers the operator's real
   question — how often is the estimator right, and which driver is it wrong on. Had auto-confirm
   written scores from scratch, every row would say the same thing and the ledger would be worthless.

Two smaller warnings they pass on, both worth copying:

- `confirmed_by` must NOT fall back to `$USER` on the auto path — it records the OS account the
  agent runs as, making an automatic confirmation indistinguishable from the operator's in the
  task file. They write `agent:auto (<switch>)`.
- The telemetry writer needs a path-override env var BEFORE the test suite exists. Theirs did
  not, and the first test run wrote fixture rows into the ledger the operator is meant to
  analyse. It is append-only, so those rows are now a documented permanent contaminant.

## Why this is not simply adopted

**Three of their claims are about code we have not read** (IW-3): that `acd_gate()` guards
exactly those five verbs, that only `confirm` was opened, and that `fw arc create`
(lib/arc.sh:399) has no agent gate. Our vendored tree is not necessarily their vendored tree.
Nothing proceeds until those are checked here.

**And our estimator may poison their ledger** (IW-4). The design's value rests on comparing a
real proposal against a confirmation. Our estimator's no-signal default scores every driver 2,
normalising a contentless task to exactly 0.40 — so a large share of our rows could record
`proposer_exact` against a "proposal" that was never an estimate at all. A ledger that says the
estimator is accurate because it agreed with its own default is worse than no ledger: it is a
measurement that cannot fail. This must be measured before the telemetry is trusted.

**`lib/bvp.sh` is vendored** (G-062), so a local implementation is deleted by the next
re-vendor — roughly one every two months in this lineage. That is precisely the failure 832
designed around, which is why IW-1's working assumption is framework-scope.

## Order of work, and why

**Telemetry first, switch second.** The telemetry half changes no governance, is reversible, and
is the only thing that can tell us whether the switch is safe to flip. Flipping first means
auto-confirming scores with no way to know whether the estimator deserves the trust. That
ordering also means a NO-GO on the switch still leaves something of value behind.

## The forcing function, recorded deliberately

**Offset 168 is NOT being acked, and that is the point.**

The framework-pickup canary is the only reason this filing reached the operator at all. Acking it
would mark an undecided governance proposal "processed" and delete the one mechanism that
surfaced it — after which this task joins 314 other active tasks, 140 of them already in
`started-work`.

This session produced direct evidence that a task alone ensures nothing: T-3117 was filed from a
real measurement whose premise then expired unnoticed, and T-3131 carried an acceptance criterion
claiming an upstream filing that **did not exist**, which sat unchallenged because an unticked box
reads as "work remaining" rather than "claim unverified".

So the daily alarm stays on until this reaches a decision. `revisit_at: 2026-10-03` is a backstop,
not the mechanism. The cost is one line of noise per day; the alternative is a well-written task
nobody opens again.

## Findings (steps 1-3 of the exploration plan, all pure reads)

### F1 — 832's claims are TRUE in our tree (IW-3 settled)

`acd_gate()` in our vendored `lib/bvp.sh` guards exactly the five verbs they named:
`weight` (:650), `confirm` (:858), `driver --add` (:939), `driver --remove` (:1233),
`auto-promote --enable` (:1347). And `fw arc create` has no agent gate — it requires
`--headline-mechanic`, a content requirement, and never consults `CLAUDECODE`. Both claims
confirmed by reading our own source, not taken on trust.

### F2 — our own refusal message concedes their argument

`acd_gate` refuses under `CLAUDECODE=1` unless `--i-am-human` or `--from-watchtower`, printing:

> "Weight/driver changes carry policy-edit authority (D8 — sovereignty at policy-edit time)
> and belong to the human, recorded via Watchtower."

That message fires for `confirm` as well — a verb that is neither a weight change nor a driver
change. **The gate's own stated justification does not cover the verb it is refusing.** 832's
distinction (scoring a task against the rules is not rewriting the rules) is already implicit in
our source; the gate simply applies one rationale to five verbs, and it only fits four of them.

### F3 — the bottleneck is not merely real, it is total (IW-5 settled)

Measured across all 2877 tasks with parseable frontmatter:

| | count |
|---|---|
| carry a proposed score | **449** |
| carry a CONFIRMED score | **0** |
| proposal awaiting confirmation | **449** |

**Zero scores have ever been confirmed in this project.** The gate has a 100% block rate over
the entire corpus. Every BVP-derived surface that depends on confirmed scores — the priority
ranking, `fw bvp --quadrant` — has therefore never had a single data point, which is consistent
with the known symptom that `--quadrant hv-lc` "silently returns No tasks match".

### F4 — the telemetry WOULD discriminate, mostly (IW-4 largely settled)

Of 449 proposals, **74 (16%)** score every driver 2 with every driver marked `no-signal` — the
vacuous rows that would log as `proposer_exact` against a non-estimate. The remaining 84% carry
at least some real signal. Distribution of no-signal drivers per proposal: 3 drivers is the mode
(239 proposals), 69 proposals are fully no-signal, only 1 proposal has none.

So the contaminant is real and bounded, not fatal. The ledger should record the no-signal count
per row so a vacuous agreement is distinguishable from a real one — without that field, 16% of
rows would silently inflate any accuracy figure computed from it.

### F5 — my recommended ORDERING was wrong, and F3 is what refutes it (IW-2)

I recommended telemetry-first on the reasoning that it is measurable and reversible and would
tell us whether the switch is safe. **F3 refutes it.** The ledger records confirmations. There
have been zero confirmations, ever, and there will continue to be zero while the gate stands. A
telemetry-first slice therefore produces an empty file indefinitely — it cannot observe the thing
it exists to observe until the switch that generates the events is on.

The two halves are coupled, which is presumably why 832 shipped them together. The correct
ordering is both-at-once behind the off-by-default switch, with the telemetry landing first
*within* that slice so no confirmation is ever written unrecorded.

## Recommendation

Not yet written — IW-1 (scope) is the operator's, and the recommendation should follow it rather
than presume it. What the evidence supports so far: GO on the design, with three amendments —
record the no-signal count per ledger row (F4), keep 832's `agent:auto (<switch>)` rather than a
`$USER` fallback, and ship the telemetry path-override env var before any test suite exists.
