# agent-send transport-select seam (V6 slice S2)

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

| Value | Meaning | S2 behavior |
|-------|---------|-------------|
| `hub` (default) | Post via the local hub (or the peer's hub when `--to` resolved a remote peer). | Today's behavior, unchanged. Never probes (`reachable=skip`). No stderr plan line. |
| `direct` | Intent: send straight to the peer's OWN hub. | Probes the peer hub's reachability; records the plan. Live routing still via hub (S4 will branch). |
| `auto` | Prefer direct, fall back to hub. | Same as `direct` in S2 (probe + record). S4 wires the actual fallback. |

An invalid value exits `2` with a clear message. The default `hub` reproduces
today's send path exactly — no probe, no extra output.

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

**Live send** — when a non-default transport is requested, one line to **stderr**:

```
agent-send: transport-plan: transport=direct direct_addr=127.0.0.1:9100 \
            reachable=yes — S2 records intent only; live routing still via \
            hub 127.0.0.1:9100 (direct/fall-back is S4)
```

The default `hub` transport prints no such line — stdout and stderr are
byte-for-byte what they were before S2.

## Test

`bash scripts/test-agent-send-transport.sh` — 7 hub-independent checks: flag
validation (exit 2), hub/direct/auto dry-run `RESOLVED` lines from a canned
fleet fixture, probe reachable-vs-unreachable via loopback, default-preserved,
and the live-path stderr plan line (present for `direct`, absent for `hub`).
SKIPs cleanly with no hub. The existing `bash scripts/test-agent-send.sh` (A–G)
still passes — S2 adds no regression to the send/confirm paths.

## Next slices

- **S3** direct-path sidecar journaled-receipt (writes to the S1 journal).
- **S4** try-direct/fall-back orchestration (the actual routing branch this seam
  enables).
- **S5** journal-authoritative + firehose suppression.
