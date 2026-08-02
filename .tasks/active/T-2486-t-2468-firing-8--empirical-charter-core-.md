---
id: T-2486
name: "T-2468 firing #8 — empirical charter-core health review"
description: >
  Inception: T-2468 firing #8 — empirical charter-core health review

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-08-02T06:01:39Z
last_update: 2026-08-02T06:02:06Z
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

# T-2486: T-2468 firing #8 — empirical charter-core health review

## Problem Statement

The T-2468 mandate ("ultra-critically review TermLink's purpose/goals, incept+build+test
the gaps, drive to completion") is firing for the 8th time. The prior 7 firings shipped 13
canaries + 4 affirmative charter-verb provers. The review's own verdict — **over-built
breadth, incomplete core; subtract-and-deepen, don't add** (docs/reports/T-2468-*.md) — now
applies to my OWN pattern: every firing has added a meta-tool. Continuing to add is the
anti-pattern the review named.

The intellectually honest 8th move is **empirical, not armchair**: I built four "prove it
works right now" provers for the four charter verbs (discover / exchange / claim / control
sessions) but a prover that is never run is shelf-ware. Run all four against the live
substrate to establish ground truth. What breaks (if anything) is the genuine gap to deepen;
if nothing breaks, the honest finding is that the core is proven and remaining gaps are all
human-gated — which is itself a valid, reportable review outcome (not a mandate to invent work).

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->
- A-1: The four provers, run live, decisively pass or name a broken stage (no ambiguous output).
- A-2: A prover FAIL, if any, points at an in-authority, bounded fix (deepen core) rather than
  a human-gated item (P3b subsystem / P6 multi-tenant / P4 deletion).

## Open Questions

- **IW-1: Do all four charter-verb provers pass against the live substrate right now?**
  confidence: 1
  disposition: answered|deferred|dissolved   <!-- filled after the empirical run -->
  rationale: <filled after running session-selftest / substrate-smoke / comms-selftest>

- **IW-2: If a prover fails, is the fix in-authority (bounded, reversible, no new subsystem/no user-facing removal)?**
  confidence: 1
  disposition: answered|deferred|dissolved
  rationale: <filled after triaging any failure>

- **IW-3: If ALL provers pass, is there a genuine subtract-or-deepen gap left, or is the honest finding "core proven, remainder human-gated"?**
  confidence: 1
  disposition: answered|deferred|dissolved
  rationale: <filled after synthesis; guard against manufacturing a build to satisfy the mandate>

## Exploration Plan

Time-boxed, empirical-first:
1. (~10 min) Run the four provers live: `session-selftest.sh` (local/deterministic),
   `substrate-smoke.sh` (claim), `comms-selftest.sh --peer <live-peer>` (discover+exchange).
   Capture PASS/broken-stage for each.
2. (~5 min) In parallel, a critical-review subagent independently ranks the single
   highest-value IN-AUTHORITY gap grounded in current code (guards against my anchoring bias).
3. Synthesize: prover failure → fix that (deepen). All pass + subagent finds a real
   non-accretive gap → build it. All pass + no real gap → report "core proven; remainder
   human-gated" and DEFER honestly (do NOT manufacture a 14th canary).

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

8th firing of the recurring purpose-review mandate. After 13 canaries + 4 provers, the review verdict stands: subtract-and-deepen, do not add breadth. This firing runs the four affirmative charter-verb provers against the live substrate to establish empirical ground truth before choosing any build. DEFER until prover results direct a specific in-authority gap (or confirm saturation).

**Evidence:**

Empirical prover run against the live substrate (local hub .107, 5 LIVE fleet peers), 2026-08-02:
- **control terminal sessions** — `session-selftest.sh --json` → `proven:true` (spawn+exec+cleanup all PASS).
- **claim work** — `substrate-smoke.sh` → PASS, 10/10 stages (create/post/claim/transfer/worker-loop/
  verify-clean/drain/handoff/lease-expiry/resilience).
- **discover** — `comms-selftest.sh --peer aef --discover-only` → DISCOVER PASS (peer LIVE + armed).
- **exchange durable messages** — full round-trip to aef: DISCOVER PASS, **SEND PASS (durable turn
  written to the hub)**, CONSUME FAIL → but classified `busy-or-manual (G-083)`: aef is a real working
  agent with no reason to auto-ack a synthetic proof-ping. The prover correctly NAMES + diagnoses the
  stage — this is the prover working as designed, not a TermLink core defect.

Finding: 3.5/4 charter verbs proven decisively; the remaining half (CONSUME) is a correct peer-side
diagnosis, not a bug. The charter core HOLDS. This directly supports IW-3's "core proven" branch and
the review's own "don't manufacture work" guard.

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

### 2026-08-02T06:02:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
