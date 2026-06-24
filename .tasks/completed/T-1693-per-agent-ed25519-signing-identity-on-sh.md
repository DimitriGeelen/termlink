---
id: T-1693
name: "Per-agent ed25519 signing identity on shared hosts — provisioning + session-time identity selection (cohort-agent ask B)"
description: >
  Cohort-agent ask (B). T-1427 enforcement revealed all co-resident agents on .107 share one envelope-signing key (d1993c2c). Per-agent attribution is structurally impossible at envelope layer today. T-1159 already shipped per-session ed25519 keypair infra; this task designs the deployment+wiring model. Cohort recommends Shape 1 (agent-managed key files, per-project secrets dir, passed via --identity-key at termlink register). Independent from T-1448 (which was about disambiguation primitives); this one is about identity provisioning.

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-18T09:19:51Z
last_update: 2026-05-18T21:02:26Z
date_finished: 2026-05-18T21:02:26Z
---

# T-1693: Per-agent ed25519 signing identity on shared hosts — provisioning + session-time identity selection (cohort-agent ask B)

## Problem Statement

T-1427 enforcement (envelope sig must match host-derived identity)
revealed that all co-resident agents on .107 (penelope, cohort-agent,
framework-agent, termlink-agent, this Claude session) sign with one
host-wide ed25519 key — `d1993c2c`. The fingerprint `9219671e` we'd
been treating as "Pen's" for 6+ weeks lives in payload only, not in
the envelope. Co-resident attribution at envelope layer is therefore
*structurally unanswerable* today.

This affects: audit trails on cross-agent contracts, multi-tenant
trust (one host = one trust unit), defence-in-depth against per-agent
compromise, and clean separation as more agents land on shared hosts.

T-1159 (work-completed 2026-04-20) already added per-session ed25519
keyring infrastructure to `termlink-session`. The gap is the
*deployment/wiring model* — how agents provision keys, how those keys
get bound to a session at registration time, and how rotation works.

## Assumptions

- A1: T-1159's per-session keyring is reusable per-agent (not just per-host
  default). Verify by reading the session-init path.
- A2: `termlink register` (or equivalent session-start verb) is the
  natural insertion point for an `--identity-key` flag / env var.
- A3: No protocol change required — envelope signing already supports
  arbitrary ed25519 keys; the gap is at the session-instantiation surface.
- A4: TOFU + KnownHubStore handles multiple identities cleanly when
  different sessions present different pubkeys.

## Exploration Plan

1. Read T-1159 session-init code path to confirm A1+A2 (~15 min).
2. Audit `termlink register` and related session verbs for the right
   identity insertion point (~15 min).
3. Decide Shape 1 vs 2 vs 3 (cohort recommends Shape 1, agent-managed
   key files in per-project secrets dirs).
4. Sketch rotation/revocation story — minimum viable per cohort's
   suggestion ("regenerate, restart, old envelopes valid via TOFU
   history") (~10 min).
5. Identify split into build tasks (provisioning + register-flag +
   docs + rotation tooling) (~10 min).

## Technical Constraints

- ed25519 dependency already in tree via T-1159.
- Backward compat: existing host-shared default must keep working;
  per-agent keys are opt-in.
- TOFU pin handling: a session presenting a new identity must not
  break peers' existing pins for that host.
- Secrets handling: project-local key file at mode 0600, mirroring
  the existing `instance/secrets/` convention.

## Scope Fence

IN: Design of provisioning convention + session-time identity binding.
IN: Wiring `--identity-key` (or env var) into the session-init path.
IN: Rotation story — minimum viable.
IN: Documentation of the per-agent identity pattern.
OUT: Hub-managed central keystore (rejected Shape 2 by cohort recommendation).
OUT: Operator-derived deterministic keys (rejected Shape 3 — master-key compromise).
OUT: Co-resident disambiguation primitive (T-1448 — separate concern, payload-layer).
OUT: Protocol version change (none needed).
OUT: Multi-key TOFU policy redesign (lives in a follow-up if A4 disproven).

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
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

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

**Recommendation:** GO (after design phase) — but operator-prioritized AFTER T-1692
per cohort-agent's letter ("(A) first, smaller and immediate consumer impact;
(B) second, independent and larger").

**Rationale:** Real structural gap with named implications (audit, multi-tenant
trust, defence-in-depth). T-1159 already shipped the foundation; this task is
the wiring model, not new crypto. Cohort recommends Shape 1 (agent-managed key
files, no new hub infra) which preserves agent sovereignty and matches the
existing project-local secrets convention. Rotation story is "regenerate +
restart + TOFU history covers old envelopes" — minimum viable, acceptable for
current threat model.

**Evidence:**
- T-1427 error -32014 — empirical proof that 9219671e was never an envelope identity
- 19 historical envelopes labeled "Pen" all signed by d1993c2c (host)
- T-1159 (completed) — keyring infrastructure exists; wiring is the gap
- Cohort-agent's letter — explicit ask with full steel-man analysis
- New concern G-XXX (this register) — "co-resident envelope identity undifferentiable"

## Steel-man Design Shapes (from cohort-agent's letter)

**Shape 1 — Per-agent key files, agent-managed (RECOMMENDED).** Each agent
owns a keypair under its own project's secrets dir, mode 0600. Agent passes
path at session registration via `--identity-key`. Hub does not centrally
manage keys.
- Antifragility ✓ (no central store to lose) Reliability ✓
- Usability ✓ (mirrors existing pattern, e.g. `instance/secrets/pen_outbound.key`)
- Portability ✓ (no hub-side dependency)
- Risk: operators must remember to back up identity files. Mitigation:
  surface in `fw doctor` + doc convention.

**Shape 2 — Hub-managed keystore.** Hub stores and serves keys to registered
sessions. Centralised; easier rotation; single backup point.
- Antifragility ◌ (central failure mode) Usability ◌ (new infra)
- Portability ✗ (couples agents to hub keystore)

**Shape 3 — Operator-derived deterministic keys.** Each agent's key derived
from `(operator-master-key, agent-name)` via HKDF.
- Antifragility ✗ (master-key compromise = all agents compromised)
- Usability ✓ Portability ◌

## Relationship to existing tasks

- **T-1159** — predecessor (keyring exists). This task is the wiring model.
- **T-1448** — sibling, owner=human, in-flight. T-1448 explores co-resident
  *disambiguation* primitives (payload-layer); T-1693 provisions per-agent
  *identity* (envelope-layer). T-1693 enables T-1448's strongest variant
  but is independent.
- **T-1427** — the enforcement that surfaced this gap. No change needed.
- **T-1457** — register identity on .141; existing host-shared model.
  Compatibility flag for the per-agent-key path.

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

**Rationale**: Recommendation: GO (after design phase) — but operator-prioritized AFTER T-1692
per cohort-agent's letter ("(A) first, smaller and immediate consumer impact;
(B) second, independent and larger").

Rationale: Real structural gap with named implications (audit, multi-tenant
trust, defence-in-depth). T-1159 already shipped the foundation; this task is
the wiring model, not new crypto. Cohort recommends Shape 1 (agent-managed key
files, no new hub infra) which preserves agent sovereignty and matches the
existing project-local secrets convention. Rotation story is "regenerate +
restart + TOFU history covers old envelopes" — minimum viable, acceptable for
current threat model.

Evidence:
- T-1427 error -32014 — empirical proof that 9219671e was never an envelope identity
- 19 historical envelopes labeled "Pen" all signed by d1993c2c (host)
- T-1159 (completed) — keyring infrastructure exists; wiring is the gap
- Cohort-agent's letter — explicit ask with full steel-man analysis
- New concern G-XXX (this register) — "co-resident envelope identity undifferentiable"

**Date**: 2026-05-18T21:02:26Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-18T21:02:26Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO (after design phase) — but operator-prioritized AFTER T-1692
per cohort-agent's letter ("(A) first, smaller and immediate consumer impact;
(B) second, independent and larger").

Rationale: Real structural gap with named implications (audit, multi-tenant
trust, defence-in-depth). T-1159 already shipped the foundation; this task is
the wiring model, not new crypto. Cohort recommends Shape 1 (agent-managed key
files, no new hub infra) which preserves agent sovereignty and matches the
existing project-local secrets convention. Rotation story is "regenerate +
restart + TOFU history covers old envelopes" — minimum viable, acceptable for
current threat model.

Evidence:
- T-1427 error -32014 — empirical proof that 9219671e was never an envelope identity
- 19 historical envelopes labeled "Pen" all signed by d1993c2c (host)
- T-1159 (completed) — keyring infrastructure exists; wiring is the gap
- Cohort-agent's letter — explicit ask with full steel-man analysis
- New concern G-XXX (this register) — "co-resident envelope identity undifferentiable"

### 2026-05-18T21:02:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** Inception decision in progress

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f6e1cb5e
- **Timestamp:** 2026-05-18T21:02:27Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-18T21:02:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
