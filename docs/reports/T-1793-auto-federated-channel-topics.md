# T-1793 — Auto-federated channel topics across hubs: does the fleet want it?

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/completed/T-1793-auto-federated-channel-topics-across-hub.md`.
> The reasoning below was recorded on 2026-05-25; this file relocates it out of
> the archived task file into `docs/reports/` per C-001.
>
> **This task's record is thinner than its siblings, and the artifact does not
> pad it.** The task file's `## Problem Statement` and `## Assumptions` sections
> are unfilled templates. Everything below comes from the one section that was
> written — the recommendation — plus the recorded decision. See the note on the
> discrepancy between them at the end; it is reproduced, not resolved.

## Context

T-1791 established that **TermLink has no inter-hub channel-topic federation
primitive**: cross-hub coordination is client-driven. A topic named
`agent-chat-arc` on hub A and one on hub B are unrelated state, and visibility
across them requires explicit `channel post --hub <addr>` or `termlink_remote_call`.

T-1793 was filed as follow-up #3 to ask a narrower question: should auto-federation
be *added* as a feature?

## The trade recorded

**Benefits**

- cleaner agent UX
- a single source of truth across the fleet
- no need to remember `--hub` or `remote_call` for shared topics

**Costs**

- state-sync complexity
- consistency-model choice — last-write-wins? vector clocks? CRDTs?
- conflict resolution
- bandwidth amplification on every post
- ordering guarantees across hubs
- retention-divergence handling

## Outcome — parked

Parked at `horizon: later`, for three stated reasons:

1. T-1166 retirement is **not blocked** by the absence of federation.
2. The current client-driven pattern **works correctly when used**.
3. The architectural cost is significant.

### Revisit when

Either of:

- multiple agents are **independently** surprised by per-hub semantics despite the
  documentation (i.e. G-060 stays alive), **or**
- a concrete fleet-wide coordination workflow emerges that the client-driven
  pattern cannot serve cleanly.

Both triggers are about observed behaviour rather than opinion, which is what
makes the parking honest rather than indefinite.

## A discrepancy in the record, reproduced as found

The task file's `## Recommendation` reads **DEFER**. The `## Decision` block
recorded on 2026-05-25T17:27:58Z reads **GO** — but its rationale is the DEFER
rationale verbatim, ending *"Parked at horizon=later because..."*.

So the recorded verdict and the recorded reasoning disagree. Every substantive
statement in the task file describes a deferral; only the one-word decision field
says otherwise, and the task was then archived at `horizon: later`, which is
consistent with DEFER and not with GO.

This artifact does not resolve that. Reading the evidence, the deferral is what
actually happened, but the decision field is a human-sovereignty record and
correcting it is not an agent's call. It is flagged here so that a future reader
of `docs/reports/` sees the inconsistency rather than inheriting whichever half
they happen to read first.
