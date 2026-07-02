# arc-004 push-transport — demo evidence (T-2310)

**Arc:** arc-004 `push-transport` (anchor T-2303)
**Headline mechanic:** a live agent receives a DM the instant it is posted via a
hub→client WebSocket push (sub-second), replacing the 15 s doorbell-then-poll /
1 s poll wake-read floor, and cleanly degrades back to polling if the socket
drops — the durable `dm:` topics, receipts, journal and offline queue stay
authoritative underneath.

**Reproducer:** [`scripts/demo-ws-push.sh`](../../scripts/demo-ws-push.sh)
**Captured:** 2026-07-02
**Binary:** `target/release/termlink` (release build carrying S1–S4 + S3b)

---

## What the demo proves

1. **Sub-second push.** A `channel subscribe inbox.queued --push` consumer
   receives the `inbox.queued` doorbell frame — carrying the DM's **durable
   `message_offset`** — the instant the DM is posted, over a hub→client
   WebSocket. Measured post→push latency: **91–99 ms** across three runs (see
   below), against a **1 s** poll floor / **15 s** doorbell floor previously.
2. **Clean degrade-to-poll.** When the WebSocket drops (here: the hub is
   stopped), the consumer emits a degrade notice and falls back to the existing
   poll loop. The durable substrate is unchanged — WS is a faster transport, not
   a new source of truth.

## Isolation contract

The demo runs entirely against an **isolated** hub under a temp
`TERMLINK_RUNTIME_DIR` (hub secret + cert) and a temp `HOME` (hubs.toml,
known_hubs). It never touches the shared `:9100` hub or the operator's
`~/.termlink`, and tears the hub down on exit. This is why it is safe to re-run
anywhere as evidence.

## Exact commands (as scripted)

```sh
# isolated hub
TERMLINK_RUNTIME_DIR=$RT termlink hub start --tcp 127.0.0.1:9199 &

# isolated hubs.toml profile so --push can mint a TCP token
#   [hubs.demo-ws]
#   address     = "127.0.0.1:9199"
#   secret_file = "$RT/hub.secret"

# live push consumer (TCP -> TLS[local pinned cert] -> hub.auth -> hub.ws_subscribe)
TERMLINK_RUNTIME_DIR=$RT termlink channel subscribe inbox.queued --push --hub 127.0.0.1:9199

# post a DM to an inbox:* topic -> hub injects the inbox.queued doorbell
TERMLINK_RUNTIME_DIR=$RT termlink channel post inbox:demo-ws-$$ --payload 'hello' --hub 127.0.0.1:9199
```

## Captured output (run 1 of 3)

```
=== arc-004 WS push demo (T-2310) ===
binary:         target/release/termlink
hub:            127.0.0.1:9199   (isolated runtime_dir, torn down on exit)
topic:          inbox:demo-ws-4188639
posted body:    hello-from-demo-4188639
post->push:     91 ms
push frame:     [push] inbox.queued seq=0: {"addressee_session_id":"demo-ws-4188639","channel":"inbox:demo-ws-4188639","enqueued_at":1783017078387,"message_offset":0,"schema_version":"1.0"}
degrade notice: [push] WS unavailable (WebSocket stream error: IO error: peer closed connection without sending TLS close_notify ...) — degrading to poll

RESULT: PASS — push arrived sub-second (91 ms < 1000 ms)
        degrade-to-poll transition observed on WS drop
```

## Reproducibility (3 runs)

| Run | post→push latency | degrade-to-poll | result |
|-----|-------------------|-----------------|--------|
| 1   | 91 ms             | observed        | PASS   |
| 2   | 99 ms             | observed        | PASS   |
| 3   | 93 ms             | observed        | PASS   |

Consistently **sub-100 ms**, well under both the 1 s poll floor and the 15 s
doorbell floor.

## Notes on the measurement

- The reported `post→push` number is measured wall-clock from **just before**
  the `channel post` process is spawned until the push frame appears on the
  consumer's stdout. It therefore **includes** the post binary's own
  spawn + `hub.auth` + RPC round-trip (~tens of ms) plus a 50 ms output-poll
  granularity. It is an **upper bound** — the push after the hub records the
  post is effectively immediate. Even as an upper bound it is ~10× under the
  1 s poll floor and ~150× under the 15 s doorbell floor.
- The degrade notice text varies by how the socket ends: an abrupt hub stop
  surfaces as `WS unavailable (… peer closed connection …) — degrading to poll`
  (the `Err` arm); a graceful stream end surfaces as
  `WS stream ended — degrading to poll` (the `Ended` arm). Both paths degrade to
  the same authoritative poll loop — the outcome the arc guarantees.

## Scope reminder (from S3b / T-2309)

The WS carries hub **aggregator** events. A plain `channel.post` to an
*arbitrary* topic injects none; only `inbox:*` posts inject an `inbox.queued`
doorbell (plus session-forwarded events). So the live-agent value of `--push` is
the **instant DM doorbell** (`--push inbox.queued`, carrying a durable
`message_offset`), which is exactly what this demo exercises — not a generic
per-topic tail.

## Provenance

- Build slices: S1 T-2305 (hub WS upgrade) · S2 T-2306 (broadcast→client push) ·
  S3 T-2307 (per-connection topic filter + degrade-to-poll) · S4 T-2308
  (delivery offset through push, verified contract) · S3b T-2309 (live CLI
  consumer `channel subscribe --push` + `termlink-session::ws_consumer`).
- Documented follow-ons (out of arc-004 GO scope): WS-over-Unix push for
  co-located agents; active reconnect-to-WS with backoff (v1 degrades to poll
  and stays).
