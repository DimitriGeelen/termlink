# T-2995 — RPC-surface wiring-check sweep

**Status:** exploration in progress. Created BEFORE research (C-001).
**Concerns:** C-27 (`orchestrator.route`), C-28 (`dialog.presence`), C-31 (`session.*`),
C-32 (`event.*` family), C-33 (`event.broadcast`).
**Source:** `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md` rows C-27..C-33 (S-21).

## Why these five are one question, not five

Each row asks the same thing of a different RPC method: **is this surface wired, or is it
resident code nobody can reach?** One predicate, five instances — so this is one inception
under the sizing rule, not an umbrella. What differs per row is only the *disposition* a
clean answer implies (WIRE, DELETE, or leave alone), and one of those dispositions —
C-27's, against non-goal #4 — is explicitly reserved to the human.

## The prior that must be tested first

C-30 found that `kv.*` usage is **structurally invisible**: session-daemon calls never pass
the hub audit sink, so a zero-call reading there measures the instrument, not the traffic.
C-31 records that the same blind spot "plausibly applies (not individually verified)".

That makes IW-2 load-bearing for the whole sweep. If the blind spot extends to `session.*`
and `event.*`, then "zero calls" in those rows is not weak evidence of disuse — it is **no
evidence at all**, and any DELETE reasoning resting on it is unsound regardless of how the
wiring matrix comes out. IW-2 is therefore answered before any disposition is proposed.

## Open questions

- **IW-1** — per-surface wiring matrix: hub-implemented? CLI verb? MCP tool?
- **IW-2** — does the C-30 audit blind spot apply to `session.*` / `event.*`?
- **IW-3** — does the `event.broadcast` reference sweep (DELETE checks 4-5) come back clean?
- **IW-4** — does anything depend on `orchestrator.route` remaining present (the federation tripwire)?

## Findings

_(pending — filled as each spike completes)_

## Recommendation

_(pending — written after the findings, not before)_

## Decision

Not mine to make. `owner: agent` covers the measurement; the C-27 ruling against non-goal #4
is reserved to the human, and this task records no decision of any kind.
