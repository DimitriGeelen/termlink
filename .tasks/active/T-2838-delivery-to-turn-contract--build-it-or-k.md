---
id: T-2838
name: "Delivery-to-turn contract — build it or keep nudging (G-083)"
description: >
  Inception: Delivery-to-turn contract — build it or keep nudging (G-083)

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [comms, substrate, g-083, delivery]
components: []
related_tasks: [T-2468, T-2482, T-2485, T-2388, T-2394, T-2395, T-2413, T-2414, T-2416, T-2479, T-1800, T-1249, T-2700]
created: 2026-08-24T22:35:59Z
last_update: 2026-08-24T22:39:19Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 5            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.9                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2838: Delivery-to-turn contract — build it or keep nudging (G-083)

## Problem Statement

Does TermLink build a delivery-to-turn contract — delivery acknowledged from inside the
recipient's own turn loop, plus a typed result-manifest envelope — or does interactive
agent<->agent communication stay dependent on manual nudging?

Stated goal (operator, 2026-08-24): TermLink as an interactive communication medium, primarily
agent<->agent, also operator<->agent; topologies 1:1, N:N, N:operator, 1:operator; plus artifact
passing by reference — an orchestrator dispatches an assignment (with an asserted agent profile),
the agent works, writes files, and reports back THAT those files were written, so a downstream
agent consumes conclusions by reference rather than a binary blob.

Operator assessment: "Partially certainly, but that interactive communication is still flaky at best."

Full analysis: docs/reports/T-2838-delivery-to-turn-contract.md (C-001 research artifact).

## Assumptions

- A-1: A launched agent session can acknowledge delivery from inside its own turn loop without modifying Claude Code itself.
- A-2: For sessions we do not control, PTY-inject plus mandatory read-cursor confirmation converts silent failure into loud diagnosable failure.
- A-3: Liveness sourced from the consuming loop rather than a sidecar heartbeat eliminates the LIVE-not-listening class by construction.
- A-4: A typed result manifest (paths, hashes, summary, status) removes the need for byte transfer between agents sharing a filesystem.
- A-5: termlink-bus requires no change; the entire delivery gap sits above the log engine.
- A-6: Consumption confirmation can be built on existing receipt/read-cursor primitives without a wire-protocol version bump. **[CONFIRMED 2026-08-25 by S4.]**

## Open Questions

- **IW-1: Can a launched agent session ack delivery from inside its own turn loop without modifying Claude Code?**
  confidence: 1
  disposition: pending
  rationale: Spike S2. The MCP server runs in-process with the session so an MCP-side ack is plausible, but nothing proves the ack coincides with an actual *turn* rather than a tool call.

- **IW-2: What is the minimum viable turn-envelope + result-manifest schema?**
  confidence: 1
  disposition: pending
  rationale: Spike S3. T-1249 encodes the right instinct; no assignment/manifest envelope type exists anywhere.

- **IW-3: Does PTY-inject stay a supported delivery mode or narrow to operator-only?**
  confidence: 2
  disposition: pending
  rationale: Decision, not spike. It is the only route into a session we did not launch, so it cannot be deleted; the question is whether it may ever report success without confirmation.

- **IW-4: Does the contract need a wire-protocol version bump, and is that safe today?**
  confidence: 5
  disposition: answered
  answer: >
    No bump needed. S4 (2026-08-25) confirmed A-6. `receipt` is already a first-class wire
    msg_type; `Envelope.artifact_ref` already exists and is already inside canonical signed
    bytes (channel.rs:944); receipts are published to the topic and read back by
    compute_ack_status, so consumption is third-party observable today. The contract is
    therefore NOT sequenced behind T-2700. Evidence in
    docs/reports/T-2838-delivery-to-turn-contract.md section "S4 — Primitive audit".
  rationale: Spike S4. T-2699 found PROTOCOL_VERSION_TOO_OLD has zero emission sites; T-2700 (wiring it) is captured, owner human; fleet hubs run ~1000 commits stale (T-2377). A wire change today has no compatibility gate.

- **IW-5: Does the contract live hub-side (enforced) or client-side (convention)?**
  confidence: 1
  disposition: pending
  rationale: Spikes S2+S4. Convention is what failed for the heartbeat.

## Exploration Plan

- **S1 — Reproduce deterministically (2h).** Drive a rung-but-not-consumed delivery on purpose against a busy/manual recipient; capture offset, receipt state, and what `/canaries` + `comms-selftest.sh` report. Deliverable: reproduction recipe in the research artifact.
- **S2 — Native consumer spike (4h).** Minimal agent loop that blocks on the substrate and acks from inside its own turn; measure whether the ack can coincide with a real turn. Tests A-1, A-3; disposes IW-1.
- **S3 — Envelope schema draft (2h).** Assignment envelope + result manifest, round-tripped against the existing `artifact.put`/`get` path. Tests A-4; disposes IW-2.
- **S4 — Primitive audit (1h).** Do today's receipt/read-cursor primitives support bounded consumption confirmation without a wire change? Tests A-6; disposes IW-4.

## Technical Constraints

- Claude Code's REPL is not modifiable — the contract must work with the session as-is.
- PTY inject is the only route into a session we did not launch; it cannot be removed, only made honest.
- Directive #4 Portability: no Linux-only mechanism. `scripts/check-platform-lock.sh` (T-2693) fires on `/proc`, `setsid`, `ss`, `systemctl` and requires an allowlist entry stating how the non-Linux path degrades; macOS uses `pf` and differs on PTY/tmux details.
- Charter non-goal #1 (hub-mediated star): no peer-to-peer delivery. `no_federation_tripwire.rs` / `no_spoke_mesh_tripwire.rs` fail the build on violation.
- Charter non-goal #2 (not a system of record): manifests reference artifacts, never archive them.
- Protocol version negotiation is unwired (T-2699/T-2700) — a wire change today has no compatibility gate.
- Trust model is a cooperating fleet with TOFU-pinned hubs and a shared HMAC secret (charter non-goal #5).

## Scope Fence

**IN** — the delivery/consumption contract; what a delivery may report when unconfirmed; liveness from the consumer rather than a sidecar heartbeat; assignment-envelope and result-manifest schema; a go/no-go with a bounded build decomposition on GO.

**OUT** — the CLI/MCP adapter refactor (122,860 LOC over a 5,240-LOC engine); the MCP surface redesign (262 flat tools, ~39k tokens flat-loaded); write-set collision detection (P6, still DEFER'd); any build artifact.

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

**GO if:** S2 shows an agent can ack from inside its own turn loop (A-1), giving a by-construction fix rather than another detector; the contract is expressible without an ungated wire-protocol break (IW-4), or the break is sequenced behind T-2700; a result-manifest envelope round-trips on the existing artifact path (A-4); and the build decomposes into separately-shippable tasks.

**NO-GO if:** reliable consumption proves impossible without modifying Claude Code — in which case the honest outcome is to make unconfirmed delivery *loudly* unconfirmed and stop claiming interactivity the substrate cannot deliver; or the contract requires peer-to-peer delivery or breaches a charter non-goal; or scope cannot be bounded below the adapter refactor.

## Verification

test -f docs/reports/T-2838-delivery-to-turn-contract.md
grep -q "G-083" docs/reports/T-2838-delivery-to-turn-contract.md
grep -q "PL-253" docs/reports/T-2838-delivery-to-turn-contract.md

## Recommendation

**Recommendation:** GO

**Rationale:**

GO. G-083 (severity high, status watching) has been diagnosed since 2026-07-10 with the root cause named — PTY keystroke injection lands unsubmitted in a busy/manual session and is discarded on --continue, while listener-heartbeat.sh proves a background script alive rather than the session listening, so LIVE != listening by construction. The concern records the fix as "not yet built — pending operator". Everything built since is DETECTION (woken-but-silent canary, wake-confirm.sh, woken-silent-triage.sh, comms-selftest.sh, diagnose-unconsumed.sh, plus G-085 to un-stick the canary), never a delivery contract. Operator reports the symptom persists ("interactive communication is still flaky at best"). Detection is saturated; the mechanism is the remaining gap, and it blocks the stated product goal of interactive agent<->agent and operator<->agent exchange.

**Evidence:**

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

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

**Rationale**: Recommendation: GO

Rationale:

GO. G-083 (severity high, status watching) has been diagnosed since 2026-07-10 with the root cause named — PTY keystroke injection lands unsubmitted in a busy/manual session and is discarded on --continue, while listener-heartbeat.sh proves a background script alive rather than the session listening, so LIVE != listening by construction. The concern records the fix as "not yet built — pending operator". Everything built since is DETECTION (woken-but-silent canary, wake-confirm.sh, woken-silent-triage.sh, comms-selftest.sh, diagnose-unconsumed.sh, plus G-085 to un-stick the canary), never a delivery contract. Operator reports the symptom persists ("interactive communication is still flaky at best"). Detection is saturated; the mechanism is the remaining gap, and it blocks the stated product goal of interactive agent<->agent and operator<->agent exchange.

Evidence:

**Date**: 2026-08-25T09:18:11Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-08-24T22:38:46Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-25T09:18:11Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale:

GO. G-083 (severity high, status watching) has been diagnosed since 2026-07-10 with the root cause named — PTY keystroke injection lands unsubmitted in a busy/manual session and is discarded on --continue, while listener-heartbeat.sh proves a background script alive rather than the session listening, so LIVE != listening by construction. The concern records the fix as "not yet built — pending operator". Everything built since is DETECTION (woken-but-silent canary, wake-confirm.sh, woken-silent-triage.sh, comms-selftest.sh, diagnose-unconsumed.sh, plus G-085 to un-stick the canary), never a delivery contract. Operator reports the symptom persists ("interactive communication is still flaky at best"). Detection is saturated; the mechanism is the remaining gap, and it blocks the stated product goal of interactive agent<->agent and operator<->agent exchange.

Evidence:
