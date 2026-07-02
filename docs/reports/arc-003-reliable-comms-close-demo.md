# Arc-003 `reliable-comms` — Close Demo Evidence

**Arc:** arc-003 (`reliable-comms`) — "Reliable cross-agent communication"
**Anchor:** T-2291 (inception)
**Date:** 2026-07-02
**Captured by:** driving arc-003 to completion (agent), operator-directed close.

## Headline mechanic (from the registry)

> An agent sends a message to a peer by identity and receives a confirmed
> delivery receipt — delivered direct host-to-host on the LAN with hub
> fallback — instead of seeing it silently lost across non-federating hubs.

This document is the wire-level evidence that the headline mechanic fires. It
captures the four V6-slice end-to-end suites that exercise the full
send-by-identity → try-direct/fall-back → journaled-confirmed-receipt path,
each run green on 2026-07-02.

## Constituent tasks (all `work-completed`)

| Task | Slice | Deliverable |
|------|-------|-------------|
| T-2292 | V1 | Per-agent identity by default (RC1 — send *by identity*) |
| T-2293 | V2 | Fleet discovery registry (RC2 — resolve peer host) |
| T-2297 | V2b | Hub-stamped observed source address (attested host, hardening) |
| T-2294 | V3a | Deterministic notify (sidecar wake) |
| T-2295 | V3b | Delivery-confirm by default + unconfirmed-delivery canary |
| T-2296 | V6 (apex) | Direct transport-first + hub fallback + per-conversation journaling |
| T-2298 | V6-S1 | Per-conversation journal read-side mirror |
| T-2299 | V6-S2 | Transport-select seam + reachability probe |
| T-2300 | V6-S3 | Sidecar journaled-receipt + stage-aware confirm |
| T-2301 | V6-S4 | Try-direct / fall-back orchestration |
| T-2302 | V6-S5 | Journal-authoritative + firehose suppression for dm |

## Wire-level demo — the mechanic firing

Each suite below drives real `termlink` binaries against a live loopback hub.
All four green on 2026-07-02 (exit 0).

### 1. Transport-select seam + reachability probe — `scripts/test-agent-send-transport.sh` (7/7)

```
PASS: T1: bad value rejected (exit=2)
PASS: T2: hub transport never probes
PASS: T3: direct + live loopback hub probes reachable=yes
PASS: T4: auto + closed port probes reachable=no
PASS: T5: S4 default is auto; local degenerate probes nothing
PASS: T6: live direct records plan to stderr, POSTED unchanged on stdout
PASS: T7: --transport hub emits no plan line (unchanged)
Results: 7 pass / 0 fail / 0 skip
```

*Mechanic part demonstrated:* **send by identity, transport chosen by
reachability** — a `direct` send probes the peer and posts host-to-host (T3, T6);
`auto` degrades cleanly when the peer is unreachable (T4, T5).

### 2. Try-direct / fall-back orchestration — `scripts/test-agent-send-orchestration.sh` (5/5)

```
PASS: O1: direct path delivered (rc=0, no fallback), topic=dm:ab…2865133:d1993c2c3ec44c94
PASS: O2: loud fallback + delivered via hub leg (rc=0), topic=dm:ab…2865133:d1993c2c3ec44c94
PASS: O3: direct+down fails loud (rc=3), never posted, never fell back
PASS: O4: hub escape hatch never falls back, no plan line (rc=2, unchanged)
PASS: O5: default is auto — unreachable host fell back loud (rc=3, deferred)
Results: 5 pass / 0 fail
```

*Mechanic part demonstrated:* **direct host-to-host with hub fallback** — O1 is
the direct-delivered happy path; O2 is direct-fails-then-hub-leg-delivers; O5 is
the default `auto` falling back loudly (never silently) when the peer is down.
No path silently loses the message.

### 3. Sidecar journaled-receipt + stage-aware confirm — `scripts/test-sidecar-auto-confirm.sh` (5/5)

```
PASS: T1 default-off is a no-op (receipts=0 journal=0)
PASS: T2 journal populated (rows=3)
PASS: T3 one stage=delivered receipt, up_to=2 (content watermark)
PASS: T4 offset guard held (receipts still 1)
PASS: T5 stage surfaced in DELIVERED line (rc=0)
Results: 5 pass / 0 fail
```

*Mechanic part demonstrated:* **confirmed delivery receipt** — a received turn
produces a durable `stage=delivered` receipt carrying a content watermark
(`up_to=2`), the observable "it arrived" signal that replaces silent loss.

### 4. Journal-authoritative + firehose suppression — `scripts/test-journal-reaper.sh` (5/5)

```
PASS: R1: firehose trimmed to newest 2 (offsets: 3 4 ), journal has all 5, presence untouched
PASS: R2: guard refused (SKIP-UNSAFE), all 6 offsets still on firehose (offset 0 survived)
PASS: R3: non-dm topic refused loud (rc=2)
PASS: R4: recent-dm unchanged after reap (5 turns), firehose-only control sees only 2 (trim confirmed)
PASS: R5: idempotent no-op (SKIP-WINDOW), firehose unchanged
Results: 5 pass / 0 fail
```

*Mechanic part demonstrated:* **no silent loss under retention** — the journal
is authoritative (⊇ firehose); the reaper refuses to trim any turn not first
mirrored to the journal (R2 SKIP-UNSAFE), so history survives firehose trimming
(R1, R4).

## Reproduce

```
cd /opt/termlink
for s in test-agent-send-transport test-agent-send-orchestration \
         test-sidecar-auto-confirm test-journal-reaper; do
  bash "scripts/$s.sh"; echo "[$s exit=$?]"
done
```

## Residual (operator-gated, non-blocking)

- **T-2297 Human [RUBBER-STAMP]** — live end-to-end proof of hub-attested
  `observed_addr` needs a hub rebuild+restart. A hardening confirmation; V6
  already ships on the T-2293 self-reported address, so this does not gate the
  headline mechanic.
- **Reaper cron activation (T-2296 AC3)** — S5 shipped the reaping *mechanism*;
  running it periodically is operator-side (arc-002 R2 sweep precedent), so
  durables stay off the firehose only while the reaper runs.
