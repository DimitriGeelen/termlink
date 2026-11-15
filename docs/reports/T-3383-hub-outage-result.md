# T-3383 — Hub outage durations: result after ~34 days (2026-11-16)

Collector: `scripts/collect-hub-outages.sh`, every minute since 2026-10-11 (runme action 1). Summariser: `bash scripts/summarise-hub-outages.sh` (and `--json`), run 2026-11-16. Samples: 49,517 per hub, about 34 days at 1/min; 0 malformed lines. Intervals are sample-resolution estimates (± 1 min).

## Result

| Hub | Samples | Outages | p95 | Max |
|---|---|---|---|---|
| workstation-107-public (.107) | 49,517 | 191 | 121 s | 6,240 s (1 h 44 min) |
| ring20-dashboard (.121) | 49,516 | 57 | 540 s | 6,240 s |
| ring20-management (.122) | 49,517 | 47 | 540 s | 6,240 s |
| node1-pool (.122:19100) | 26,532 | 17 | 2,580 s | 2,580 s |
| greenfield (.132) | 49,517 | 63 | 129,000 s | 298,260 s (hub down ~11 days to 2026-11-15) |
| laptop-141 (.141) | 49,517 | 13 | 857,940 s | 857,940 s (laptop switched off) |
| **All** | | **575** | **2,281 s** | 857,940 s |

**H-1 (a 1 h window covers p95 outage): covered.** That holds fleet-wide (p95 38 min) and on every production hub (p95 ≤ 9 min).

## What the 1 h starting values do not cover

- All three production hubs share the same longest outage, 6,240 s (1 h 44 min). That points to one common event (network or host maintenance), not three independent ones. A 1 h credential lifetime / stale-list window would have expired during it. With renewal from 30 min, a credential issued just before such an event lapses within it, and the session has to re-authenticate after the hub returns.
- greenfield and laptop-141 are long-absence hosts (days), not short outages. No credential window should try to cover them; they re-join.

## For the two T-3351 rulings

1. OQ-3 = D (credential 1 h, renewal from 30 min, 24 h absolute, stale-list 1 h): the starting values are confirmed at p95. The one measured exception is a 1 h 44 min shared outage. That is an input for the operator if the starting values are revisited; it is not a ruling.
2. OQ-4 = D (new conversations always via the hub; revisit option B once per-agent OS accounts exist): this measurement is the hub-down input that revisit needs. The revisit itself is gated on per-agent OS accounts (OQ-2 target), and goes to the operator as its own decision then.
