# T-2717 — Inception decisions that contradict their own recommendation

Found under T-2716 (2026-06). Report written 2026-11-16 (T-3800 backlog triage), against the vendored AEF be.52.

## The two instances

| Task | Date decided | `## Recommendation` | `## Decision` | Contradicting rationale (excerpt) |
|---|---|---|---|---|
| T-1793 | 2026-05-25 | DEFER | GO | the DEFER text, ending "Parked at horizon=later because…" |
| T-2288 | 2026-06-26 | NO-GO | GO | the NO-GO text, including "a termlink-driven build now would violate the gate" |

## The four consistent cases (the precondition)

| Task | Date decided | Recommendation | Decision |
|---|---|---|---|
| T-1692 | 2026-05-18 | GO | GO |
| T-1693 | 2026-05-18 | GO | GO |
| T-1830 | 2026-05-28 | GO | GO |
| T-2054 | 2026-06-09 | GO | GO |

All four consistent cases had a GO recommendation. Both contradictions occurred exactly where the recommendation was not GO, five weeks apart. That makes it a mechanism (re-using the recommendation body as the rationale while typing `go`), not a one-off slip. Re-measured 2026-11-16 from each task file's `**Recommendation:**` and `**Decision**:` lines.

## Not tool auto-fill

`fw inception decide` requires `--rationale` (`.agentic-framework/lib/inception.sh`, "Rationale required: --rationale 'explanation'", line 512 in be.52; lines 421-423 when found). So in both cases the recommendation body was passed explicitly.

## What cannot be known

Whether each was a deliberate operator override (legitimate: sovereignty includes overriding a recommendation) or a mis-typed verdict is unrecoverable from the artifacts. This task altered neither decision. The defect was the undetectability, not the verdicts.

## Remedy: already upstream (AEF T-4727)

AEF fixed the undetectability independently. In be.52 (`lib/inception.sh` ~744-772), `decide` parses the Recommendation's verdict with the same extractor Watchtower's card uses. When the recorded verb differs, it prints `WARNING: recording <X> but the Recommendation is <Y> (T-4727)` and writes `**Diverges from Recommendation**: <Y>` into the `## Decision` block. It warns but never blocks, and Watchtower's decide runs the same CLI. An override is now auditable at a glance, which is what this task set out to request. No separate filing was made; it would duplicate T-4727.

The proposal drafted here (require explicit confirmation or `--override` on divergence) is stricter than T-4727's warn-and-record. Since sovereignty includes overriding, warn-and-record is the right weight, so it is not pursued.
