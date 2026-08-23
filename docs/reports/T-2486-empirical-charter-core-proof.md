# T-2486 — Empirical charter-core proof (T-2468 firing 8)

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/active/T-2486-t-2468-firing-8--empirical-charter-core-.md`.
> The exploration it describes was carried out earlier; this file relocates that
> trail into `docs/reports/` per C-001, it does not add findings. Nothing here
> was inferred — every claim below appears in the task file.
>
> **Status at consolidation:** exploration complete (3 of 3 IW questions
> disposed, all at confidence 3). The task remains `owner: human` and undecided;
> no decision is recorded or implied by this document.

## The question

The T-2468 mandate — *"ultra-critically review TermLink's purpose/goals, incept +
build + test the gaps, drive to completion"* — was firing for the 8th time. The
prior seven firings had shipped 13 canaries and 4 affirmative charter-verb
provers.

That is the problem this inception was opened on. The review's own verdict was
**over-built breadth, incomplete core; subtract-and-deepen, don't add**
(`docs/reports/T-2468-*.md`), and by firing 8 that verdict applied to the review's
own pattern: every firing had added another meta-tool. Continuing to add would be
the anti-pattern the review had just named.

The move chosen instead was **empirical rather than armchair**. Four "prove it
works right now" provers existed, one per charter verb (discover / exchange
durable messages / claim work / control terminal sessions) — and *a prover that is
never run is shelf-ware*. So: run all four against the live substrate and take the
result as ground truth. Whatever broke would be the genuine gap to deepen. If
nothing broke, the honest finding would be that the core is proven and the
remaining gaps are all human-gated — itself a valid review outcome, and explicitly
**not** a mandate to invent work.

## Assumptions tested

- **A-1** — the four provers, run live, decisively pass or name a broken stage,
  with no ambiguous output.
- **A-2** — a prover FAIL, if any, points at an in-authority, bounded fix
  (deepen the core) rather than a human-gated item (P3b subsystem, P6
  multi-tenant, P4 deletion).

## Findings

### IW-1 — Do all four charter-verb provers pass against the live substrate right now?

*confidence 3 · answered*

**Yes — 3.5 of 4 proven live.** `session-selftest` PROVEN; `substrate-smoke`
10/10; comms DISCOVER and SEND both PASS. The only non-PASS was comms CONSUME,
and it was correctly classified `busy-or-manual` (G-083) — a peer-side diagnosis,
not a TermLink defect.

**The charter core holds.**

### IW-2 — If a prover fails, is the fix in-authority (bounded, reversible, no new subsystem, no user-facing removal)?

*confidence 3 · answered*

The provers found no core defect. But a parallel critical-review subagent surfaced
a genuine in-authority reliability defect that **no prover covered**: the
durable-log reader walled an entire topic on a single poison record.

That fix was bounded (2 files, 3 tests), reversible, and added no new surface. It
was built and shipped as **T-2487** (commit `5b0c0134`; 98/98 bus lib green, hub
compiles).

The gap between "all four provers pass" and "a real reliability defect exists" is
the durable lesson here: the provers assert the four verbs work, not that the
paths beneath them are resilient.

### IW-3 — If all provers pass, is there a genuine subtract-or-deepen gap left, or is the honest finding "core proven, remainder human-gated"?

*confidence 3 · answered*

**Both, honestly.** One genuine deepen existed and is now shipped — T-2487's
reader resilience, which deepens the antifragility of the charter's first noun
while adding zero breadth.

The residual is human-gated:

- the fsync-before-index DURABILITY half (T-2464 — a perf/ADR call)
- P3b make-it-live subsystem (T-2481)
- P6 multi-tenant
- P4 deprecated-tool deletion (soak + GO)

No 14th canary or prover was manufactured. The anti-pattern this firing named was
explicitly avoided.

## Outcome

The core is empirically proven; the one reliability defect found outside prover
coverage is shipped; everything remaining requires human decisions this inception
does not have the authority to make.
