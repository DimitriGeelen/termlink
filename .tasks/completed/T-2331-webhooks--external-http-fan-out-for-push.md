---
id: T-2331
name: "Webhooks — external HTTP fan-out for push-transport (arc-004 Candidate B)"
description: >
  Inception: Webhooks — external HTTP fan-out for push-transport (arc-004 Candidate B)

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: [T-2303, T-2322, T-2325]
created: 2026-07-03T09:36:09Z
last_update: 2026-07-03T09:49:01Z
date_finished: 2026-07-03T09:49:01Z
revisit_at: 2026-10-01           # T-1451: quarterly checkpoint; the REAL trigger is demand-gated (see revisit_evidence_needed)
revisit_evidence_needed: "A concrete external HTTP consumer wanting hub push is named with a real workflow (Watchtower live page, Slack/PagerDuty alert, or CI trigger)."
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2331: Webhooks — external HTTP fan-out for push-transport (arc-004 Candidate B)

## Problem Statement

arc-004 (`push-transport`) shipped hub→client **WebSocket** live push for agent-to-agent
delivery (Candidate A, T-2322–T-2325). The T-2303 inception named a second candidate —
**webhooks** — but deferred it as "external-only augmentation (A1)" without a first-class,
surfaced decision record. This inception promotes that buried sub-decision into a standalone,
revisitable go/no-go: **should TermLink build outbound HTTP webhooks now, so the hub can push
events to external (non-agent) consumers — Watchtower live pages, Slack/PagerDuty alerts, CI
triggers?**

Who: operators wiring TermLink into external observability/automation. Why now: the standing
directive explicitly names "websockets and/or webhooks"; the WS half is shipped, so the webhook
half deserves an explicit, recorded decision rather than remaining an implicit deferral buried
inside T-2303.

## Assumptions

- **A1 (inherited, confirmed T-2303):** Agents are NOT HTTP servers — they cannot receive an
  inbound webhook. ⇒ webhooks can only ever be an OUTBOUND channel to external consumers, never
  an agent-to-agent live-delivery mechanism. This is the assumption that caps webhooks' value.
- **A2:** The agent-to-agent live-push need is FULLY met by the shipped WebSocket path — a
  webhook adds zero coverage for the agent↔agent case. (Evidence: arc-004 closed=shipped.)
- **A3:** There is currently NO named external consumer that needs hub→HTTP push. Watchtower
  polls today; Slack/PagerDuty/CI integration has not been requested by any real workflow.
- **A4:** The hub is greenfield for outbound HTTP — no `reqwest`/`hyper`/`ureq` client in any
  crate manifest — so a webhook feature is a net-new dependency + retry/security surface, not
  an incremental extension. (Verified 2026-07-03, see Evidence.)

## Open Questions

- **IW-1: Can webhooks carry agent-to-agent live delivery (replace/augment WS for the agent case)?**
  confidence: 3
  disposition: dissolved
  rationale: A1 — agents aren't HTTP servers, so they cannot receive inbound webhooks. Moot for
  the agent↔agent case; webhooks are outbound-to-external only.

- **IW-2: Is there a concrete external consumer today that justifies building webhook fan-out?**
  confidence: 2
  disposition: deferred
  rationale: No named consumer exists (A3). This is the demand-gated trigger — DEFER until one is
  named. Captured in `revisit_evidence_needed`.

- **IW-3: If built, what is the technical shape and cost?**
  confidence: 2
  disposition: answered
  rationale: Net-new outbound HTTP client in `termlink-hub` (reqwest/hyper), a webhook-target
  config surface, per-target retry/backoff + dead-letter, and a security review (SSRF allowlist +
  HMAC payload signing). ~small subsystem (target_blast_radius 3). See Technical Constraints.

## Exploration Plan

This is a decision inception — no prototype is warranted while IW-2 (demand) is unmet. The
exploration is desk research, already completed:
1. Re-read T-2303's webhook deferral + directive scoring (14 vs WS 15). ✅
2. Confirm the hub has no outbound HTTP client (greenfield cost). ✅ (grep, 2026-07-03)
3. Confirm the agent-to-agent need is WS-covered (arc-004 closed=shipped). ✅
4. Enumerate the external-consumer use cases that WOULD justify a GO (Watchtower live push,
   Slack/PagerDuty alert, CI trigger) — none currently requested. ✅

No spikes: building an outbound HTTP client to validate demand that doesn't exist would be the
make-work this inception exists to avoid.

## Technical Constraints

- **Greenfield outbound HTTP:** no `reqwest`/`hyper`/`ureq`/`isahc`/`surf` in any crate manifest
  (verified 2026-07-03). A webhook feature adds a net-new async HTTP client to `termlink-hub`,
  with its own TLS, connection-pool, and timeout surface.
- **SSRF / egress safety:** a hub that POSTs to operator-configured URLs is an SSRF vector
  (internal-network probing via crafted targets). A GO requires an allowlist + egress policy.
- **Delivery semantics:** external HTTP is best-effort — needs per-target retry/backoff, a
  dead-letter path, and idempotency keys (mirrors the substrate offline-queue design, T-2051).
- **Payload authenticity:** external consumers must verify the payload came from the hub —
  HMAC-signed payloads (distinct webhook-signing key, not the peer-auth `hub.secret`).
- **Portability (Directive 4):** outbound HTTP must not become a hard dependency of the core
  substrate — webhooks must be an opt-in, compile-or-config-gated augmentation.

## Scope Fence

**IN scope (this inception):** the go/no-go on building outbound webhook fan-out; enumerating
use cases, cost, and the demand trigger.

**OUT of scope:** any webhook implementation code (no GO recorded); inbound webhooks (dissolved
by A1); replacing/augmenting the WS agent-to-agent path (covered, arc-004); Watchtower's own
polling→push migration (separate concern — Watchtower is a consumer, not the hub).

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

**GO if:**
- A concrete external consumer is named with a real workflow (e.g. "Watchtower should live-update
  via hub push instead of polling", or "on-call wants hub→PagerDuty on capacity-refuse"), AND
- that consumer's need cannot be met more cheaply by the existing poll / `channel subscribe`
  surface (an external adapter that polls and re-emits), AND
- the SSRF/allowlist + HMAC payload-signing security requirements are accepted as part of the build.

**NO-GO / DEFER if:**
- No external consumer is named (current state) — webhooks would be a speculative feature with
  zero coverage gain over the shipped WS path (A2), OR
- the demand can be satisfied by an external adapter polling `channel subscribe` (no hub change).

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

**Recommendation:** DEFER

**Rationale:**

Webhooks cannot carry agent-to-agent live delivery: assumption A1 (agents are not HTTP servers, confirmed in T-2303) means an agent cannot receive an inbound webhook, so webhooks are strictly an EXTERNAL fan-out channel (hub -> Watchtower/Slack/CI). The agent-to-agent live-push need is already met by the shipped WebSocket path (arc-004 Candidate A, T-2322-2325). Directive scoring in T-2303 put webhooks at 14 vs WS 15. No external HTTP consumer currently exists to justify the build, and the hub is greenfield for outbound HTTP (no reqwest/hyper client — non-trivial new dependency + retry/security surface). DEFER until a concrete external consumer materializes; revisit trigger and evidence-needed captured in the task frontmatter.

**Evidence:**

- No outbound HTTP client in the workspace: `grep -rnE '^\s*(reqwest|hyper|ureq|isahc|surf)\b'
  crates/*/Cargo.toml Cargo.toml` → NONE (2026-07-03). Greenfield confirmed (A4).
- No existing webhook code: `grep -rln -i webhook crates/` → NONE (2026-07-03).
- arc-004 registry `.context/arcs/push-transport.yaml`: status=closed, decision=shipped (WS).
  Agent-to-agent live-push need met (A2).
- T-2303 inception deferred webhooks as "external-only augmentation (A1)"; directive score
  14 (webhooks) vs 15 (WS).
- Full analysis: `docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

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

**Rationale**: Recommendation: DEFER

Rationale:

Webhooks cannot carry agent-to-agent live delivery: assumption A1 (agents are not HTTP servers, confirmed in T-2303) means an agent cannot receive an inbound webhook, so webhooks are strictly an EXTERNAL fan-out channel (hub -> Watchtower/Slack/CI). The agent-to-agent live-push need is already met by the shipped WebSocket path (arc-004 Candidate A, T-2322-2325). Directive scoring in T-2303 put webhooks at 14 vs WS 15. No external HTTP consumer currently exists to justify the build, and the hub is greenfield for outbound HTTP (no reqwest/hyper client — non-trivial new dependency + retry/security surface). DEFER until a concrete external consumer materializes; revisit trigger and evidence-needed captured in the task frontmatter.

Evidence:

- No outbound HTTP client in the workspace: `grep -rnE '^\s(reqwest|hyper|ureq|isahc|surf)\b'
  crates//Cargo.toml Cargo.toml` → NONE (2026-07-03). Greenfield confirmed (A4).
- No existing webhook code: `grep -rln -i webhook crates/` → NONE (2026-07-03).
- arc-004 registry `.context/arcs/push-transport.yaml`: status=closed, decision=shipped (WS).
  Agent-to-agent live-push need met (A2).
- T-2303 inception deferred webhooks as "external-only augmentation (A1)"; directive score
  14 (webhooks) vs 15 (WS).
- Full analysis: `docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

**Date**: 2026-07-03T09:49:01Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-07-03T09:39:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-07-03T09:49:01Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: DEFER

Rationale:

Webhooks cannot carry agent-to-agent live delivery: assumption A1 (agents are not HTTP servers, confirmed in T-2303) means an agent cannot receive an inbound webhook, so webhooks are strictly an EXTERNAL fan-out channel (hub -> Watchtower/Slack/CI). The agent-to-agent live-push need is already met by the shipped WebSocket path (arc-004 Candidate A, T-2322-2325). Directive scoring in T-2303 put webhooks at 14 vs WS 15. No external HTTP consumer currently exists to justify the build, and the hub is greenfield for outbound HTTP (no reqwest/hyper client — non-trivial new dependency + retry/security surface). DEFER until a concrete external consumer materializes; revisit trigger and evidence-needed captured in the task frontmatter.

Evidence:

- No outbound HTTP client in the workspace: `grep -rnE '^\s(reqwest|hyper|ureq|isahc|surf)\b'
  crates//Cargo.toml Cargo.toml` → NONE (2026-07-03). Greenfield confirmed (A4).
- No existing webhook code: `grep -rln -i webhook crates/` → NONE (2026-07-03).
- arc-004 registry `.context/arcs/push-transport.yaml`: status=closed, decision=shipped (WS).
  Agent-to-agent live-push need met (A2).
- T-2303 inception deferred webhooks as "external-only augmentation (A1)"; directive score
  14 (webhooks) vs 15 (WS).
- Full analysis: `docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-3c0a97d3
- **Timestamp:** 2026-07-03T09:49:02Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-3
     - evidence: `IW-3 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

### 2026-07-03T09:49:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
