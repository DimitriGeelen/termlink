---
id: T-2468
name: "Critical review of TermLink purpose-goals — identify and prioritize gaps"
description: >
  Inception: Critical review of TermLink purpose-goals — identify and prioritize gaps

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-31T10:47:59Z
last_update: 2026-07-31T10:48:22Z
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

# T-2468: Critical review of TermLink purpose-goals — identify and prioritize gaps

## Problem Statement

TermLink's stated purpose is to be a **portable, reliable, antifragile communication
and coordination substrate** enabling a fleet of AI agents (and humans) across many
machines to discover each other, exchange messages reliably, coordinate work, and
stay observable — traced to the Four Constitutional Directives (Antifragility,
Reliability, Usability, Portability).

Over ~2400 tasks the surface has grown enormous: 11 cron canaries, ~273 MCP tools
(~39k tokens/agent context cost per memory arc-005), dozens of substrate primitives
and slash-verbs, and a recurring **"shipped ≠ live"** failure class (G-069) where
capabilities are recorded closed but dark in the field for weeks. This inception
asks, ultra-critically: **where does the built system diverge from its stated
purpose, and which divergences warrant action** — separating genuine gaps (things
the purpose demands but the system lacks) from over-service (complexity that does
not trace to a directive) from adoption gaps (built-but-not-live). The output is a
prioritized gap register with per-gap GO/NO-GO recommendations for the human.

Why now: the operator has explicitly asked for a critical purpose review; the fleet
is freshly at a green steady-state (all reachable hubs doorbell-capable), which is
the right moment to ask "is what we built the right thing, and is it whole?"

## Assumptions

- A-1: The Four Constitutional Directives are the correct yardstick for "purpose."
- A-2: A capability recorded `closed=shipped` is NOT evidence it is live in the field.
- A-3: Complexity that does not trace to a directive is a cost, not a feature.

## Open Questions

- **IW-1: Purpose–reality alignment — does the built system serve its stated goals?**
  What are TermLink's canonical, written purpose/goal statements (ADR, FRAMEWORK.md,
  README, constitutional directives), and where does the implementation measurably
  under- or over-serve them?
  confidence: 1
  disposition: <answered|deferred|dissolved>
  rationale: <filled during exploration>

- **IW-2: Comms rail (the doorbell replacement) — is the round-trip actually reliable end-to-end?**
  The core value prop is guaranteed cross-agent comms. Write is guaranteed; the
  round-trip (deliver → wake → ack) is not (per project_comms_loud_contract, T-2285
  ack-with-retry gap, T-2385/86/87 remnants). What are the remaining reliability holes
  and which are load-bearing?
  confidence: 1
  disposition: <answered|deferred|dissolved>
  rationale: <filled during exploration>

- **IW-3: Coordination substrate — coherent and used, or over-built?**
  Claims / find-idle / governor / queue / cv-index and their ~273 MCP tools + slash
  verbs: is this coherent and actually exercised, or is there directive-untraceable
  bloat (arc-005: ~39k tokens/agent)? Where is the complexity/value line?
  confidence: 1
  disposition: <answered|deferred|dissolved>
  rationale: <filled during exploration>

- **IW-4: Adoption / portability / "shipped ≠ live" — is what we built actually running?**
  G-069 recurs (stale/dark binaries, 0-wakers-fleet-wide, shipped≠live). Is there a
  structural adoption/deploy gap between "merged" and "capability-live across the
  fleet," and does the portability directive (MCP/standards, no lock-in) hold?
  confidence: 1
  disposition: <answered|deferred|dissolved>
  rationale: <filled during exploration>

- **IW-5: Which gaps warrant action, at what priority, and which are agent-scoped vs need human GO?**
  Synthesis question — the gap register + per-gap recommendation is the deliverable.
  confidence: 0
  disposition: <answered|deferred|dissolved>
  rationale: <filled during synthesis>

## Exploration Plan

Parallel evidence-gathering (4 review agents, one per IW-1..IW-4), each returning a
structured finding set (stated-goal evidence + ranked gaps with severity + file:line
evidence + 1-2 recommended inceptions), written up by the orchestrator into the
research artifact `docs/reports/T-2468-termlink-purpose-review.md`. Then synthesize
IW-5: a prioritized gap register. Time-box: one session for review + register;
drive the top agent-scoped session-sized gap(s) through build+test; surface
new-subsystem GOs for the human.

## Technical Constraints

- Framework governance: new subsystems / >3 new files / new CLI route → require a
  human GO (autonomous boundaries; G-020 pickup-scope). Agent-scoped bounded fixes
  may be incepted+built+tested autonomously.
- Context budget: use parallel subagents returning summaries (not raw dumps) to keep
  orchestrator context lean.
- Off-.107 hosts (.121 no foothold, .141 offline) bound what "drive to live" can reach.

## Scope Fence

IN: TermLink's purpose/goals vs current implementation; gap identification +
prioritization; driving agent-scoped session-sized gaps to completion.
OUT: rewriting the framework (AEF) itself; large new subsystems without a human GO;
upgrading off-reach hosts (.121/.141 — external/foothold-blocked).

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

Exploration-first inception: gather evidence on purpose-vs-reality divergence across comms rail, coordination substrate, reliability/observability, and adoption/portability before recommending per-gap GO/NO-GO. No build authorized until the gap register is produced and reviewed.

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

### 2026-07-31T10:48:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
