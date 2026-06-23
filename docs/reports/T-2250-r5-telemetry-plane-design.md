# T-2250 — R5: local-first per-agent failure telemetry plane (inception)

**Arc:** arc-substrate-fitness (arc-002), Lock 2 (observable governance delivery),
after R4 (T-2243, shipped). **Workflow:** inception — design-first; the human
decides GO/NO-GO/DEFER via `fw task review T-2250`. **Agent recommendation:
DEFER** (design is decision-ready; open the build after R7 hygiene + a human call
on transport). Direction set by arc-002 plan §0.5 **Q2** (human-decided
2026-06-22).

## 1. Why (the gap R5 closes)

The discovery's F-INSTRUMENTATION finding: the substrate is blind to its own
failure modes. R4 (T-2243) closed the worst case — the silent poison-drop is now
a recoverable dead-letter. But the substrate still cannot answer, after the fact:

- How often did posts flap / retry / get queued before delivery?
- When did a spoke's circuit breaker trip, and for how long?
- What was RTT to the hub over time (degradation precedes failure)?
- Which agents died *without* a clean exit (the "can't-self-report" class — a
  crashed agent cannot post its own death)?

These are **time-series** signals. The key design insight from Q2: **telemetry
and presence are opposite data classes.** Presence is *current-state* → compact /
expire (R2's `LatestPerCvKey`). Telemetry is *history* → retain, but
**aggregated**. The F1 bug (T-1991) was applying forever-raw retention to
current-state data; the symmetric mistake here would be applying compact-to-latest
to time-series data and losing the history. Aggregation dissolves the
AS_FAILURE_OBSERVABILITY ↔ AS_RESOURCE_FOOTPRINT tension: keep the signal, drop
the volume.

## 2. Design constraints (the hard requirements)

1. **Local-first capture.** The recorder writes to a local durable store on each
   agent host. This is non-negotiable for the can't-self-report class: a record
   that lives only in the hub is lost exactly when the hub or the agent dies.
   Local capture survives crashes and has no observer effect on the hot path.
2. **Best-effort aggregated push, no new silent-drop.** The daily push must ride
   R4's durable offline queue (T-2243) so a failed push is dead-lettered, not
   lost. The telemetry plane must not reintroduce the very blindness it measures.
3. **Bounded collection topic (dogfood Q1).** The collector-side topic gets the
   same tiered/aggregated retention — never `forever`-raw. Raw Tier-0 stays local
   for forensics (24–48h); aggregates are kept long.

## 3. Proposed captured-signal schema (IW-2 — human ratifies on GO)

Per-agent local recorder, append-only, one row per event:

| Signal | Fields | Source |
|---|---|---|
| `post_discard` | ts, topic, reason, attempts | offline-queue poison path (now dead-letter, T-2243) |
| `queue_flap` | ts, topic, pending_depth, oldest_age_ms | queue-status transitions (T-2083) |
| `breaker_trip` | ts, peer, open_ms | spoke circuit breaker |
| `reconnect` | ts, peer, downtime_ms, attempt_n | reconnect path |
| `rtt_sample` | ts, peer, rtt_ms | periodic, sampled (not every call — footprint) |
| `clean_exit` | ts, reason | deregister / SIGTERM handler — its ABSENCE is the signal |

Daily local rollup → aggregated record per (agent, signal, day): counts,
percentiles (RTT p50/p95), max downtime. That aggregate is what gets pushed.

## 4. Storage + retention (tiered rollup)

- **Tier 0 (raw, local):** append-only file/SQLite under the agent's runtime dir,
  24–48h retention, never leaves the host. Forensic detail.
- **Tier 1 (daily aggregate, local→pushed):** the rollup; pushed once/day over
  the queue; kept long on the collector under bounded/aggregated retention.
- Dogfoods Q1: the collector topic uses `messages:N` or `days:N` — **not**
  `forever` (the F1 anti-pattern), and **not** `LatestPerCvKey` (that's for
  current-state, would destroy the series).

## 5. Transport — the plan's internal tension (IW-1)

The plan contradicts itself: R5's §4 one-liner says telemetry is
"**NOT-over-TermLink**," but Q2's resolution (§0.5) says "daily **AGGREGATED push
over TermLink** riding R4's queue."

**Recommendation: Q2's resolution wins** (it is the later, explicit human
decision). Reconciliation: *raw* telemetry is NOT-over-TermLink (Tier-0 stays
local — that's what the §4 phrase protects); the *daily aggregate* IS pushed over
TermLink (cheap, bounded, dead-lettered). The two statements describe different
tiers, not a true conflict. Surfaced for the human to confirm — if the human
intends raw to never touch the wire, that's already satisfied; if they intend
even aggregates to stay off TermLink, that reopens Q2.

## 6. Sequencing (IW-3)

**Recommend opening the R5 build AFTER R7** (live-host hygiene). R7 reaps the
981/1420 test/smoke topics and rotates the 1.36GB audit log; building telemetry
*before* that de-noising means the first baseline aggregates are measured against
a polluted surface. R5 design is ready now; R5 *build* benefits from a clean
measurement floor.

## 7. Recommendation: DEFER

The design is decision-ready and the direction is sound. DEFER (not NO-GO)
because: (a) the build should be sequenced after R7; (b) IW-1 transport wants a
one-line human confirmation; (c) on GO this decomposes into ≥3 sized build tasks
(local recorder; daily rollup+push over R4 queue; collector topic+retention) —
that decomposition is itself the GO action, not work to do under this inception
id. If the human judges R4 + the existing governor/queue/claims observability
sufficient for the next discovery, NO-GO is defensible and cheap.

## 8. Sovereign boundary — what this task did NOT do

- Did **not** decide GO/NO-GO/DEFER (agent advisory; `fw task review T-2250` is
  the human's gate).
- Did **not** implement anything — no recorder, no collector, no topic created,
  no source touched.
- Did **not** mint build tasks — decomposition happens on a human GO.

*Research is not authorization. Design surfaced; the human decides whether and
when to build.*
