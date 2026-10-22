# External design review: which conformance obligations belong to a worker-contract backend?

You are an independent systems-design reviewer. Be critical: disagreeing with the recommendation is
useful. Answer in English, at most ~700 words, in markdown, using the sections listed at the end.
Answer from this text only; do not run tools.

## Context

**TermLink** is a Rust substrate. It runs one hub per Linux host, and AI coding agents (each in a
terminal session TermLink manages) use it to discover each other, exchange durable messages, claim work and
control terminal sessions. A small fleet is run by one human operator.

**The worker contract (wc/0.3)** is a shared, backend-neutral spec, owned by a sibling project (the agent
framework, "AEF"), for dispatching AI workers. A *dispatch* has *attempts*. Each attempt moves through states:
dispatched → running → done / lost / unknown. Writes are fenced: by a nonce, by a supervisor credential, by a
supervisor generation counter, and by compare-and-set on the expected prior state. Spec highlights:
- The **sweep** (the backend's own observer) marks an attempt `lost` only after its session is seen absent in two
  sweeps and a grace period has passed.
- **Answers:** a dispatcher can answer a worker's question with a deadline. Unacknowledged answers *bounce* back to
  the dispatcher; they are never retried.
- **T7d:** an operator resolves an `unknown` attempt from the hub host (not over the network). An *attested* done
  (vouched for by the operator, no evidence) is "not finished work": downstream consumers such as review verdicts and
  pass-rate must refuse it, just as they refuse a `paused` attempt.
- **Parent-session loss (spec §4.6):** if the dispatcher's session disappears, ownership moves to a named successor
  through `adopt`. Until it is adopted, questions and bounces "surface in the project operator queue".

AEF ships **25 conformance vectors**: scripted scenarios of ops plus expected end states, meant for any backend. The
TermLink hub backend runs them through an adapter that reports an op the backend lacks as *not-implemented*, never as
a pass. Today: **18 pass, 0 fail, 6 not-implemented, 1 n/a** (that one needs a real AI worker).

## The six not-implemented cases

1. **CC-08 Hub restart, op `wait`:** a consumer waits on an attempt with a bound and receives its state. The hub has
   only a read (`dispatch.get`), no blocking wait.
2. **CC-09 Partition, ops `partition` / `heal`:** the worker host loses, then regains, contact with the authority.
   The hub has no model of host contact. A lost worker shows up as an absent session registration, which the sweep
   already handles (two absent sweeps plus grace, or an "undecidable" observation).
3. **CC-10 Parent-session loss, ops `inspect operator_queue` and `adopt`:** a question asked while the dispatcher is
   gone must appear in the operator's queue, and a successor must be able to adopt the dispatch. The hub has durable
   operator notices, but none for "question with no live dispatcher", and nothing transfers dispatcher ownership.
4. **CC-13 Backend parity (lock-timeout variant), op `hold_lock`:** another holder keeps the write lock, and the write
   must fail closed after the backend's bound. Hub writes take SQLite `BEGIN IMMEDIATE` with a 30 s bound, and unit
   tests already hold that lock from a second connection. The adapter has no `hold_lock` op.
5. **CC-19 Paused is terminal, op `consume`:** an evaluation-layer consumer refuses a `paused` attempt as
   `not_finished_work`.
6. **CC-22 Attested done, op `consume`:** the same consumer refuses an attested done. The hub record already carries
   `attested`, `outcome` and a null evidence digest, which is everything a consumer needs to decide.

## Options

- **A (recommended).** Build three small hub pieces:
  - `dispatch.wait`, a bounded long-poll that returns on a state change or at the bound;
  - an operator notice for a question with no live dispatcher, plus `adopt`;
  - adapter plumbing that holds the real SQLite lock.

  Ask AEF to accept two mappings: partition = the existing absent-registration rule, and the `consume` refusals
  belong to AEF's evaluation layer (n/a for a backend). Optionally expose a read-only `finished_work` fact.
- **B.** Narrow hub: build nothing; ask AEF to mark all six n/a, or to accept the unit tests as evidence.
- **C.** Defer until AEF has reviewed.

## What we want from you

1. For each of the six cases: is it a backend obligation, or something else? Say which. Where would you put it, and
   why?
2. Is option A's partition mapping (absence of a session registration = partition) sound, or does it lose an
   important distinction, e.g. a partition where the worker keeps running and later completes?
3. Is putting the `consume` refusals in the evaluation layer right? Or should the backend refuse to *hand out* an
   attested/paused result as finished, so a careless consumer cannot miss it?
4. What could go wrong with `dispatch.wait` (long-poll) and `adopt` (ownership transfer)? Races, starvation,
   authorization: who may adopt?
5. Your recommendation (A, B, C or another), with the single most important reason.

Sections: **Verdict** · **Per-case** · **Risks** · **What I would change** · **Recommendation**.
