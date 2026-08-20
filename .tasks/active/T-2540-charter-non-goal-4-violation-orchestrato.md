---
id: T-2540
name: "Charter non-goal #4 violation: orchestrator.route policy-in-substrate (subtract
  or grandfather?)"
description: >
  Purpose-review (T-2468) non-goal-adherence finding, verified-in-code. The hub carries
  orchestrator.route backed by ~1981 lines (route_cache.rs/circuit_breaker.rs/bypass.rs)
  of adaptive routing POLICY — the exact thing charter non-goal #4 reserves for the
  AEF layer. NOT deprecated; advertised in hub.capabilities. Consumer-check: ZERO
  first-party callers (advertised-but-uncalled); only refs are 2024 pre-charter design
  reports (T-233/237/238 'smart routing' lineage). DECISION (human sovereignty): (A)
  SUBTRACT — remove the RPC + 3 modules + capability advert, relocate needed routing
  to AEF/client; (B) GRANDFATHER — amend charter for a sanctioned exception; (C) intermediate.
  GATE: fleet-wide external-consumer check (any peer/AEF raw orchestrator.route RPC?)
  BEFORE removal. Largest subtract candidate the campaign has surfaced (~2k lines).
  Owner human: consequential (live advertised capability), sovereignty-level, needs
  external-consumer verification.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-08T16:20:13Z
last_update: '2026-08-20T15:21:21Z'
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
  - ts: '2026-08-20T15:20:37Z'
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
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 4
      effort: 7
    rationale: blast_radius=3 (no-signal); tier=4 (no-signal); effort=7 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2540: Charter non-goal #4 violation: orchestrator.route policy-in-substrate (subtract or grandfather?)

## Problem Statement

TermLink's charter (docs/CHARTER.md) names a hard NON-GOAL #4: *the substrate is
mechanism, not policy — workflow/orchestration policy belongs to the AEF layer,
not the hub.* Yet the hub ships **`orchestrator.route`**: a JSON-RPC method backed
by ~1981 lines of *adaptive routing policy* (specialist-preference by task_type, a
confidence-scored learning cache, a circuit breaker, and Tier-3 bypass promotion).
This is precisely the policy that non-goal #4 reserves for AEF. It is NOT
deprecated and IS advertised in the hub's capability set, yet has **zero
first-party callers**. The decision is a product-identity one the human owns:
does this capability stay (grandfathered with a charter amendment) or leave
(subtracted to restore the charter)? Now, because the T-2468 purpose review's
mandate is "subtract-and-deepen" and this is the single largest off-charter
subtract candidate the campaign has surfaced (~2k lines). See the full brief:
`docs/reports/T-2540-orchestrator-route-decision-brief.md`.

## Assumptions

- A-1: `orchestrator.route` implements adaptive *policy* (not a mechanism-only
  primitive). — to confirm by reading the three modules.
- A-2: There are zero first-party callers in this repo (CLI / MCP / scripts).
- A-3: The only references are pre-charter (2024) design reports, not live use.
- A-4 (UNVERIFIABLE from this repo): no *external* peer/AEF process issues the
  raw `orchestrator.route` RPC across the fleet. This is the GATE and requires a
  cross-project check (blocked by the T-559 project-boundary from this session).

## Open Questions

- **IW-1: Are there any EXTERNAL (non-first-party) consumers of the raw
  `orchestrator.route` RPC across the fleet or the AEF layer?**
  confidence: 1
  disposition: deferred
  rationale: In-repo first-party callers = zero (verified). External callers
  cannot be confirmed from /opt/termlink (T-559 boundary blocks cross-project
  grep of /opt/999-AEF and peer repos). This is the removal GATE — a human or a
  cross-project session must clear it before any subtract lands.

- **IW-2: If subtracted, does any NEEDED routing behavior have to be relocated to
  AEF/client, or is the capability purely dead weight?**
  confidence: 2
  disposition: deferred
  rationale: Zero first-party callers suggests dead weight, but a relocation
  analysis (what would an AEF orchestrator that wanted this policy have to
  re-implement?) is part of the GO build scope, not this inception.

- **IW-3: Subtract vs grandfather — does the charter get amended for a sanctioned
  exception, or does the capability leave the substrate?**
  confidence: 2
  disposition: deferred
  rationale: This is the human sovereignty decision. Agent recommendation = GO to
  subtract (restore charter), because the policy is uncalled and non-goal #4 is a
  deliberate architectural line. But amending the charter is equally valid if the
  human judges the routing policy strategically worth keeping in-substrate.

## Exploration Plan

1. Verify A-1: read route_cache.rs / circuit_breaker.rs / bypass.rs — confirm
   adaptive-policy nature + line counts. (DONE — see brief.)
2. Verify A-2/A-3: grep the repo for `orchestrator.route` / `orchestrator_route`
   callers across CLI, MCP, scripts, docs. (DONE — see brief.)
3. Clear IW-1 (the GATE): cross-project consumer check — cannot be done from this
   session (T-559). Hand to human / a cross-project session.
4. Human decides IW-3 (subtract / grandfather / intermediate).

## Technical Constraints

- **T-559 project-boundary:** this session cannot grep /opt/999-AEF or peer repos
  for external `orchestrator.route` callers — the GATE check (IW-1) is
  structurally cross-project and human-gated.
- **Consequential:** `orchestrator.route` is a LIVE advertised capability;
  removing it is a breaking change for any (unknown) external caller — hence the
  gate-before-removal discipline.

## Scope Fence

**IN scope (this inception):** confirm the violation in code, confirm zero
first-party callers, frame the subtract-vs-grandfather decision with evidence.
**OUT of scope:** the actual removal (a GO build task), the cross-project
external-consumer check (IW-1 gate, human/cross-project), any charter amendment
(human-blessed sovereignty edit).

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

Full evidence + option analysis: `docs/reports/T-2540-orchestrator-route-decision-brief.md`.

**GO (subtract) if:**
- Charter non-goal #4 ("substrate stays mechanism, not policy") is to remain a
  load-bearing line, AND
- The IW-1 gate clears — the fleet-wide external-consumer check finds NO peer/AEF
  process issuing the raw `orchestrator.route` RPC (first-party callers already
  verified zero). Then: delete the 3 methods + 3 modules (~1981 lines), keep
  `session discover` + `channel claim` as the sanctioned mechanism.

**NO-GO (grandfather) if:**
- The human judges the built routing policy strategically worth keeping
  in-substrate → amend `docs/CHARTER.md` for a sanctioned exception AND fund the
  T-247 adversarial-surface paydown (arbitrary-string bypass promotion,
  concurrency).

**INTERMEDIATE (deprecate-then-remove) if:**
- IW-1 cannot be cleared quickly → mark the 3 methods deprecated (stop
  advertising, warn on call, per the T-1166 `remote_inbox_*` pattern), soak one
  release, then remove.

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

**Rationale:** Verified in code: orchestrator.route (router.rs:78/1186, advertised in hub.capabilities, NOT deprecated) + ~1981 lines (route_cache 824 + circuit_breaker 488 + bypass 669) implement adaptive routing POLICY (task_type specialist preference, confidence-scored learning cache, circuit breaker, Tier-3 bypass promotion) — a direct charter non-goal #4 violation ('substrate stays mechanism, not policy'). Consumer-check found ZERO first-party callers (no CLI/MCP/script); only 2024 pre-charter design-report refs (T-233/237/238). Recommend GO to SUBTRACT (relocate to AEF), gated on a fleet-wide external-consumer check first. Human owns the subtract-vs-grandfather product decision. **Full decision brief (options A/B/C with cost/risk/reversibility, the IW-1 gate procedure, and the GO build-task scope): `docs/reports/T-2540-orchestrator-route-decision-brief.md`.** Update (2026-08-08): re-verified in code — line counts exact (824+488+669=1981), advertised at router.rs:1023-1027, NOT deprecated; the subtract surface is THREE methods (orchestrator.route + orchestrator.bypass_status + orchestrator.bypass_invalidate); the pre-charter refs are the T-233/T-239/T-240/T-247 design lineage (not live callers); T-247-scenarios-adversarial.md documents un-remediated bypass-promotion security debt (an argument *for* subtract).

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

### 2026-08-08T17:39:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
