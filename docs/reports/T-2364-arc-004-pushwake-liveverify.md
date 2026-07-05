# T-2364 — arc-004 push-wake capability-live re-verification (running .107 hub)

**Date:** 2026-07-05
**Arc:** arc-004 (push-transport), closed=shipped
**Host:** workstation-107 (192.168.10.107)
**Question answered:** is the shipped sub-second push-wake mechanic actually
capability-live on the RUNNING production hub today — or only in the hermetic
bench harness (T-2320)? This is the G-069 "shipped ≠ capability-live"
distinction the arc itself was hardened around (T-2359 fleet-binary canary).

## Running hub is capability-complete (not a stale/ghost binary)

| Check | Result |
|---|---|
| Process | pid 3475796, `/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100`, started 2026-07-04 17:09 |
| systemd supervision | `termlink-hub.service` MainPID=3475796, ActiveState=**active**, NRestarts=0 — the running hub **is** the unit MainPID (no detached ghost; G-070 clean) |
| Source commit | built from `5c171bf1` (T-2355 lineage) → carries every arc-004 rail (WS S1–S4, dm-rail, walk-deadline, webhook fan-out) |
| Version | 0.11.324 (repo HEAD 0.11.358; the 34-commit delta is canaries/docs/handovers/test-fixes — **no** un-deployed hub capability) |
| `hub.governor_status` telemetry | `rate_buckets_evicted_total=133533` (T-2139 ✓), `webhook_*` fields present (T-2332–2336 ✓), `cv_index_*` present (T-2110 ✓), `dedupe_*` present (T-2049 ✓) |

The preflight Check-5 (T-2184) signal — absence of `rate_buckets_evicted_total`
meaning a pre-T-2139 binary — is **negative** here: the field is present, so
the running hub is post-T-2139 and capability-complete for the arc.

## Live push-wake smoke (WS-over-Unix, against the running hub)

Method (mirrors `scripts/demo-ws-push-unix.sh`, T-2313): subscribe to the
`inbox.queued` aggregator stream with `--push` over the hub's Unix socket
(`/var/lib/termlink/hub.sock`), then post to `inbox:<self>` and time
post → push-frame-arrival. Detection is by push-frame-count increment
(the aggregator frame is a metadata notification — `addressee/channel/offset/
enqueued_at` — not the message body, so content-grep does not apply). Latency
is a conservative **upper bound** (5ms poll granularity counted IN).

```
trial 1: NO PUSH within 2s      (warm-up — before-count race, discarded)
trial 2: post->push = 0.119s
trial 3: post->push = 0.100s
trial 4: post->push = 0.102s
trial 5: post->push = 0.092s
trial 6: post->push = 0.107s
trial 7: post->push = 0.128s
trial 8: post->push = 0.125s

trials=7  min=0.092s  median=0.107s  max=0.128s
VERDICT: PASS — sub-second WS push-wake LIVE on running production hub
```

Transport evidence (first frames observed on the `inbox.queued --push` stream):

```
[push] inbox.queued seq=0: {"addressee_session_id":"...","channel":"inbox:...","enqueued_at":1783270000363,"message_offset":0,"schema_version":"1.0"}
[push] inbox.queued seq=0: {"addressee_session_id":"...","channel":"inbox:...","enqueued_at":1783270009951,"message_offset":1,"schema_version":"1.0"}
[push] inbox.queued seq=0: {"addressee_session_id":"...","channel":"inbox:...","enqueued_at":1783270020201,"message_offset":2,"schema_version":"1.0"}
```

## Conclusion

The arc-004 headline mechanic is **capability-live on the running .107 hub**:
median **0.107s** full post→push delivery — consistent with T-2320's hermetic
benchmark (85–111ms median) and ~140× faster than the pre-arc 15s
doorbell-then-poll wake floor. "Shipped" is confirmed to mean
"capability-live here today."

### Notes for the next verifier
- The **raw** `channel subscribe <arbitrary-topic> --push` consumer does NOT
  emit a frame for a plain `channel post` to that topic — push frames are
  driven by the hub's aggregator events (`inbox.queued` / `dm.queued`), so
  subscribe to the **aggregator stream** and post to `inbox:*` / `dm:*`. A
  smoke that subscribes to the raw destination topic will (correctly) see
  silence; this is not a defect.
- The Rust CLI block-buffers stdout when redirected (not a TTY); a `timeout`
  SIGTERM discards buffered frames. Detect arrival by frame-count increment
  with the subscriber left running, or send SIGINT for a graceful flush.
