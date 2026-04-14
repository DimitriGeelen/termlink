---
id: T-1051
name: "TermLink auth/connect reliability — antifragile, self-healing hub authentication"
description: >
  Inception: TermLink auth/connect reliability — antifragile, self-healing hub authentication

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T07:09:35Z
last_update: 2026-04-14T07:10:29Z
date_finished: null
---

# T-1051: TermLink auth/connect reliability — antifragile, self-healing hub authentication

## Problem Statement

TermLink hub authentication silently breaks across the fleet whenever a hub regenerates its HMAC secret or TLS cert, with no in-band mechanism for clients to discover the new state. Hit by two independent agents on .109 within 24h (2026-04-14). Full analysis in `docs/reports/T-1051-termlink-auth-reliability-inception.md`.

## Assumptions

A-001: Hub TLS cert regeneration on every restart is the dominant TOFU failure cause. (Partially addressed by T-945/T-1028.)
A-002: Hub HMAC secret regeneration on every restart is the dominant auth failure cause. (Partially addressed by T-933.)
A-003: Clients cannot self-heal — no bootstrap protocol (chicken-and-egg: need secret to authenticate, need authentication to obtain new secret).
A-004: Stale-secret failures persisting multiple days are primarily a *detection* failure, not a recovery failure.
A-005: Most hub restarts are operator-initiated.

## Exploration Plan

Five spikes (see research artifact). Spikes 1+2 completed 2026-04-14.

## Technical Constraints

- Existing primitives preserved: TOFU + HMAC auth.
- Project boundary enforcement (T-559) blocks client-side writes to `/root/.termlink/` from within /opt/termlink agents.
- No key-management service (Vault, etc.) — over-engineering for current scale.
- SSH to hub hosts is not always available for manual recovery.

## Scope Fence

IN SCOPE: secret rotation model, TOFU pinning model, operator UX for unavoidable manual paths, detection/alerting on stale credentials, fleet-doctor enhancements.

OUT OF SCOPE: TLS story rewrite, key management services, replacing the TOFU+HMAC primitives.

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO on Option D (hybrid minimum viable antifragile)

**Rationale:** Persist-if-present code (T-933, T-945/T-1028, T-1031) is correct and deployed — the design gap is the absence of a rotation-announce protocol and of structural detection/alerting when auth persistently fails. Option D is the smallest viable delta: persist-by-default + auto-self-register a learning on first auth-mismatch (carrying `date_observed` + `hub_fingerprint` for memory-drift detection) + auto-register a concern after persistent fleet-doctor failure (G-019 compliance) + `termlink fleet reauth <profile>` one-command operator heal (with optional `--bootstrap-from` anchor, out-of-band required). Options A (two-tier root+session) is premature crypto; B is inside D; C folds into D. Peer reviewed by ring20-dashboard session (independent hit of same failure class within 24h).

**Evidence:**
- Two independent agents on .100 and .121 hit the same auth-mismatch on .109 within 24h.
- Code inspection confirms persist-if-present is implemented correctly at all three layers (hub secret, TLS cert, restart runtime_dir handoff).
- Client secret file on .100 dated 2026-04-13 11:48; v0.9.844 deployed to .109 at 19:38+; first-time persist-if-present rollout rotated the secret once, no mechanism to announce.
- Fleet-doctor already surfaces the failure with a correct hint, but nothing escalates or codifies; two agents rediscovered the same analysis independently.
- R1 (memory drift) and R2 (bootstrap chicken-and-egg on step 4 of D) contributed by ring20-dashboard peer review, both accepted into the design.

**Decomposition:** 5 build tasks T-1052 (learning auto-register) → T-1053 (concern auto-register after N days) → T-1054 (`fleet reauth` command, Tier-1) → T-1055 (`fleet reauth --bootstrap-from`, Tier-2 with explicit anchor) → T-1056 (CLAUDE.md rotation protocol docs).

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

**Rationale**: Option D: persist-by-default + auto-registered learning + fleet reauth heal command. Peer reviewed.

**Date**: 2026-04-14T07:58:16Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-14T07:10:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-14T07:58:16Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Option D: persist-by-default + auto-registered learning + fleet reauth heal command. Peer reviewed.
