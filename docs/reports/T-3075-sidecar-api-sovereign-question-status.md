# T-3075 — Sidecar API inception: Sovereign-question status (2026-09-24)

## Context

The full architectural analysis for this inception was already written
2026-09-22 as `docs/design/arc-011-sidecar-api-architecture.md` (§1-§9,
charter collision, steelman/strawman, directive scoring, IW-1..IW-4). This
report is the C-001 research artifact for TODAY's re-check session: does
anything discovered since 2026-09-22 change that analysis?

## Finding: SQ-1 answered two of the four open questions without reopening them

`.context/arcs/arc-011.yaml` SQ-1 was resolved by the operator on 2026-09-23,
the day after the design doc was written. Its question ("Does the injector
belong to AEF rather than TermLink?") is the same question the design doc
calls IW-2. Resolution: the injector stays in TermLink, reframed as "a
PRIMITIVE built over TermLink's own session control, not an orchestration
engine" — "it already lives here, is proven end to end (T-3069), and moving
it would put the PTY classifier and the hub read on opposite sides of a
project boundary."

Re-reading the design doc's §6 (bright line: local host state only, never
inter-host delivery) against that resolution: the operator's own framing
("primitive, not orchestration engine") is only charter-consistent if the
sidecar stays inside that bright line. So IW-1 (charter collision) inherits
IW-2's resolution rather than needing a separate ruling — the operator did
not have to say "and it's local-control-only" explicitly, because approving
"stays in TermLink as a primitive" is not compatible with any other reading.

IW-3 (what does hub-independence actually buy) already has the design doc's
own steelman answer (§3): addressability during an outage, for host-local
questions only. That is not sovereign, it is `analysis of what the confirmed
scope already gives you.

## Finding: IW-4 has no ruling anywhere

Read `.context/arcs/arc-011.yaml` end to end (383 lines, 7 sovereign
questions logged, SQ-1 through SQ-7) looking for anything touching portable
respawn / systemd-only tradeoffs. Nothing. IW-4 is genuinely untouched —
it is the one open item on this task, and it gates a real build decision
(the S1/S12 build task needs a respawn mechanism to pick). Recorded as
`disposition: deferred` on the task, not decided by this agent.

## Outcome

Task file updated: IW-1/IW-2/IW-3 dispositioned "answered" citing the
evidence above (transcription of an existing operator decision + existing
analysis, not a new decision); IW-4 left `deferred` and written up as the
live Sovereign question for the human. Recommendation section updated:
still GO-on-analysis, ready for `fw inception decide T-3075 go` once the
human disposes of IW-4 (answer it, or explicitly accept deferring it into
the build task). This agent did not and cannot invoke that verb — Tier-0,
confirmed by the enforcement hook refusing even `fw inception decide --help`.
