# Agent-conversation arc — end-to-end test suite

> **Quick check:** `BIN=./target/release/termlink ./tests/e2e/arc-suite.sh`
> Six cross-hub e2e scripts run in ~10s. Look for `ARC SUITE GREEN`.

The agent-conversation arc (T-1325 → T-1391) layers Matrix-style primitives
(threads, reactions, edits, redactions, receipts, pins, stars, forwards,
DMs) onto termlink topics. After the [arc inception](../reports/T-1384-multi-agent-readiness-inception.md)
DEFER → GO reversal, T-1390..T-1395 added cross-hub e2e coverage to prove
the arc works on the multi-machine fleet, not just locally.

This doc is the operator-facing runbook for that suite.

## Fleet pre-requirements

The suite assumes:

| Requirement | How the suite checks |
|---|---|
| Local termlink ≥ `0.9.1542` (channel.\* RPC watermark) | `--version` parsed in pre-flight; aborts on lower |
| Hub at `127.0.0.1:9100` (`.107` self) reachable | `channel list --hub 127.0.0.1:9100` in pre-flight |
| Hub at `192.168.10.122:9100` (ring20-management) reachable | `channel list --hub 192.168.10.122:9100` in pre-flight |
| At least one live session on `.122` for cross-hub `remote exec` | `remote list ring20-management` parsed at startup |

If the third hub (`ring20-dashboard` / `.121`) is auth-broken, the suite still
runs — it doesn't depend on `.121`. To bring `.121` back, see the [hub auth
rotation protocol](../../CLAUDE.md#hub-auth-rotation-protocol).

## What each script proves

All scripts live in `tests/e2e/`.

### 1. `live-agents-conversation.sh` (T-1387)

Five live sessions on `.107` post in parallel as separate `--sender-id`
stand-ins. One additional post comes from `.122` via cross-hub TCP. All six
land on a single `.107` topic; canonical state shows 6 distinct senders.

Marker: `LIVE-AGENT E2E PASSED`

### 2. `cross-hub-bidirectional-6agents.sh` (T-1390)

Two topics: one originated on `.107` and one on `.122`. 6 senders each.
Cross-hub TCP posts go in BOTH directions. Cross-hub READ test confirms
canonical state converges byte-identically regardless of which hub
initiates the read. Hub independence verified — Topic A on `.107` does
NOT leak to `.122` (no accidental replication).

Marker: `BIDIRECTIONAL CROSS-HUB E2E PASSED`

Resolves T-1384 A3 (cross-hub canonical-state convergence — was BLOCKED).

### 3. `cross-hub-matrix-flow.sh` (T-1391)

Six-agent conversation thread exercising replies, reactions, edits, and
redactions. `frank-122` reacts and replies via cross-hub TCP — proves
`metadata.in_reply_to` resolves cross-hub. `channel thread <topic> 0`
read from `.122` matches the same read from `.107`.

Marker: `MATRIX-FLOW E2E PASSED`

### 4. `cross-hub-presence-flow.sh` (T-1392)

Covers the remaining Matrix surface: ack/receipts (cross-hub ack from
`.122`), typing emit + list, pin/pinned, star/starred, forward (cross-topic
copy with provenance), ancestors (upward chain walk), quote (parent-quoted
render), describe. Cross-hub READ convergence verified for pinned and
receipts.

Marker: `PRESENCE-FLOW E2E PASSED`

### 5. `cross-hub-dm-flow.sh` (T-1394)

Matrix-style direct message between two agents on different hubs. Alice on
`.107` sends via `channel dm --send` (canonical `dm:<a>:<b>` topic
auto-derived). Bob on `.122` replies via cross-hub TCP on the SAME topic
(peer-fingerprint pattern: one side computes the topic, both post to it).
Threaded reply linkage verified via `channel quote`.

Marker: `DM-FLOW E2E PASSED`

### 6. `cross-hub-stress-soak.sh` (T-1395)

Two-phase soak (a third phase re-runs the suite when invoked standalone):

- **Fan-in:** 50 parallel posts to one topic — 40 local + 10 cross-hub from
  `.122`. Verifies count == 50 (zero loss), offsets 0..49 contiguous (no
  gaps/dupes), 10 cross-hub posts attributed correctly.
- **Fan-out:** 5 topics × 10 senders per topic in parallel. Each topic
  carries exactly 10 envelopes.
- **Re-suite (standalone only):** runs `arc-suite.sh` again to prove no
  leftover state damage. Skipped when invoked from inside the suite via
  `ARC_SUITE_RUN=1`.

Marker: `STRESS-SOAK E2E PASSED`

## The `arc-suite.sh` runner (T-1393)

`tests/e2e/arc-suite.sh` runs all six scripts in order with a fleet
pre-flight gate. Single command answers: *is the agent-conversation arc
still green?*

```sh
BIN=./target/release/termlink ./tests/e2e/arc-suite.sh
```

Output: per-script PASS/FAIL with timing, then a summary table and
`ARC SUITE GREEN` marker. Exits non-zero on any failure.

Override hubs / version watermark via env:

```sh
HUB_107=127.0.0.1:9100 HUB_122=192.168.10.122:9100 \
MIN_VERSION=0.9.1542 \
BIN=./target/release/termlink ./tests/e2e/arc-suite.sh
```

## Troubleshooting

### `FAIL: session tl-XXXXX not in 'termlink list'`

Hardcoded session IDs that drifted. Fixed in T-1393 — `live-agents-conversation.sh`
now resolves the local session list dynamically. If you see this in another
script, replace the literal with:

```sh
mapfile -t SESSIONS < <("$BIN" list 2>/dev/null | awk '/^tl-/ {print $1}' | head -N)
```

### `FAIL: hub .122 ($HUB_122) unreachable`

`fleet doctor` will report the cause. Most common: secret rotation after a
hub restart. Heal with:

```sh
termlink fleet reauth ring20-management --bootstrap-from auto
```

(See [hub auth rotation protocol](../../CLAUDE.md#hub-auth-rotation-protocol).)

### `FAIL: local binary X.Y.Z < 0.9.1542`

Rebuild: `cargo build --release -p termlink-cli`.

### `FAIL: no live session on ring20-management`

The `.122` host needs at least one termlink session running for the
cross-hub `remote exec` paths. Start one with `termlink spawn` on `.122`,
or wait for the watchdog to bring one back.

### Script exits 0 but PASS marker missing

Some output post-processor stripped the marker line. Inspect the saved
log under `/tmp/<script>.out` (kept by `arc-suite.sh` after each run).

## Identity caveat (per-user, not per-session)

`channel post --sender-id <X>` overrides the envelope's `sender_id`, so
the bus sees N distinct senders even when a single shell drives all the
posts. Real per-session identity (one identity per termlink session, not
per `~/.termlink/identity/` file) was deferred — see T-1384's analysis.
The e2e suite uses `--sender-id` overrides as stand-ins; the topology
exercised is real (cross-hub TCP, both directions, concurrent), even
though the identity model isn't.

`channel edit` and `channel redact` have NO `--sender-id` flag, so those
primitives are always attributed to the local identity. T-1391's e2e
documents this as a CLI surface limitation, not an arc bug.

## Related work

- T-1384 — multi-agent readiness inception (DEFER → GO)
- T-1386 — initial 6-agent multi-machine e2e (synthetic identities)
- T-1387 — first live-agent e2e
- T-1390..T-1395 — this suite
