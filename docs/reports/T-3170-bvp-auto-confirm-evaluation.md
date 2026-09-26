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

## Findings

[FILLING — nothing verified yet]

## Recommendation

[FILLING — DEFER at filing; see Go/No-Go criteria in the task]
