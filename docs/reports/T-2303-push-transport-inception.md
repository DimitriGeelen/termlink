# T-2303 — Push-based transport for reliable-comms (WebSockets / webhooks)

**Type:** Inception (explore → go/no-go). **Filed:** 2026-07-02. **Recommendation at
filing:** DEFER (no spikes run yet). **Arc under reconsideration:** arc-003
`reliable-comms` (closed 2026-07-02, see `docs/reports/arc-003-reliable-comms-close-demo.md`).

> C-001: this file is the thinking trail — updated incrementally as the exploration
> produces findings. The conversation is ephemeral; this file is permanent.

## 1. The problem

arc-003 delivers cross-agent messages **reliably** but via a **doorbell-then-poll** shape:

```
sender ──channel.post (TCP)──▶ peer's hub :9100        (V6 direct, hub fallback)
                                     │
                                     ▼
recipient ◀── sidecar doorbell (wake) ──┐
recipient ── channel subscribe (POLL) ──┘  reads the turn
recipient ── journaled receipt (stage=delivered) ──▶ confirm
```

Three costs sit in that shape:
1. **Two round-trips** (wake, then poll-to-read) and latency floor set by the poll cadence.
2. **Dependence on the sidecar doorbell path** being live for the wake.
3. **PL-089 scaling scar** — the event-subscribe long-poll approach cost ~13K rpc_audit
   entries/sec, i.e. the "streaming" we do today is expensive.

**Push-based transport** attacks that seam directly. But the critical framing: arc-003
already delivers *reliably* (confirmed, journaled, idempotent, queue-buffered). So the
value push offers is **latency + poll-cost reduction + ergonomics**, NOT correctness. The
go/no-go must beat the *status quo on a measured number*, not on "push is nicer."

## 2. The two technologies are not interchangeable

| | WebSockets | Webhooks |
|---|---|---|
| Direction | persistent, bidirectional, hub→client push stream | one-shot HTTP callback hub→endpoint |
| Fits | **live agent sessions** (replace poll) | **external consumers that host an endpoint** |
| Recipient must | hold an open connection to its hub | run an inbound HTTP server |
| Replaces | doorbell-then-poll for live delivery | nothing for agents; augments dashboards/CI |
| Auth | reuse hub TLS+HMAC over the upgraded socket (IW-3) | endpoint-side secret/signature |

**Load-bearing early hypothesis (A1/IW-1):** live agent sessions are PTY processes reached
only *through* the hub + doorbell — they almost certainly **cannot host an inbound HTTP
endpoint**, so **webhooks are for external consumers, not agent-to-agent**. If that holds,
"replace with webhooks" is a non-starter for the live path and the real replacement
candidate is **WebSockets**.

## 3. Candidate set (to be scored, not pre-decided)

- **A — WebSockets (hub→client push stream):** agent opens a WS to its hub, subscribes to
  its `dm:*` + `agent-presence` topics, hub streams envelopes. Collapses notify+read into
  one push; directly addresses PL-089.
- **B — Webhooks (hub→endpoint callbacks):** augments external/non-interactive consumers
  (Watchtower, canary crons, CI, peer projects) that can host an endpoint.
- **C — Both, layered:** WS for live sessions, webhooks for external. Hybrid.
- **D — Neither / keep current (null hypothesis):** arc-003 already delivers reliably;
  push must be justified on a measured latency/cost win.

## 4. Constraints that shape the answer

- **Agents aren't HTTP servers** (A1) — kills webhooks for the live path.
- **Reuse hub identity** — TLS cert (TOFU pin) + HMAC secret (T-1427). A WS surface must
  ride this, not fork a new auth scheme (portability + security cost otherwise).
- **G-060 (no federation)** — push is hub↔client only; cross-hub isolation is unchanged and
  OUT of scope.
- **PL-089** — a WS stream must not reproduce per-event rpc_audit amplification.
- **arc-003 primitives stay** — offline queue (T-2051), post-idempotency (T-2049),
  delivery-confirm + journal; push rides on them and must not weaken any guarantee (IW-5).

## 5. Exploration plan (time-boxed, no build until GO)

1. Baseline measurement (~30 min) — status-quo latency + PL-089 poll cost. *(IW-2, A2)*
2. WebSocket feasibility spike (~2 h, throwaway) — hub WS upgrade + TLS/HMAC reuse +
   minimal hub→client stream. *(IW-3, A3)*
3. Webhook consumer enumeration (~30 min) — who can actually host an endpoint. *(IW-1, A1)*
4. Integration analysis (~1 h) — compose with queue/idempotency/confirm/journal. *(IW-5)*
5. Directive scoring (~30 min) — candidates A–D × D1–D4, synthesize recommendation.

## 6. Preliminary directive lean (pre-evidence — to be confirmed/overturned by spikes)

- **D1 Antifragility:** poll is resilient to transient drops; a persistent WS adds a
  reconnect/heartbeat failure surface. Neutral-to-slightly-negative for WS unless reconnect
  is clean. Webhooks need retry+idempotency (we have idempotency keys).
- **D2 Reliability:** arc-003 is the reliability floor; push must not sink below it. WS must
  integrate with confirm+journal; webhooks need delivery retry.
- **D3 Usability:** push (esp. WS) is a clear win — real-time, no poll loop, lower latency.
- **D4 Portability:** WS + webhooks are open standards (plus) vs today's bespoke TCP JSON-RPC;
  but each adds a new server surface. Net-neutral, leaning positive on standards.

Preliminary shape (**subject to Spike 1's number**): if the baseline shows real
latency/poll-cost pain → **WS to augment/replace the live poll path** (Candidate A/C) is the
strongest; **webhooks only where an external consumer hosts an endpoint** (Candidate B as a
layer); **keep the arc-003 reliability primitives underneath either**. If the baseline is
fine → lean **D (keep current)**, since push would add failure surface for ergonomics.

## 7. Dialogue Log

### 2026-07-02 — inception opened
- **Human:** "incept the addition/replacement of webhooks and websockets for the work we
  just did [arc-003]" + "reflect, tell me you understand."
- **Agent:** reflected the current doorbell-then-poll model back; noted WS ≠ webhooks (live
  vs external); flagged *one inception = one question*; asked scope (add/replace/explore) +
  tech (WS/webhooks/both).
- **Human:** away (no answer in 60s).
- **Agent decision:** proceed with the most defensible framing — scope = *explore then
  recommend* (an inception shouldn't pre-decide add-vs-replace); tech = evaluate *both* WS
  and webhooks held to one go/no-go on direction. Filed T-2303 (DEFER), wrote this artifact
  and the exploration plan, paused **before** spikes for human review of the plan (inception
  discipline step 2). Scope/tech question to be re-confirmed when the human returns.
