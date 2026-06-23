---
id: T-2261
name: "Pickup: Land no inter-hub federation by design (T-2229/PL-176/G-060) in shared FRAMEWORK.md — peers keep re-filing fix passive replication (ring20 R1, 3rd recurrence) (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-2259. Type: feature-proposal.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [pickup, feature-proposal]
components: []
related_tasks: []
created: 2026-06-23T20:15:01Z
last_update: 2026-06-23T20:55:49Z
date_finished: 2026-06-23T20:55:49Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
source_task_id_in_origin: T-2259
source_project_in_origin: "termlink"
---

# T-2261: Pickup: Land no inter-hub federation by design (T-2229/PL-176/G-060) in shared FRAMEWORK.md — peers keep re-filing fix passive replication (ring20 R1, 3rd recurrence) (from termlink)

## Problem Statement

**DUPLICATE — closed as dedup of T-2259.** Auto-created by the pickup observer ingesting termlink's *own* outbound envelope (no-inter-hub-federation correction (P-048 self-echo)) round-tripping back into its own pickup inbox via the channel-bridge mirror (a self-echo, not a peer filing — `source_project_in_origin: termlink`, `source_task_id_in_origin: T-2259`). The work is already owned by T-2259. No separate exploration warranted. Root issue: the pickup observer does not skip self-authored envelopes (filed upstream).

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

**Recommendation:** NO-GO (close as duplicate)
**Rationale:** Self-echo of termlink's own outbound pickup, not independent work. Live thread is T-2259. Tracking twice splits the decision and clutters the review queue.
**Evidence:**
- Frontmatter `source_project_in_origin: termlink`, `source_task_id_in_origin: T-2259` — self-authored.
- no-inter-hub-federation correction (P-048 self-echo).


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

### 2026-06-23T20:55:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-23T20:55:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Self-echo duplicate of T-2259 (own outbound pickup round-tripped); human-authorized dedup
