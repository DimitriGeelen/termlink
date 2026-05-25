---
id: T-1793
name: "Auto-federated channel topics across hubs — does the fleet want it? (T-1791 follow-up #3)"
description: >
  Inception: Auto-federated channel topics across hubs — does the fleet want it? (T-1791 follow-up #3)

status: started-work
workflow_type: inception
owner: human
horizon: later
tags: [federation, T-1791, G-060]
components: []
related_tasks: [T-1791, T-1166, T-1792]
revisit_at: 2026-08-21
revisit_evidence_needed: "Multiple agents independently surprised by per-hub channel-topic semantics despite T-1792 documentation, OR a concrete fleet-wide coordination workflow emerges that client-driven cross-posting can't serve cleanly."
created: 2026-05-21T19:14:37Z
last_update: 2026-05-21T19:16:21Z
date_finished: null
---

# T-1793: Auto-federated channel topics across hubs — does the fleet want it? (T-1791 follow-up #3)

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

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

T-1791 inception established TermLink has no inter-hub channel-topic federation primitive; cross-hub coordination is client-driven. This optional follow-up explores whether to ADD auto-federation as a feature. Benefits: cleaner agent UX, single source of truth across fleet, no need to remember --hub or remote_call for shared topics. Costs: state-sync complexity, consistency model choices (last-write-wins? vector clocks? CRDTs?), conflict resolution, bandwidth amplification on every post, ordering guarantees across hubs, retention divergence handling. Parked at horizon=later because: T-1166 retirement is not blocked, current client-driven pattern works correctly when used, and the architectural cost is significant. Revisit when: multiple agents independently surprised by per-hub semantics despite documentation (G-060 stays alive), OR a concrete fleet-wide coordination workflow emerges that the client-driven pattern can't serve cleanly.

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

### 2026-05-21T19:16:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
