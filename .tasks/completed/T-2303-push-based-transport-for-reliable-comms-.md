---
id: T-2303
name: "Push-based transport for reliable-comms — websockets and/or webhooks"
description: >
  Inception: Push-based transport for reliable-comms — websockets and/or webhooks

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-02T09:24:21Z
last_update: 2026-07-02T15:40:51Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2303: Push-based transport for reliable-comms — websockets and/or webhooks

## Problem Statement

Arc-003 (`reliable-comms`, closed 2026-07-02) delivers cross-agent messages via direct
TCP `channel.post` to the peer's own hub (hub fallback if unreachable), **wakes** the
recipient with a sidecar **doorbell**, then has the recipient **poll** (`channel
subscribe` / `/check-arc`) to read, and journals a delivery-confirm receipt. This
**doorbell-then-poll** model is (a) two round-trips, (b) dependent on the sidecar
doorbell path being live, and (c) built on an event-subscribe long-poll with a known
scaling scar (**PL-089**: long-poll on `event.collect` generated ~13K rpc_audit
entries/sec).

**The question (one go/no-go):** should reliable-comms adopt **push-based transport** —
**WebSockets** (a persistent hub→client stream for live agent sessions) and/or
**webhooks** (HTTP event callbacks for external/non-interactive consumers) — to
**replace or augment** the current model, and is the added server surface +
connection-lifecycle complexity justified, given arc-003 already delivers *reliably*
(so the value on offer is **latency + poll-cost**, not correctness)?

**For whom:** live agent-to-agent sessions (latency); external consumers like Watchtower,
canary crons, CI, peer projects (event fan-out). **Why now:** arc-003 just closed with a
working-but-polling transport seam (V6-S2 `transport-select`), so the seam to swap is
fresh and mapped.

## Assumptions

<!-- Registered via fw assumption add --task T-2303 -->
- **A1** Live agent sessions (PTY processes behind the hub) cannot host an inbound HTTP
  endpoint → **webhooks fit external consumers, not agent-to-agent**. [test: enumerate]
- **A2** The current model's real pain is **latency + poll cost (PL-089)**, not delivery
  correctness (arc-003 confirms delivery). [test: measure baseline]
- **A3** A WebSocket server surface on the hub can **reuse the existing TLS cert (TOFU
  pin) + HMAC secret identity model** (T-1427) without a new auth scheme. [test: WS spike]

## Open Questions

- **IW-1: Can a live agent session receive a webhook (host an inbound HTTP endpoint)?**
  confidence: 3
  disposition: answered
  rationale: No. Live agents are PTY processes behind hub+doorbell with no listening HTTP server; no inbound webhook receiver exists anywhere (only outbound Slack examples). Watchtower Flask (:3003) CAN host. Webhooks = external-only. See report §8.3.
- **IW-2: What is the measured baseline latency + rpc_audit cost of doorbell-then-poll today?**
  confidence: 2
  disposition: answered
  rationale: DM wake latency floor = 15 s sidecar poll interval (`notify-sidecar.sh`) + 1 s receipt granularity; no sub-second path. PL-089's ~13K audit/sec is the `event.collect` aggregator path (skip-listed), NOT DM delivery. Code-grounded, not a live end-to-end measurement (residual — see recommendation DEFER caveat). Report §8.1.
- **IW-3: Does the hub auth model (HMAC + TLS pin, T-1427) extend cleanly to a WS upgrade handshake?**
  confidence: 3
  disposition: answered
  rationale: Yes. HMAC is verified once-per-connection and scope cached for the connection lifetime (`server.rs:508`); a WS authenticates at upgrade and reuses it. A3 confirmed. Report §8.2.
- **IW-4: Is push a REPLACEMENT (primary path) or an AUGMENTATION (opt-in transport)?**
  confidence: 2
  disposition: answered
  rationale: Both, split by path — REPLACE the poll wake/read for the LIVE path (WS), AUGMENT external fan-out (webhooks). Durability layer is NEITHER replaced nor augmented — it stays. Final split is the human's via `fw inception decide`; evidence supports scoped GO. Report §10.
- **IW-5: How does push compose with offline-queue (T-2051), post-idempotency (T-2049), delivery-confirm + journal (arc-003)?**
  confidence: 3
  disposition: answered
  rationale: Push rides ON TOP of the substrate as a faster wake/read transport; queue/idempotency/confirm/journal stay as the durability + exactly-once layer. A dropped WS degrades to the current poll path (substrate authoritative) — latency optimization with safe fallback. Report §8.4.

## Exploration Plan

Time-boxed spikes (read-only / prototype-only — no production build until GO):
1. **Baseline measurement** (~30 min) — instrument current doorbell-then-poll end-to-end
   latency + the `event.subscribe` long-poll rpc_audit cost (PL-089). This is the number
   push must beat. (IW-2, A2)
2. **WebSocket feasibility spike** (~2 h) — can the hub accept a WS upgrade (on :9100 or a
   sibling port)? Does TLS+HMAC identity (T-1427) extend? Minimal hub→client stream
   prototype in a throwaway branch/worktree. (IW-3, A3)
3. **Webhook consumer enumeration** (~30 min) — list the real external consumers that CAN
   host an endpoint (Watchtower, canary crons, CI, peer projects) vs those that can't
   (live agents). (IW-1, A1)
4. **Integration analysis** (~1 h) — how WS/webhooks compose with offline queue +
   idempotency + delivery-confirm + journal; where each guarantee lives. (IW-5)
5. **Directive scoring** (~30 min) — score candidates A–D (below) 0–5 on D1 Antifragility /
   D2 Reliability / D3 Usability / D4 Portability; synthesize recommendation.

**Candidate set (evaluated, not pre-decided):**
- **A — WebSockets (hub→client push stream):** replaces doorbell-then-poll for live sessions.
- **B — Webhooks (hub→endpoint callbacks):** augments external/non-interactive consumers.
- **C — Both, layered:** WS for live, webhooks for external.
- **D — Neither / keep current (null hypothesis):** arc-003 already delivers reliably; must be beaten on measured latency/cost.

## Technical Constraints

- **Agents are PTY processes behind the hub, not HTTP servers** → webhook recipients must
  self-host an endpoint; live agents likely cannot (A1/IW-1).
- **Hub identity model:** TCP JSON-RPC + persistent HMAC secret + TLS cert (TOFU pin). Any
  WS surface MUST reuse this (T-1427), not invent a parallel auth path.
- **G-060 — no inter-hub federation.** Push is **hub↔client only**; it does not change
  cross-hub isolation. Cross-hub still needs explicit cross-posting. OUT of scope.
- **PL-089 scaling scar:** the prior long-poll approach cost ~13K rpc_audit/sec — a WS
  stream must not reproduce per-event audit amplification.
- **Portability (D4):** WS/webhooks are open standards (plus) but add a new server surface
  (upgrade handshake, connection lifecycle, reconnect/heartbeat/backpressure).

## Scope Fence

**IN:** hub↔client transport for reliable-comms (`dm:*` + `agent-presence`); WS-vs-webhook
feasibility + directive scoring; integration with arc-003 primitives (queue/idempotency/
confirm/journal); a single go/no-go recommendation on direction.

**OUT:** inter-hub federation (G-060 — separate concern); replacing the journal / confirm /
idempotency / offline-queue layers (they stay, push rides on them); the actual transport
**build** (that's a post-GO build arc, decomposed then); third-party brokers
(Kafka/NATS/MQTT — a separate inception if anyone raises them).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- A measured baseline (Spike 1) shows the current doorbell-then-poll latency and/or PL-089
  poll cost is a real, quantified pain worth beating, AND
- A push path (WS for live, and/or webhooks for external) **reuses the existing TLS+HMAC
  identity model** (Spike 2/3) with bounded, reversible integration cost, AND
- It composes with the arc-003 primitives (offline queue / idempotency / confirm / journal)
  **without weakening any delivery guarantee** (Spike 4).

**NO-GO if:**
- The current model's latency is acceptable and push buys mostly ergonomics — added server
  surface + connection-lifecycle failure modes (reconnect/heartbeat/backpressure) outweigh
  the benefit (D1/D2 regression), OR
- The hub auth model does not extend cleanly to WS (a parallel auth path is a portability +
  security cost), OR
- Adoption requires reworking the arc-003 delivery/confirm/journal layer (unbounded scope —
  it just shipped).

**DEFER if:** the baseline measurement is promising but a WS prototype needs more than the
time-box to prove the auth/lifecycle path (re-timebox), or external-consumer demand for
webhooks is unconfirmed.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO (scoped) — WebSockets for the live agent path; webhooks as a separate, later, external-only augmentation; keep the arc-003 durability layer underneath.

**Rationale:**

Code-grounded spikes (see `docs/reports/T-2303-push-transport-inception.md` §8–10) confirm all three load-bearing findings: (1) the current DM wake has a **15 s latency floor** (`notify-sidecar.sh` poll interval) with no sub-second path, continuous per-topic poll traffic, and a fragile legacy PTY-inject doorbell (T-2285 miss-gap) — a real, quantified pain, not just ergonomics; (2) a hub WebSocket upgrade is **LOW–MEDIUM effort** because HMAC auth is already verified once-per-connection and reusable at upgrade (`server.rs:508`), events already fan out over `tokio::broadcast` channels (`aggregator.rs:60`), and `handle_connection` is generic over the stream — the only new work is a `tokio-tungstenite` dep + Upgrade handshake + a second concurrent write path; (3) webhooks are **external-only** (Watchtower/Slack/CI can host endpoints; live PTY agents cannot), so they augment external fan-out and cannot replace agent-to-agent delivery. Directive scoring: WS 15 / both 15 / webhooks 14 / keep-current 12 — push wins on Usability (D3) + Reliability (D2) while the durable substrate (journal/receipts/queue/idempotency) stays and provides graceful degradation to polling. GO is scoped to the WS live path; the durability layer is explicitly out of scope for replacement.

**DEFER caveat (the honest reviewer's alternative):** the 15 s floor is read from code constants, not a live end-to-end measurement — a 30-min live baseline against a running hub would firm up the 15 s→sub-second value before committing build effort.

**Evidence:**
- §8.1 current-path cost — `notify-sidecar.sh` (15 s interval), `agent-send.sh:373-395` (receipt re-poll), PL-089 is the `event.collect` path (skip-listed `rpc_audit.rs:46`), NOT DM delivery.
- §8.2 WS feasibility — `server.rs:508` (per-connection HMAC reuse), `aggregator.rs:60` (broadcast fan-out), generic `handle_connection`; no existing WS dep.
- §8.3 webhook consumers — no inbound receiver today; Watchtower Flask `.agentic-framework/web/app.py`:3003 can host; live agents cannot (A1 confirmed).
- §9 directive scoring table; §10 scoped GO + build-arc decomposition (S1–S4).

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

**Decision**: GO

**Rationale**: WS for live path; keep durability layer; webhooks deferred external-only

**Date**: 2026-07-02T14:17:32Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-02T09:27:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-02T14:17:32Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** WS for live path; keep durability layer; webhooks deferred external-only
