---
id: T-3476
name: "Deterministic task-assignment sidecar: one routine allocates task IDs and assigns
  work units by availability, with a request queue"
description: >
  Inception: Deterministic task-assignment sidecar: one routine allocates task IDs
  and assigns work units by availability, with a request queue

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-10-19T13:36:28Z
last_update: 2026-10-19T13:38:58Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-10-19T13:38:58Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3476: Deterministic task-assignment sidecar: one routine allocates task IDs and assigns work units by availability, with a request queue

## Problem Statement

Handing out task IDs and deciding which free worker gets which work unit are mechanical, yet
agents do both by hand today: IDs are "max in this checkout + 1" (collided 12x across branches,
T-2800) and no production orchestrator assigns work through the shipped claim primitives.
Parallel workers (T-2026 GO, T-3469) need both done centrally first. Operator, 2026-10-19:
make it a deterministic routine, ideally a sidecar with a request queue, called by the main
agent or the framework. Research: docs/reports/T-3476-task-assignment-sidecar.md.

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

- **IW-1: One service for both task-ID allocation and work assignment, or two?**
  confidence: 1
  disposition:
  rationale:
- **IW-2: Where does it run: in the hub, as an AEF sidecar API, or as its own daemon; and with non-federated hubs (G-060), who is the single assigner across hosts?**
  confidence: 0
  disposition:
  rationale:
- **IW-3: ID allocation scheme: hub counter per project, reserved ranges per worker, or claim-scoped namespace; and how does it reach vendored AEF (upstream, G-062)?**
  confidence: 1
  disposition:
  rationale:
- **IW-4: Assignment policy and queue: FIFO plus capability/availability matching; how priority (horizon, BVP) enters; durable and operator-visible; loud refusal when nothing fits?**
  confidence: 1
  disposition:
  rationale:
- **IW-5: Who calls it (orchestrator agent, fw task create, Watchtower, 055 cockpit), and what is the smallest real consumer?**
  confidence: 0
  disposition:
  rationale:

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

**Rationale:**

Shaping in progress. Five external reviewers on T-3469 (2026-10-19) agreed parallel clones re-create the 12-ID task collision (T-2800) unless allocation is central; operator proposes a deterministic sidecar routine that allocates IDs and assigns work by availability with a request queue. Evidence still to gather: existing pieces (claims, find-idle, T-2838 assignment schema, AEF T-3310 election, filed-upstream allocation fix) and where the routine should live.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-19T13:38:58Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
