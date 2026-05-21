# Review Evidence — Batch 1 (T-1482..T-1488)

Gathered: 2026-05-22 (read-only; no task files edited, no AC boxes ticked, no source changed).
Binary: `termlink 0.11.1 (04a8c3aa)` at `/root/.cargo/bin/termlink`. Run from `/opt/termlink`.
Own identity FP: `d1993c2c3ec44c94`. Hub: running, 14 live sessions.

**Fleet-state caveat:** The chat-arc fleet is currently quiet — no *other* peers are
posting. `agent presence` (fleet-peer view) excludes the operator's own FP, so it reads
empty right now even though the local agent's own history is visible (39 posts via
`agent who`). This is an EVIDENCE-EMPTY state for the presence-family tasks, not a failure.
The recorded GO evidence in each task file (dated 2026-05-04) showed live peers; that data
has aged out of even the 24h window 18 days later.

---

## T-1482 — agent presence (fleet-wide peer summary)

**Command:**
```
termlink agent presence --window-secs 86400
```
**Verdict:** EVIDENCE-EMPTY

```
(no peers active in window=86400s)
```
JSON form (`--json`): `{"peers": [], "window_secs": 86400}` — clean empty envelope, parses.
Note: the task's *agent* verification asserts `len(peers) >= 1`; that would currently fail
because no other peer is active. The *human* AC only asks to observe table alignment/header —
which cannot be judged on an empty fleet. Verb runs cleanly, returns valid empty state.

---

## T-1483 — agent who --target <name> (error-message clarity)

**Commands:**
```
termlink agent who --target nonexistent-session-xyz
termlink agent who --target some-name --target-fp deadbeefdead
termlink agent who
```
**Verdict:** EVIDENCE-CLEAN

```
error: Session 'nonexistent-session-xyz' not found: session not found: nonexistent-session-xyz
Error: specify either --target or --target-fp, not both
Error: must specify either --target <name> or --target-fp <hex>
```
All three error paths name the offending input/flag combo clearly. Ready to glance-and-tick.

---

## T-1484 — agent presence --filter-project (empty-with-filter message)

**Command:**
```
termlink agent presence --filter-project nonexistent-xyz
```
**Verdict:** EVIDENCE-CLEAN

```
(no peers active in window=3600s matching project=nonexistent-xyz)
```
The empty-state message names the filter (`matching project=nonexistent-xyz`), so the
operator can distinguish "filter excluded everyone" from "fleet is silent". This is exactly
what the human AC asks to verify, and the message reads naturally. (Default window 3600s shown;
behavior identical with --window-secs.)

---

## T-1485 — agent contact --ack-required (timeout error wording)

**Command:**
```
termlink agent contact --target-fp deadbeefdeadbeef --message hi --ack-required --ack-timeout-secs 5
```
**Verdict:** EVIDENCE-CLEAN

```
Posted to dm:d1993c2c3ec44c94:deadbeefdeadbeef — offset=5, ts=1779405166995
error: no ack from peer fp=deadbeefdeadbeef within 5s on topic=dm:d1993c2c3ec44c94:deadbeefdeadbeef. The post landed (chat-arc is offset-durable) — peer just hasn't responded yet. Re-run without --ack-required for fire-and-forget, or increase --ack-timeout-secs.
```
Exit code 10 (RC=10), fired within ~5s. Error names peer_fp, topic, timeout, and gives two
concrete next steps (drop the flag, or raise the timeout). Operator-actionable. Clean.

---

## T-1486 — agent presence --watch (live dashboard steadiness)

**Commands:**
```
termlink agent presence --watch --watch-interval 3 --window-secs 86400   # timeout-killed (expected)
termlink agent presence --watch --json                                    # rejection path
```
**Verdict:** HUMAN-VISUAL-ONLY

Watch loop: RC=124 (timeout-killed, expected for a live streaming loop — means it ran without
crashing). Frame body is the idle-fleet empty-state because no other peers are active; the
ANSI clear-screen frames are emitted but steadiness/flicker is an inherently visual judgment
that text capture cannot prove.

`--watch --json` correctly rejected at parse time:
```
{"error":"--watch and --json are incompatible: --watch streams re-rendered text frames; --json is one-shot. Pick one.","ok":false}
```
Command starts cleanly without erroring; the visual "no flicker / columns aligned" criterion
needs a human at a live terminal (ideally when a peer is active so rows render).

---

## T-1487 — agent ping <target> (one-liner scannability)

**Commands:**
```
termlink agent ping --target-fp d1993c2c3ec44c94 --window-secs 86400
termlink agent ping --target-fp deadbeefdeadbeef --window-secs 60
```
**Verdict:** EVIDENCE-CLEAN

```
d1993c2c3ec44c94 (d1993c2c3ec44c94): online — last seen 55m ago (window=86400s)
deadbeefdeadbeef (deadbeefdeadbeef): offline — last seen never (window=60s)
```
Online RC=0, offline RC=1. Both fit one line; online/offline distinction is obvious at a
glance. Exactly the scannable one-liner the human AC asks to verify.

---

## T-1488 — agent who --thread <T-XXX> (thread-filter output)

**Commands:**
```
termlink agent who --target-fp d1993c2c3ec44c94 --window-secs 86400
termlink agent who --target-fp d1993c2c3ec44c94 --thread T-1487 --window-secs 86400
```
**Verdict:** EVIDENCE-CLEAN

Unfiltered:
```
peer_fp:           d1993c2c3ec44c94
last_seen:         3351s ago (ts_ms=1779401821906)
posts_in_window:   39 (window_secs=86400)
from_projects:
  010-termlink                       26
  002-Claude-Partner-Network         11
  050-email-archive                   1
  3021-Bilderkarte-tool-llm           1
```
Filtered (--thread T-1487):
```
# filter_thread=T-1487
peer_fp:           d1993c2c3ec44c94
last_seen:         3351s ago (ts_ms=1779401821906)
posts_in_window:   0 (window_secs=86400)
from_projects:     (none observed in window)
```
Filtered count (0) <= unfiltered count (39); from_projects narrows to none (no T-1487-tagged
posts exist — expected); last_seen stays populated (filter-independent design validated).
Exactly what the human AC asks to compare. Clean.
