---
id: T-2540
name: "Charter non-goal #4 violation: orchestrator.route policy-in-substrate (subtract or grandfather?)"
description: >
  Purpose-review (T-2468) non-goal-adherence finding, verified-in-code. The hub carries orchestrator.route backed by ~1981 lines (route_cache.rs/circuit_breaker.rs/bypass.rs) of adaptive routing POLICY — the exact thing charter non-goal #4 reserves for the AEF layer. NOT deprecated; advertised in hub.capabilities. Consumer-check: ZERO first-party callers (advertised-but-uncalled); only refs are 2024 pre-charter design reports (T-233/237/238 'smart routing' lineage). DECISION (human sovereignty): (A) SUBTRACT — remove the RPC + 3 modules + capability advert, relocate needed routing to AEF/client; (B) GRANDFATHER — amend charter for a sanctioned exception; (C) intermediate. GATE: fleet-wide external-consumer check (any peer/AEF raw orchestrator.route RPC?) BEFORE removal. Largest subtract candidate the campaign has surfaced (~2k lines). Owner human: consequential (live advertised capability), sovereignty-level, needs external-consumer verification.

status: captured
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-08T16:20:13Z
last_update: 2026-08-08T16:20:13Z
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

# T-2540: Charter non-goal #4 violation: orchestrator.route policy-in-substrate (subtract or grandfather?)

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

**Recommendation:** GO

**Rationale:** Verified in code: orchestrator.route (router.rs:78/1186, advertised in hub.capabilities, NOT deprecated) + ~1981 lines (route_cache 824 + circuit_breaker 488 + bypass 669) implement adaptive routing POLICY (task_type specialist preference, confidence-scored learning cache, circuit breaker, Tier-3 bypass promotion) — a direct charter non-goal #4 violation ('substrate stays mechanism, not policy'). Consumer-check found ZERO first-party callers (no CLI/MCP/script); only 2024 pre-charter design-report refs (T-233/237/238). Recommend GO to SUBTRACT (relocate to AEF), gated on a fleet-wide external-consumer check first. Human owns the subtract-vs-grandfather product decision.

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
