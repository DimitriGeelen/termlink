# agent-send transport select (V6 slices S2 seam → S4 orchestration)

arc-003 reliable-comms V6 (apex, T-2296) **slice S2**. Adds a `--transport`
flag + a bounded reachability probe to `scripts/agent-send.sh` (the existing
routing brain) and surfaces the chosen plan. Ships **script-first** (no Rust
rebuild), same as S1 (the journal).

## What S2 is — and is NOT

S2 is the **seam + plan + probe** only. It computes *which transport would be
used* and *whether the peer's own hub is reachable*, and it surfaces that plan
(dry-run `RESOLVED` line + a stderr line on live sends). It does **NOT** change
where the mail is actually posted — the live send still goes via the local hub
(or the peer's hub when `--to` resolved a remote peer), byte-for-byte as before.

The actual try-direct/fall-back **orchestration** — send straight to the peer's
hub, and on failure fall back to the local hub — is **slice S4**. The
direct-path confirm-source change (a journaled `stage=delivered` receipt) is
**S3**. S2 only lays the rail they ride on. See
`docs/plans/T-2296-v6-direct-transport-first-design.md`.

## `--transport auto|direct|hub`

**Default flipped to `auto` in S4 (T-2301).** The table below is the shipped
S4 behavior; the S2-era "records intent only" note above describes the
intermediate rail.

| Value | Meaning | Shipped behavior (S4) |
|-------|---------|-----------------------|
| `auto` (default) | Prefer direct; fall back to hub store-and-forward if the remote host is unreachable. | Probes the remote peer hub. Reachable → **direct**. Unreachable → **LOUD fallback** to the local hub (offline-queue-backed, federates to the peer). A local peer has no remote leg → local path. |
| `direct` | Post straight to the peer's OWN hub; confirm via mechanism A (the S3 `stage=delivered` receipt). | Probes; reachable → post to the peer hub. Unreachable remote host → **FAILS loud** (exit 3), never falls back. |
| `hub` | Escape hatch — today's pre-V6 path. | Post via the local hub (or the peer's hub when `--to` resolved a remote peer). Never probes (`reachable=skip`), never falls back, no plan line. Byte-for-byte unchanged. |

An invalid value exits `2` with a clear message. `--transport hub` reproduces the
pre-V6 send path exactly — no probe, no extra output, no fallback.

## The reachability probe

A bounded wrapper around `termlink remote ping <addr>` (`cmd_remote_ping`,
`remote.rs`). It runs under a short timeout so a wedged or unreachable peer hub
can never hang the send. It runs **only** for `direct`/`auto` against a resolved
*remote* peer hub — for `hub` transport, or a peer that is on our own hub
(`direct_addr=local`), there is nothing remote to probe and the plan prints
`reachable=skip`.

The target address is the peer's hub as reported by fleet presence
(`--to <agent-id>` resolution, T-2293 self-report). Hardening this to a
hub-attested source address is a later follow-up (T-2297, design §5 Q1); S2
ships on the self-reported address, which is correct for the flat /24.

Test seams:
- `REMOTE_PING_VERB` — override the ping command (space-split) so tests feed a
  canned pass/fail without a second host.
- `TERMLINK_PROBE_TIMEOUT` — override the per-probe timeout (default `5`s).
- Loopback: `127.0.0.1:9100` (a live local hub) probes `reachable=yes`; a closed
  port (`127.0.0.1:1`) probes `reachable=no` — both branches against a real hub.

## Where the plan surfaces

**Dry-run** (`--to <agent-id> --dry-run`) — the `RESOLVED:` line gains three
fields:

```
RESOLVED: agent_id=... status=LIVE ... hub=127.0.0.1:9100 routing=remote \
          transport=direct direct_addr=127.0.0.1:9100 reachable=yes
```

**Live send** — for any non-`hub` transport, one plan line to **stderr**:

```
agent-send: transport-plan: transport=direct direct_addr=127.0.0.1:9100 \
            reachable=yes → direct (confirm via mechanism A / S3 receipt)
```

`--transport hub` prints no such line — stdout and stderr are byte-for-byte
what they were before V6.

## S4 — the orchestration (shipped, T-2301)

S4 turns the seam into the actual routing decision. After the probe, `auto`
resolves to an **effective transport**:

- **direct** (remote peer reachable, or a local peer): post to the peer's own
  hub (`--hub <peer>`; a local peer posts to the local hub). The doorbell ring
  is best-effort — a running S3 sidecar posts the `stage=delivered` receipt with
  no woken agent, so an inject failure is non-fatal. Confirm = **mechanism A**
  (the receipt envelope the poll already surfaces, S3-stage-aware).
- **fallback** (`auto`, remote host unreachable): emit the LOUD line

  ```
  agent-send: FALLBACK host 192.168.10.141:9100 unreachable → hub store-and-forward
  ```

  then post to the **local** hub. This is deliberate: a TCP cross-hub post to the
  down peer would hard-fail — that path bypasses the offline queue
  (`channel.rs` T-1385). Only the local unix path is queue-backed, so posting
  locally is the genuine STORE half of store-and-forward; the DM topic federates
  to the peer's hub with sync lag, and the sidecar/federated receipt lands the
  confirm when the host recovers. No live listener to wake (host down) → the
  doorbell is skipped. If no receipt arrives in the window, the send FAILs loud
  but names that the turn is durably stored and the T-2295 unconfirmed-delivery
  canary tracks it.

`--transport direct` against an unreachable remote host **fails loud** (exit 3)
instead of falling back. `--transport hub` never probes and never falls back.

The single sender-API confirm contract (V3b: DELIVERED-or-FAILED loud) is
unchanged — only the receipt SOURCE differs by transport (A on direct; A via the
local hub + federation on fallback). One contract, two producers.

## Test

`bash scripts/test-agent-send-transport.sh` — 7 hub-independent checks (S2 seam:
flag validation, dry-run `RESOLVED` lines, probe reachable-vs-unreachable via
loopback, default-is-auto, live-path plan line present for non-`hub` / absent for
`hub`).

`bash scripts/test-agent-send-orchestration.sh` — 5 hub-independent checks (S4
branch: reachable peer → DIRECT + DELIVERED; unreachable host → LOUD fallback +
DELIVERED via the hub leg; `--transport direct` on a down host → loud FAIL, no
post; `--transport hub` never falls back; default is auto). Loopback
`127.0.0.1:9100` (up) / `127.0.0.1:1` (closed) + a canned fleet fixture drive
both branches with no second host.

Both SKIP cleanly with no hub. `bash scripts/test-agent-send.sh` (A–G) still
passes — the default flip to `auto` preserves the local send/confirm paths.

## Next slice

- **S5** journal-authoritative + firehose suppression for `dm:` (the last slice).
