# T-2331 — Webhooks: external HTTP fan-out for push-transport (arc-004 Candidate B)

**Type:** Inception (one question, one go/no-go)
**Recommendation:** DEFER (demand-gated)
**Date:** 2026-07-03
**Parent:** T-2303 (push-based transport inception — chose WebSockets, deferred webhooks)
**Arc:** arc-004 `push-transport` (registry `.context/arcs/push-transport.yaml`, status=closed/shipped)

---

## The one question

Should TermLink build **outbound HTTP webhooks** now — so the hub can push events to
**external, non-agent** consumers (Watchtower live pages, Slack/PagerDuty alerts, CI
triggers)?

This is deliberately *not* "should we have push transport" (answered YES → WebSockets, shipped)
and *not* "webhooks vs websockets" (answered by T-2303 → WS won 15–14). It is the narrow,
still-open half of the directive's "websockets **and/or** webhooks": now that WS is shipped, do
we also build the webhook augmentation?

## Why this inception exists at all

T-2303 already analysed webhooks and deferred them — but as a *sub-clause* inside a broader
inception, with no standalone, surfaced, revisitable decision record. That means:
- the deferral has no `revisit_at` / trigger of its own (it can't ripen);
- it never appears on the operator's approval surface as a decision they made;
- "and/or webhooks" in the directive has no crisp, cited answer to point at.

This inception promotes the buried sub-decision into a first-class one. Even a DEFER is
valuable here: it converts an implicit assumption into an explicit, evidenced, revisitable
decision with a named trigger.

## Assumptions (see task frontmatter for the canonical A1–A4)

| # | Assumption | Status |
|---|---|---|
| A1 | Agents are not HTTP servers → cannot receive inbound webhooks | confirmed (T-2303) |
| A2 | Agent↔agent live-push is fully met by the shipped WS path | confirmed (arc-004 shipped) |
| A3 | No named external consumer needs hub→HTTP push today | current state |
| A4 | Hub is greenfield for outbound HTTP (no reqwest/hyper/…) | verified 2026-07-03 |

## The load-bearing logic

1. **A1 caps the ceiling.** Because an agent cannot host an HTTP endpoint, a webhook can never
   deliver *agent-to-agent*. Webhooks are, by construction, an **outbound-to-external** channel
   only. This is not a limitation to engineer around — it is definitional.
2. **A2 removes the urgency.** The agent-to-agent live-delivery need — the whole reason arc-004
   existed — is already met by WebSockets. Webhooks would add **zero** coverage for that case.
3. **So webhooks' entire value is external fan-out** (hub → Watchtower/Slack/PagerDuty/CI).
4. **A3 says that value has no buyer yet.** No workflow currently asks for it. Watchtower polls
   and is fine; no alert/CI integration has been requested.
5. **A4 says the price is non-trivial.** It's a net-new async HTTP client in `termlink-hub`,
   plus retry/backoff/dead-letter, plus a real security surface (SSRF allowlist, HMAC signing).

Building a feature whose only value is external fan-out, when no external consumer exists, at a
non-trivial cost, is textbook speculative work. Hence **DEFER**, not NO-GO — the moment a real
external consumer is named, the calculus flips and this should be revisited.

## What a GO would entail (so the estimate is honest, not hand-wavy)

- **Dependency:** add `reqwest` (or `hyper`) to `termlink-hub` — new TLS/pool/timeout surface.
- **Config:** a webhook-target registry (URL, events subscribed, signing key, retry policy).
- **Delivery:** per-target retry/backoff + dead-letter; idempotency keys (reuse the T-2051
  offline-queue design rather than reinvent).
- **Security (mandatory before any GO):**
  - SSRF allowlist — the hub POSTing to arbitrary operator URLs can probe internal networks.
  - HMAC-signed payloads with a **distinct** webhook-signing key (never the peer-auth
    `hub.secret`), so consumers can verify origin.
- **Portability (Directive 4):** opt-in / config- or compile-gated — outbound HTTP must not
  become a hard dependency of the core substrate.
- **Blast radius:** ~small subsystem (target_blast_radius = 3).

## The cheaper alternative that usually wins

Most "I want hub events in Slack/CI" needs are satisfied by an **external adapter** that polls
`termlink channel subscribe <topic> --json` (or the WS stream) and re-emits to the external
system — **zero hub changes, no new egress surface in the trusted core.** A GO is only justified
if a named consumer's need genuinely cannot be met this way.

## Recommendation

**DEFER.** Revisit when `revisit_evidence_needed` is satisfied: *a concrete external HTTP
consumer wanting hub push is named with a real workflow.* Quarterly checkpoint `revisit_at:
2026-10-01` (the G-053 daily scan will surface it), but the true gate is demand, not the date.

## Go/No-Go (canonical copy in the task file)

**GO if** a concrete external consumer is named AND its need can't be met more cheaply by a
poll/subscribe adapter AND the SSRF+signing security work is accepted.
**NO-GO/DEFER if** no consumer is named (current state) OR the need is adapter-satisfiable.

## Decision authority

`fw inception decide` is sovereignty-gated (human-only under $CLAUDECODE=1). This artifact and
the task scope are the agent's advisory; the human records the decision via `fw task review
T-2331` / Watchtower.

## Dialogue Log

- 2026-07-03 — Operator standing directive names "websockets **and/or** webhooks"; asked
  earlier whether the webhooks inception "is ready / will surface in approval." It was not a
  task, so nothing surfaced. This inception was created (agent initiative — only `decide` is
  gated) and fully scoped with a DEFER recommendation so the decision surfaces on the approval
  queue. Agent verified the greenfield-HTTP claim (grep, no outbound client) before asserting it.
