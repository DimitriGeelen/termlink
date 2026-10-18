---
id: T-3423
name: "arc-011: per-peer least-privilege hub credentials across operators (832 request
  T-1107)"
description: >
  832-Workflow-designer request (their T-1107, operator-approved, conversation peer-credentials-t1107):
  a hub keeps an allow-list of peer ed25519 public keys; a peer authenticates by signing
  a hub challenge; each entry has a scope (message only: post to inbox:* or named
  topics), an expiry, and is revoked unilaterally by deleting it; on top, termlink
  peer request/approve puts a signed request in a PENDING queue that the other operator
  approves after a fingerprint check over a second channel. No hub secret crosses.
  Survey and options A-D: /opt/832-Workflow-designer/docs/reports/T-1107-cross-hub-credential-model.md.
  Operator ruling 2026-10-12 (T-3422): A, taken into arc-011 for its cross-hub/grant
  phases; builds on the OQ-2 ruling E (receiver holds unknown keys, operator admits).
  Current state of the four questions: docs/reports/T-3422-832-peer-credentials-answer.md.

status: captured
workflow_type: inception
owner: agent
horizon: later
tags: [arc:arc-011]
components: []
related_tasks: []
created: 2026-10-12T13:56:33Z
last_update: '2026-10-18T22:32:57Z'
date_finished:
revisit_at: 2026-11-12
revisit_evidence_needed: "arc-011 PH-1 exit (X6) recorded and PH-2 planning started,
  or 832 reports their stopgap secret was exposed"
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.7                    # operator 2026-10-18 (option A): decides how other operators' agents are admitted to our hubs at all; feeds the identity/revocation work PH-2 is doing now (P10, OQ-h); 832 waits on it. Parked limits WHEN, not the value. — float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-18T22:32:57Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 4
      D4: 4
      F-RECALL: 4
      F-ORCH: 4
    rationale: D1=4 (no-signal); D2=4 (no-signal); D3=4 (no-signal); D4=4 
      (no-signal); F-RECALL=4 (no-signal); F-ORCH=4 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3423: arc-011: per-peer least-privilege hub credentials across operators (832 request T-1107)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

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

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
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
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** DEFER

**Rationale:** Operator ruled it into arc-011's later cross-hub/grant phases (T-3422, 2026-10-12): it is the hub-side twin of the OQ-2 E receiver admission just built, and grants need their lock first (NK). Revisit when PH-2 planning starts.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
