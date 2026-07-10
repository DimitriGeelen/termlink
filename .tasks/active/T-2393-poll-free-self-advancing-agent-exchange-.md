---
id: T-2393
name: "Poll-free self-advancing agent exchange — flow without manual nudge"
description: >
  Inception: Poll-free self-advancing agent exchange — flow without manual nudge

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-07-10T14:08:25Z
last_update: 2026-07-10T14:10:07Z
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

# T-2393: Poll-free self-advancing agent exchange — flow without manual nudge

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

Two collaborating agents (e.g. aef ↔ workflow-designer on T-175) do not progress
on their own — every hand-off needs a human to manually nudge ("say check",
re-send). We shipped WAKE (push-waker rings an idle PTY) and DISCOVERY (presence
reads correctly, T-2390/91/92) and SEND — all verified this session — but the
conversation still STOPS after each message instead of FLOWING to its next real
blocker. Full diagnosis + proposed "relay loop" mechanism:
`docs/reports/T-2393-poll-free-self-advancing-agent-exchange-inception.md`.

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

- **IW-1: Autonomy budget — how many hops may an exchange self-advance between human checkpoints?**
  confidence: 1
  disposition: deferred
  rationale: sovereignty call for the human; agent recommends bounded-N with a circuit-breaker surfacing at the cap (relay-loop B3).

- **IW-2: Cost tolerance — is autonomous multi-hop (each hop = a full claude turn, no human in the loop) acceptable for design threads, or gated per-arc?**
  confidence: 1
  disposition: deferred
  rationale: token spend scales with hop count; human owns the spend/control trade-off.

- **IW-3: Continuation preamble — should the "advance-or-declare + reply-on-rail" instruction injected into a woken peer's session be a single framework-owned bounded preamble, or defined per-contract?**
  confidence: 2
  disposition: deferred
  rationale: it instructs another agent's session — must be framework-owned, bounded, non-spoofable; per-contract variants risk drift + injection surface.

- **IW-4: Is B1 (reply-on-ringing-rail default) sufficient on its own to remove the "say check" symptom, independent of B2/B3?**
  confidence: 2
  disposition: deferred
  rationale: routing the return leg onto the ringing DM rail closes the immediate stall; B2/B3 add true self-advance but B1 is the fast symptom-killer — validate empirically before committing to the full loop.

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

**Rationale:**

The WAKE layer is proven live (push-waker rings an idle PTY on the per-fp DM rail). Flow still stalls for two addressable reasons in termlink's own inject/rail layer: (1) replies land on shared thread/broadcast topics, which the waker does NOT ring, so the RETURN leg never wakes the sender — a routing gap; (2) a woken claude session processes one turn then idles, with nothing instructing it to work to its next real blocker and fire the next hop — a continuation gap the doorbell can shape via what it injects. Both are in-scope and testable; genuine human-GO gates (e.g. T-175 decomposition) are a correct stop, not part of this. Recommend GO to design the minimal mechanism that makes a two-agent exchange self-advance to its next real blocker with zero manual nudge.

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

### 2026-07-10T14:10:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
