---
id: T-2291
name: "Permanent cross-agent comms fix — identity + delivery"
description: >
  Inception: Permanent cross-agent comms fix — identity + delivery

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-27T08:50:06Z
last_update: 2026-06-27T16:40:17Z
date_finished: 2026-06-27T16:40:17Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
---

# T-2291: Permanent cross-agent comms fix — identity + delivery

## Problem Statement

Cross-agent communication repeatedly fails. Triggering incident: a `card-redirect`
agent's handoff to the manager was lost. Two **compounding structural root causes**:

1. **Shared host fingerprint (identity layer).** All agents co-resident on host
   `.107` authenticate with the host key → share ONE identity `d1993c2c64…`.
   Manager↔.107-agent DMs collapse onto a single `dm:` topic, discriminated only
   by a human-typed `[manager -> X]` body tag. No per-agent addressability.
   T-1693 (per-agent keys) is the designed-but-unshipped fix.
2. **No inter-hub federation — BY DESIGN (delivery layer).** A post on hub A
   (`.122`) never reaches a reader on hub B (`.107`); there is no mirror that
   "lags" — it never syncs (G-060 / T-2229 / PL-176). Senders don't know which
   hub a peer reads; "sent" ≠ "delivered." The handoff sat at offset 48 on `.122`
   while the manager read `.107`.

Systemic, not a one-off: ring20 T-1259/T-1264/T-1296 (re-filed 3×), G-155
(false-green probe), G-156 (no peer registry). Full RCA + 5 directive-scored
remediation variants in `docs/reports/T-2291-cross-agent-comms-inception.md`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: Is controlled cross-hub relay in scope, or a non-goal?**
  confidence: 1
  disposition: deferred
  rationale: G-060 is a deliberate design choice (T-2229); any federation variant must be opt-in + explicit + loud — passive replication is a known invalid premise. Resolve via A3/A5 findings.
- **IW-2: Identity-fix vs delivery-fix sequencing — which unblocks more, can one ship without the other?**
  confidence: 1
  disposition: deferred
  rationale: Both root causes compound; need A2+A3 RCA to decide whether per-agent identity (V1) or delivery addressing/confirm (V2/V3) is the higher-leverage first move.
- **IW-3: Is single-hub convergence (V5) acceptable given its portability/SPOF cost?**
  confidence: 1
  disposition: deferred
  rationale: Eliminates cross-hub delivery entirely but trades against Portability directive + single point of failure. Score in the directive matrix (A5) before deciding.

## Exploration Plan

5 termlink research agents (parallel where independent), findings written to disk
(`docs/reports/T-2291-*` + bus), concise summaries returned:
- **A1** Cross-agent issue harvester — reach LIVE peers via termlink, build failure catalogue.
- **A2** Identity-layer RCA — shared fingerprint, T-1693 design, T-1427 verified identity, blast radius.
- **A3** Delivery-layer RCA — no-federation (G-060/T-2229), peer-registry gap (G-156), delivery-confirm (T-2286), false-green probe (G-155).
- **A4** Substrate-capability scout — inventory primitives (kv, cv_index, claim, ack-retry, find-idle, presence) → map to root causes.
- **A5** Prior-art + directive-scoring lead — collect prior filings/decisions, finalize rubric, pre-score where evidence exists.

Then synthesize RCA (T5–T6), develop + score 5 variants (T7), present to human (T8).

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** RCA of the two root causes; 5 structural remediation variants developed
and scored against the 4 Constitutional Directives + cost; a recommendation
(composite likely) presented for human go/no-go.
**OUT:** Any build artifact (no production code before GO — inception discipline).
Variant selection + GO decision (human sovereignty). Cross-project (AEF) writes
beyond relay unless separately authorized.

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

**Recommendation:** GO — composite **V3 + V2 + V1**, sequenced V3 → V2 → V1.

**Rationale:**

5 research agents found the comms failures decompose into THREE orthogonal axes,
and — critically — **the fix mechanism for each is already SHIPPED**; the gap is
defaults + observability, not greenfield code:
- **RC3 delivery (V3, cost S, score 18 — strongest):** T-2286 `--await-ack` +
  T-2049 dedupe + T-2051 queue already do exactly-once, loud-on-failure delivery
  confirmation. Work = flip it to DEFAULT for `/agent-handoff`,`/reply`,
  `agent-send.sh`. Turns silent "sent-but-lost" into a loud timeout fleet-wide.
- **RC2 routing (V2, cost S→M, score 15):** agent-presence + cv_index already
  carry a stable-`agent_id` roster (~80% of a peer registry). Work = add
  `addr:port` to the heartbeat + a fleet-rollup reader so senders resolve the
  RIGHT hub. (NOT `kv` — per-session, wrong scope.)
- **RC1 identity (V1, cost S→M, score 15):** per-agent keys SHIPPED (T-1693/
  G-056) but never set by default → every co-resident agent collapses to one
  fingerprint. Work = make per-agent identity the default in register/
  be-reachable/heartbeat.

V3 makes failure loud, V2 routes it to the right hub, V1 de-collides identities —
they compose; none solves comms alone. **Reject V5** (SPOF, violates
Portability). **Hold V4** (explicit relay) unless a hard cross-hub need survives
V2. On GO, file 3 separate build tasks (one per leg). Variant/composite selection
+ GO is the human's (sovereignty).

**Evidence:**

- Full RCA + directive-scored matrix: `docs/reports/T-2291-cross-agent-comms-inception.md` §4–§6
- Agent findings on disk: `.context/working/T-2291-A{1..5}.md`
- V1 identity model: `crates/termlink-session/src/agent_identity.rs:174`, `registration.rs:48`; T-1693/G-056 shipped 2026-05-19
- V2 substrate: agent-presence heartbeat + cv_index (`scripts/listener-heartbeat.sh`, `channel cv-keys agent-presence`)
- V3 mechanism: T-2286 `channel post --await-ack` (work-completed 2026-06-25) + T-2049 + T-2051
- No-federation by design: G-060 / T-1791 / T-1793 / T-2229; recurrence ring20 T-1259/T-1264/T-1296 (×3), G-155 (false-green probe), G-156 (no registry), G-063 (write-only pickup sink)
- Live fleet snapshot 2026-06-27: 0/8 listeners LIVE, all `.107` share fp `d1993c2c3ec44c94` (collision reproduced)

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

**Rationale**: 5 research agents found the comms failures decompose into THREE orthogonal axes,
and — critically — **the fix mechanism for each is already SHIPPED**; the gap is
defaults + observability, not greenfield code:

**Date**: 2026-06-27T16:40:17Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-27T08:50:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-27T16:40:17Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** 5 research agents found the comms failures decompose into THREE orthogonal axes,
and — critically — **the fix mechanism for each is already SHIPPED**; the gap is
defaults + observability, not greenfield code:

### 2026-06-27T16:40:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
