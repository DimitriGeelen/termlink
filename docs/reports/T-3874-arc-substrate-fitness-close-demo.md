# T-3874 — arc-substrate-fitness (arc-002) close: demo evidence

Measured on .107 on 2026-11-27, after operator ruling A ("finish T-3466 on proof, then close").

## Headline

"An operator querying presence sees only LIVE agents (no multi-day-stale ghosts), and a worker's
'complete' is never silently lost — a discarded post surfaces in a recoverable dead-letter instead
of vanishing."

## Evidence

1. **Presence is bounded, not a forever log.** `termlink channel info agent-presence --json`:
   retention `messages(1000)`, 1024 records, no flags. The discovery (T-2241) found the same topic
   on `forever` with ~30,000 stale heartbeats. Retention change path: T-2244; per-key compaction:
   T-2245; regrowth guard: T-2252 (topic-growth canary).
2. **A discarded post is dead-lettered, not dropped.** T-2243 added the dead-letter table for the
   offline queue; `bash scripts/check-dead-letter-freshness.sh` → rc 0, "healthy (0 pending,
   0 dead-lettered)"; MCP parity for the same signal: T-2253.
3. **Failures are visible fleet-wide.** T-3465 logs queue and circuit-breaker events locally;
   T-3466's daily rollup posts aggregates over TermLink; `bash scripts/telemetry-fleet-view.sh`
   (fixed in T-3878) shows four hosts reporting every day 2026-11-20..26 (dimitrimintdev = .107,
   ring20-management, ring20-dashboard, greenfield), dead-letter column 0 throughout.

## Tasks

14 in the arc, all work-completed (T-3466 last, 2026-11-27). The Sovereign questions Q1–Q3 were
answered in `docs/plans/T-2242-substrate-fitness-ingestion.md` (Q1 two-step presence retention,
Q2 local-first telemetry with a daily aggregated push, Q3 one arc); the arc's description still
calls them unresolved and is out of date on that point.
