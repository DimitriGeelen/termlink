# T-2373 — arc-004 cross-host push-wake verification

**Arc:** arc-004 push-transport · **Date:** 2026-07-06 · **Host:** workstation-107 (192.168.10.107) → ring20-management (192.168.10.122)

## Why this exists

arc-004's charter headline is: *"a live agent receives a DM the instant it is posted via a
hub→client WebSocket push stream (sub-second)."* The arc was marked `closed=shipped` and
repeatedly "live-verified" — **but every latency proof in the arc was same-host:**

| Prior evidence | Bind | Reality |
|---|---|---|
| `bench-pushwake-latency.sh` (T-2320) — 76ms median | `127.0.0.1:9196` | loopback, single host |
| `demo-ws-push.sh` (T-2310) — 91/99/93ms | `127.0.0.1:9199` | loopback |
| `demo-dm-rail-pushwake.sh` (T-2342) | `127.0.0.1:9199` | loopback (3 fp on one host) |
| T-2364 "0.107s" live re-verify | .107 → **its own** hub | same host |
| T-2313 "31ms" WS-over-Unix | Unix socket | explicitly **co-located** |

**The actual deployment path — a subscriber on host A woken by a post landing on a hub on
host B, over the real LAN with TLS-over-the-wire — was never measured until this task.** That
is a verification-completeness gap: same-host loopback was accepted as proof of an
inherently cross-host mechanic (G-019 class — the shipped status overstated the evidence).

## Method

`scripts/bench-pushwake-xhost.sh` (reusable, exit-coded). Subscriber runs on .107 and points
at the **remote** .122 hub profile over TCP+TLS; poster also targets the remote hub. Both legs
cross the wire:

```
.107  local `channel subscribe inbox.queued --push --hub ring20-management`  (WS/TLS over LAN)
   ▲                                                                              │
   │  inbox.queued push frame (over LAN)                     post inbox:<t> --hub ▼
   └──────────────────────────── .122 hub ◀── local `channel post --hub ring20-management`
```

- `t0` = just before the post RPC (issued on .107, lands on .122's hub).
- `t1` = when the pushed `inbox.queued` frame is observed in the local subscriber output.
- `latency = t1 − t0` = post-RPC-over-LAN + hub fan-out + push-frame-back-over-LAN.
- Detection polls at 20ms granularity → reported latency is an **upper bound**.

## Result (2026-07-06, `termlink 0.11.324`)

```
remote hub:  profile 'ring20-management' (192.168.10.122:9100)
trial 1: 199 ms   trial 2: 163 ms   trial 3: 181 ms   trial 4: 186 ms   trial 5: 167 ms
rang 5/5   min=163  median=181  max=199  ms
RESULT: PASS — cross-host median 181ms is sub-second.
```

An earlier ad-hoc run of the same path measured **153ms** median (126–199). Both are
consistently **sub-200ms cross-host** — ~80–100× faster than the ~15s pre-push
doorbell-then-poll floor the arc set out to replace. Raw frames confirmed real
`[push] inbox.queued` delivery with incrementing `message_offset` arriving on .107 the instant
each post landed on .122.

## Honest scope — what this number does and does not include

- **Includes:** both network legs (post RPC .107→.122, and the push frame .122→.107) plus the
  hub-side aggregator fan-out. This is the genuinely cross-host portion that all prior benches
  omitted.
- **Excludes:** the final PTY inject/echo on the receiving side. This run used a raw
  `channel subscribe --push` consumer, not a full `be-reachable` pushwaker, so the "ring the
  agent's terminal" step is not in the 181ms. That inject is a purely-local, no-network step
  already measured inside `bench-pushwake-latency.sh` (part of its 76ms loopback figure).
- **Therefore:** full cross-host agent wake ≈ 181ms (network) + tens of ms (local inject) —
  still comfortably sub-second.

## Two-number summary

| Path | Measured | Includes PTY inject? |
|---|---|---|
| Single-host loopback (T-2320) | **76 ms** median | yes |
| **Cross-host over LAN (T-2373)** | **181 ms** median (5/5) | no (network legs only) |

## Verdict

arc-004's headline mechanic holds **across a real host boundary**, not just on loopback:
sub-second push-wake confirmed at 181ms median over the LAN. The arc is sound as-shipped —
but its close was reached on **loopback-only** evidence, and this task supplies the first
genuine multi-host proof plus a reusable harness so future regressions (or a fleet upgrade
that breaks the WS path) can be caught cross-host, not just same-box.

## Reproduce

```bash
scripts/bench-pushwake-xhost.sh ring20-management          # or any reachable remote hub profile
# Env: TERMLINK_BIN, XHOST_HUB, XHOST_TRIALS. Exit 0 = sub-second PASS.
```
