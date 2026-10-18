# T-3440 — arc-002 (arc-substrate-fitness) close evidence

> **Status (2026-10-18): NOT a close.** The operator ruled the arc stays open while member work remains (T-3440 Decisions). This is a progress record of what is shipped.

Operator ruling 5b = A, 2026-10-18: close as shipped.

## Headline mechanic

"An operator querying presence sees only LIVE agents (no multi-day-stale ghosts), and a
worker's 'complete' is never silently lost — a discarded post surfaces in a recoverable
dead-letter instead of vanishing."

## Evidence (measured 2026-10-18 on .107)

1. **Presence is current-state, not a forever-log.** `termlink channel cv-keys agent-presence --json`
   returns `count: 2`, one entry per live agent (R2, T-2244..T-2248: latest-per-cv-key
   retention + compaction). Before the arc, presence was an unbounded log of every heartbeat
   (T-1991).
2. **Discarded posts surface instead of vanishing.** `termlink channel queue-status --json`
   reports a `dead_letters` count (0 today). A post rejected 10 times moves to the
   `dead_letters` table instead of being deleted (R4, T-2243; MCP parity T-2253). The
   dead-letter canary (T-2558) reads it daily; its log carries earlier firings, so the
   path has been exercised in production, not only in tests.
3. **Growth is watched.** The topic-growth canary (T-2252) and the bounded RPC audit log
   (T-2251) close the volume half.

## Members

11 of 12 tasks work-completed: T-2243, T-2244, T-2245, T-2246, T-2247, T-2248, T-2249,
T-2251, T-2252, T-2253, T-2254.

Open: **T-2250** (R5 telemetry plane design), an inception owned by the operator, DEFER with
`revisit_at: 2026-07-25`. It stays a standalone deferred task; closing the arc does not
decide it.
