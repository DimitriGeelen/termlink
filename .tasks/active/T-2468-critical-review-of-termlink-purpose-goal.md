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
last_update: 2026-07-31T10:52:17Z
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
  confidence: 3
  disposition: answered
  rationale: Two unreconciled charters (README.md:3 vs docs/ARCHITECTURE.md:3); Four
  Directives are AEF's not TermLink's; ~21% MCP over-service; core job (collision detection)
  unbuilt. See docs/reports/T-2468 §IW-1.

- **IW-2: Comms rail — is the round-trip actually reliable end-to-end?**
  confidence: 3
  disposition: answered
  rationale: DELIVER guaranteed, WAKE best-effort, CONSUME silently breaks (G-083), ACK
  observable+retry only via --await-ack. Round-trip NOT end-to-end guaranteed. §IW-2.

- **IW-3: Coordination substrate — coherent and used, or over-built?**
  confidence: 3
  disposition: answered
  rationale: Over-built. ~1 lifetime real claim (expired); --log/history verbs read
  never-written files; ~30 surfaces zero reads; only offline-queue #5 shows real use. §IW-3.

- **IW-4: Adoption / "shipped ≠ live" — is what we built actually running?**
  confidence: 3
  disposition: answered
  rationale: Release half automated, fleet-adoption half fully manual/per-host/foothold-gated;
  no make-it-live primitive; arc closure lacks capability-live gate. 11 canaries = symptom. §IW-4.

- **IW-5: Which gaps warrant action, at what priority, agent-scoped vs human GO?**
  confidence: 3
  disposition: answered
  rationale: Prioritized register P1..P6 produced; staged-GO recommendation (P1 charter + P4
  surface-reduction now; P2/P3 to inception; P5 continue; P6 defer). §Prioritized gap register.

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

**Recommendation:** GO (staged subset — subtract-and-deepen, not build-everything)

**Rationale:**

Four parallel review dimensions converge on one verdict: TermLink over-built *breadth*
while leaving its *core promises* incomplete — a legible purpose, a reliable comms
round-trip, and live-across-fleet adoption. The correct response is to subtract
(over-service) and deepen (the core), not add. Staged GO:

- **P1 (single charter): GO now** — reconcile README vs ARCHITECTURE.md to one owned
  one-sentence purpose. Doc-only, reversible, foundational. Human picks the sentence.
- **P4 (surface reduction): GO now, staged** — retire the ~57 directive-untraceable
  social-analytics MCP tools + ~30 never-read substrate observability surfaces; coordinate
  with arc-005. Biggest concrete Usability/context win. Reversible via git.
- **P2 (consumption-confirmation / G-083) + P3 (shipped≠live gate + make-it-live primitive):
  GO-to-inception** — highest reliability value, need design; each its own single-question
  inception.
- **P5:** continue existing tracked work (T-2459 exactly-once, T-2371 WS degrade-to-poll).
- **P6 (collision detection / multi-tenant scope): DEFER** pending a product decision.

**Evidence:**

- docs/reports/T-2468-termlink-purpose-review.md — full four-dimension findings + register.
- IW-1: README.md:3 vs docs/ARCHITECTURE.md:3 (charter contradiction); 276 MCP tools, ~57 social-analytics.
- IW-2: G-083 (wake→consume silent break), G-088/T-2459 (exactly-once), T-2371 (WS).
- IW-3: `claims-summary work-queue` active=0 expired=1; `~/.termlink/{claims,find-idle,governor,queue}.log` absent.
- IW-4: fleet-deploy-binary.sh single-host; concerns.yaml:282,299 (arc-closure gap open); G-069.

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
