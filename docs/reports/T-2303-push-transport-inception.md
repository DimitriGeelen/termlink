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

### 2026-07-02 — spikes run (code-grounded), recommendation upgraded DEFER → GO
- **Human:** re-sent the same request twice; away both times after asking. Scope/tech
  question posed twice (60s timeout both).
- **Agent decision:** running spikes is delegated *initiative* (the go/no-go is the human's),
  so advanced the exploration via **codebase-grounded feasibility research** (no prototype/
  build artifacts — inception discipline #3). Three research strands (2 subagents + 1 inline).
  Findings below; recommendation upgraded to **GO (scoped)**; decision still the human's via
  `fw inception decide`.

## 8. Findings (spikes 1–4, code-grounded)

### 8.1 Current "doorbell-then-poll" cost — the value target (IW-2, A2)
- **Wake:** two mechanisms. Legacy PTY-inject doorbell (T-1800) rings a keystroke into the
  peer's PTY — but mid-turn keystrokes are **dropped** (T-2285 "miss-gap"), the fragility
  that motivated the sidecar. Current deterministic sidecar (T-2294, `notify-sidecar.sh`)
  does **not** inject — it polls `channel unread` per `dm:` topic and flips a local flag +
  heartbeat.
- **Read:** repeated **single-shot polls** (`channel unread` + `channel subscribe` cursor
  snapshots), not a blocking stream. Sender confirms delivery by re-polling `channel
  subscribe` for the receipt (`agent-send.sh:373-395`, `sleep 1`, timeout 10s × 3 rings).
- **Latency floor:** dominated by the **15 s sidecar interval** (`notify-sidecar.sh`, min
  5 s) + 1 s receipt-poll granularity. **No sub-second wake path exists.**
- **PL-089:** the ~13K rpc_audit/sec was the `event.collect` aggregator fan-in, **masked by
  a skip-list** (`rpc_audit.rs:46`), not reduced — and it is **not** the DM path (DM uses
  `channel unread`/`subscribe`). So PL-089 is an *adjacent* scar, not the DM cost. The DM
  cost is the 15 s latency floor + continuous per-topic poll traffic.
- **Keep underneath (do NOT touch):** durable `dm:` topics + offset cursors + journal +
  `stage=delivered` receipts + offline queue (T-2051) + heartbeat deaf-detection
  (`notify-check.sh`). **Push changes the wake/read, not the durability layer.**

### 8.2 WebSocket feasibility — LOW–MEDIUM (IW-3, A3) ✅ A3 holds
- Hub = pure `tokio` + raw `TcpListener`, TLS via `tokio-rustls` terminating per-connection
  (`server.rs:677`), yielding a **generic `AsyncRead+AsyncWrite`** stream — `handle_connection`
  is already generic over `S`.
- **Auth reuses cleanly:** HMAC is verified **once per connection** and the scope cached for
  its lifetime (`server.rs:508`); a WS can authenticate at the upgrade and reuse it. **A3
  confirmed.**
- **Push primitive already exists:** events fan out internally over `tokio::broadcast`
  channels (`aggregator.rs:60`) — exactly what a WS push loop drains. Today clients re-poll
  (`event.subscribe` 5 s, `channel.subscribe` 60 s cap) instead of receiving pushes.
- **Cost:** add `tokio-tungstenite` + an HTTP `Upgrade` handshake in front of the JSON-RPC
  accept path + a **second concurrent write path** (select over broadcast receiver + inbound
  frames) alongside the half-duplex line loop. Bounded, additive, reversible.

### 8.3 Webhook consumers — external-only (IW-1, A1) ✅ A1 holds
- **No inbound webhook receiver exists today.** Only *outbound* Slack paging examples
  (`docs/operations/substrate-cron-recipes.md`).
- **Can host an endpoint:** Watchtower (Flask, `.agentic-framework/web/app.py`, :3003),
  external services (Slack already outbound), CI/OneDev.
- **Cannot:** live agent sessions (PTY behind hub+doorbell, no listening HTTP server);
  ephemeral cron scripts (pull-based). **A1 confirmed — webhooks augment external fan-out,
  they cannot carry agent-to-agent live delivery.**

### 8.4 Integration (IW-5)
Push rides **on top of** the existing substrate: the WS stream is a faster *transport for the
wake+read*, while offline-queue / post-idempotency / delivery-confirm / journal remain the
durability + exactly-once layer unchanged. A dropped WS degrades to the current poll path
(the substrate is still authoritative) — so push is a latency optimization with a safe
fallback, not a new source of truth.

## 9. Directive scoring (0–5)

| Candidate | D1 Antifragility | D2 Reliability | D3 Usability | D4 Portability | Total |
|---|---|---|---|---|---|
| **A — WebSockets (live path)** | 3 | 4 | 5 | 3 | **15** |
| **B — Webhooks (external)** | 3 | 3 | 4 | 4 | **14** |
| **C — Both, layered** | 3 | 4 | 5 | 3 | **15** |
| **D — Keep current (null)** | 4 | 3 | 2 | 3 | **12** |

Rationale: A/C win on **D3** (sub-second push vs 15 s floor, no poll loop) and **D2** (removes
the fragile PTY-inject miss-gap; rides the same durable substrate). D scores highest on **D1**
(proven, antifragile heartbeat-deaf-detection) but lowest overall — the status-quo pain is
real. B is the most *portable* integration but only for external consumers.

## 10. Recommendation — GO (scoped)

**GO to a scoped build, NOT a wholesale replacement:**
1. **WebSockets for the live agent path (Candidate A)** — the high-value core. Replace the
   15 s-floor doorbell-then-poll wake+read with a hub→client WS push stream that drains the
   existing broadcast channels; authenticate at the upgrade (reusing HMAC scope); **keep the
   durable substrate + receipts + queue underneath**, with graceful degradation to polling if
   the socket drops. Feasibility LOW–MEDIUM; bounded, additive, reversible.
2. **Webhooks as a SEPARATE, later, external-only augmentation (Candidate B)** — for
   Watchtower / Slack / CI fan-out. Lower priority; its own inception/build if external demand
   materializes. **Not** a path for agent-to-agent delivery (A1).
3. **Do NOT replace** the journal / receipts / idempotency / offline-queue layer — push is a
   wake/read transport, the durability layer stays.

**Honest limitation:** the 15 s latency floor is read from code constants, not a live
end-to-end measurement. If the human wants the value firmed up before committing build effort,
a 30-min live baseline (Spike 1 executed against a running hub) would confirm the
15 s→sub-second delta is worth the added concurrent-write-path complexity. This is the one
reason a reasonable reviewer might choose DEFER over GO.

**If GO:** decompose into a build arc — S1 hub WS upgrade endpoint (dep + handshake + auth
reuse), S2 concurrent write path draining broadcast → client, S3 client WS subscribe +
degrade-to-poll fallback, S4 wire delivery-confirm/journal through the WS path unchanged. The
substrate stays; only the wake/read transport changes.
