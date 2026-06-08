# T-2028 Inception Research — Substrate primitive #10: throughput/connection budget + retention/compaction policy

**Status:** PARTIAL-GO with three sub-tracks. Retention/compaction already EXISTS — verify + document only. Connection cap + rate limit + observability are real gaps; file separate small build tasks.
**Artifact created:** 2026-06-08
**See also:** T-2018 ADR §6 #10; T-1991 (agent-presence bloat precedent); T-2027 (the broadcast-with-replay primitive — its `retention: keep-latest` half belongs here).

## 1. The §6 framing

ADR §6 primitive #10: *"No connection cap, rate limiter, or backpressure governor exists. T-1991 (agent-presence bloat) was found in PRODUCTION, not predicted. Retention/compaction must be designed in from the start, not bolted on."*

Three distinct sub-problems bundled into one §6 entry. Investigation reveals they have very different statuses.

## 2. Ground-truth from the running substrate

```
crates/termlink-bus/src/retention.rs    → defines `enum Retention { Forever, Days(N), Messages(N) }`
crates/termlink-bus/src/lib.rs:92-103   → create_topic(name, retention) + topic_retention(topic) API
crates/termlink-bus/src/lib.rs:375-385  → compaction logic: per-topic keep_after / keep_last computation
crates/termlink-hub/src/rpc_audit.rs:685 → only stub: warn_if_legacy_rate_limits_within_window (no enforcement)
```

| §6 sub-problem                             | Current state                                                          |
|--------------------------------------------|------------------------------------------------------------------------|
| Per-topic retention (Days/Messages/Forever) | ✅ Exists, in production (e.g. `agent-presence: Forever`)             |
| Compaction logic (size/age based)           | ✅ Exists, runs on retention policy                                    |
| `keep-latest` retention (T-2027's sibling)  | ❌ Missing; small addition (`Retention::Latest` variant)                |
| Connection cap                              | ❌ Missing. No `max_connections` config. No 429-style refusal path.    |
| Rate limit per-sender / per-topic / per-RPC | ❌ Missing. No leaky-bucket or token-bucket. Stub in `rpc_audit` only. |
| Budget visible in `channel info` / status   | ❌ Not exposed. Operators can't see "how close am I to the limit?".    |

The §6 framing slightly overstates the gap. Retention IS designed in (per-topic policy at topic creation time, compaction enforces it). What's missing is the connection/rate budget half, where the substrate has no production-grade governor at all.

## 3. T-1991 precedent — what would have helped?

T-1991 surfaced as `agent-presence` topic with ~1800 envelopes wedging tokio workers during `channel.subscribe`. Root cause turned out to be **per-binary-version slowdown** in the subscribe path (a regression), not topic-size per se — but the symptom was triggered by size growth.

Two distinct mitigations:

1. **Subscribe-path resilience** — fix the per-version slowdown directly (was the actual fix shipped).
2. **Topic-size bounding** — set `agent-presence` retention to `Messages(N)` (e.g. N=200, enough to cover ~100 minutes of heartbeats at 30s interval × ~5 agents). This is operator policy that ALREADY exists as a verb (just hasn't been applied to that topic).

Lesson: the framework has the capability; the gap was operator awareness. Improved observability (the third sub-track below) would have surfaced "topic X is growing 60 envelopes/minute, retention is Forever, runway-to-pain ~30 min" before the wedge.

## 4. Three sub-tracks, three recommendations

### Track A — Retention/compaction: AUDIT + small additive surface

- **Recommendation:** Audit existing primitives' design against the available retention policies; ensure each topic created by the substrate sets a retention. (`agent-presence` should be `Messages(200)` not `Forever` — operator decision, file as a task on GO.)
- **Add `Retention::Latest`** as a one-line enum variant + compaction case (keep most recent envelope, evict the rest). This is T-2027's deferred sibling.
- **Cost:** ~30 LOC, 1 vertical slice. Pure additive.
- **Conditional on T-2027 GO:** worth bundling — `--from-latest` (subscribe-side, T-2027) + `Retention::Latest` (compaction-side, here) together complete the broadcast-with-replay story.

### Track B — Connection cap + rate limit: SMALL BUILD

- **Recommendation:** File a small build task. Per-process connection cap (e.g. `MAX_CLIENT_CONNECTIONS=64` configurable; refuse new connections with a clear "hub at capacity, retry in N s" RPC error). Per-sender token bucket (e.g. 100 RPCs/s/sender, with `X-RateLimit-*` style headers in the RPC response).
- **Cost:** ~150 LOC, 3 vertical slices (governor module, RPC envelope extension, observability into `hub status`).
- **Refusal must be LOUD** per IW-3 hint — no silent dropping. Return a structured RpcError with `code=-32029 OVERLOADED`, `retry_after_ms` in data, surfaced in CLI as `hub at capacity (retry in 2.3s)`.

### Track C — Budget observability: SMALL BUILD

- **Recommendation:** Surface budgets in `termlink channel info <topic>` (current size, retention policy, growth rate over last hour) and `termlink hub status` (connection count, rate-limit hits in last hour, top senders by RPC count).
- **Cost:** ~80 LOC, 2 vertical slices (hub-side counters, CLI/MCP read paths).
- **Wins:** T-1991-style "found in production not predicted" failure mode becomes a "fleet doctor surfaces it" event.

## 5. IW dispositions

- **IW-1 (per-topic retention API — exists or add?):** EXISTS as `enum Retention { Forever, Days(N), Messages(N) }`. One small addition (`Retention::Latest`) is worth shipping as T-2027's compaction sibling. Confidence=4.
- **IW-2 (compaction trigger — time/size/both, per-topic):** EXISTS as per-topic policy (Days = time-based, Messages = size-based). Both modes available. Confidence=4.
- **IW-3 (connection cap — per-process / per-host / per-hub; refuse vs queue):** PER-PROCESS, REFUSE. Per-process matches the deployment shape (one hub per host typically); refuse-with-structured-error is loud per IW-3 hint (G-058 silent-failure precedent). Confidence=3.
- **IW-4 (rate limit — per-sender / per-topic / per-RPC; visible in `topic info`):** PER-SENDER. Per-topic adds policy complexity for limited gain; per-RPC is too granular. Per-sender bucket aligns with the trust-model (HMAC identifies the sender). Visible via `hub status` and per-RPC response headers. Confidence=3.
- **IW-5 (T-1991 precedent — would-have-helped policy):** Two-pronged answer (subscribe-path resilience + topic-size bounding) — the second was always available, just not applied. Real fix was the first, but better observability would have surfaced both options before the wedge. Confidence=4.

## 6. Recommendation

**PARTIAL GO** with three sub-tracks:

- **Track A (Retention)** — GO, scope = audit + add `Retention::Latest`. Bundle with T-2027 build task if T-2027 goes.
- **Track B (Connection + Rate)** — GO as a separate small build task. Loud-refuse semantics, observable.
- **Track C (Observability)** — GO as a separate small build task. Wires budgets into existing `channel info` + `hub status`.

**Why not one big build task:** the three tracks are independent and have different sizes; bundling creates a 350+ LOC task that's harder to slice. Three small tasks ship cleanly in order.

**Why not DEFER the whole thing:** the retention/compaction half ALREADY exists in code and just needs an audit; the connection/rate half is concretely missing and the design is well-understood (standard governor pattern). No measurement spike needed. Substrate is well-positioned to ship.

## 7. GO criteria evaluation (from §Go/No-Go Criteria)

- ✅ "Cross-cutting review surfaced concrete budget" — three tracks, each scoped and sized.
- ✅ "Each primitive's design respects it" — retention already does; connection/rate to be added with explicit per-sender bucket.
- ✅ "Missing policy primitives bounded and small" — 30 + 150 + 80 LOC, well-bounded.

## 8. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §3 "durable channel logs" | ✓ Retention/compaction is the durability/size trade-off; explicit per-topic policy preserves operator control. |
| §5 "one writer, serialized" | ✓ Rate-limit governor adds a *check* before the write; doesn't change the writer count. |
| §6 #10 framing | ✓ Resolved: 3 sub-tracks, each with concrete scope. |
| §9 "AEF will generate exactly that traffic class" | ✓ Per-sender rate limit + observability mean AEF can self-throttle, and operators see the budget. |

## 9. Open follow-up tasks to file on GO

- **Track A:** Audit task — review each topic created by substrate code, ensure retention is set; bundle `Retention::Latest` addition with T-2027 build.
- **Track B:** Build task — connection cap + per-sender rate limit + loud-refuse RPC error.
- **Track C:** Build task — surface budget state in `channel info` + `hub status`; CLI + MCP read paths.
- *(Operator decision)* — set `agent-presence` topic retention to `Messages(200)` (or measure-informed N) to prevent T-1991 recurrence. Small operator-side change once Track A lands.
